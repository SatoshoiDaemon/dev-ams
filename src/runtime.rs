use crate::{
    actions::{
        accuracy_rating, evasion_rating, resolve_action_targets, ActionDefinition, ActionRejection,
        ActionStep,
    },
    attributes::{Attribute, AttributeRules},
    combat::{
        apply_status_modifiers_to_damage, resolve_attack, resolve_damage, ActionTraceContext,
        AttackOutcome, AttackRequest, DamageRequest, DamageResult, DamageStage, DeterministicRng,
        ExplainLast, ExplanationStore, ParryReaction,
    },
    effects::{ActiveStatus, StatusCategory, StatusError, StatusRegistry},
    events::CombatEvent,
    numeric::GameInt,
    scaling::{calculate_scaling, ScalingError, ScalingSource},
    session::{CombatSession, EngineCommand, QueuedEngineCommand, ReactionReservation},
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ActionResolution {
    pub action_id: String,
    pub actor_id: String,
    pub target_ids: Vec<String>,
    pub resolved_steps: usize,
    pub events: Vec<CombatEvent>,
    pub damage_results: Vec<DamageResult>,
    pub stages: Vec<DamageStage>,
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("action rejected: {0:?}")]
    Action(ActionRejection),
    #[error("unknown action {0}")]
    UnknownAction(String),
    #[error("unknown entity {0}")]
    UnknownEntity(String),
    #[error(transparent)]
    Combat(#[from] crate::combat::CombatError),
    #[error(transparent)]
    Scaling(#[from] ScalingError),
    #[error(transparent)]
    Status(#[from] StatusError),
    #[error(transparent)]
    Event(#[from] crate::events::EventError),
    #[error(transparent)]
    Numeric(#[from] crate::numeric::NumericError),
}

impl From<ActionRejection> for RuntimeError {
    fn from(value: ActionRejection) -> Self {
        Self::Action(value)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn resolve_declared_action(
    session: &mut CombatSession,
    definitions: &std::collections::BTreeMap<String, ActionDefinition>,
    rules: &AttributeRules,
    statuses: &StatusRegistry,
    rng: &mut DeterministicRng,
    explanations: &mut ExplanationStore,
    declaration: &crate::actions::ActionDeclaration,
    context: &ActionTraceContext,
) -> Result<ActionResolution, RuntimeError> {
    let rng_state_before = rng.state();
    let definition = definitions
        .get(&declaration.action_id)
        .ok_or_else(|| RuntimeError::UnknownAction(declaration.action_id.clone()))?;
    let actor = session
        .entities
        .get(&declaration.actor_id)
        .ok_or_else(|| RuntimeError::UnknownEntity(declaration.actor_id.clone()))?;
    if !actor.is_alive() {
        return Err(ActionRejection::ActorInactive.into());
    }
    if declaration.cast > 0
        && ((definition.properties.magical && actor.statuses.get("base:silent").is_some())
            || (definition.properties.weapon_dependent && actor.tags.contains("disarmed")))
    {
        return Err(ActionRejection::CastCancelled.into());
    }
    let target_ids = resolve_action_targets(
        actor,
        definition,
        &declaration.target_ids,
        &session.entities,
    )?;
    let mut result = ActionResolution {
        action_id: definition.id.clone(),
        actor_id: declaration.actor_id.clone(),
        target_ids: target_ids.clone(),
        resolved_steps: 0,
        events: Vec::new(),
        damage_results: Vec::new(),
        stages: Vec::new(),
    };
    let mut last_attack = None;
    let mut last_connected = true;
    session.events.begin_action();
    session.processed_events.clear();
    for step in &definition.steps {
        match step {
            ActionStep::Attack {
                damage_type,
                flat_damage,
                scaling,
            } => {
                last_connected = false;
                for target_id in &target_ids {
                    let source = session
                        .entities
                        .get(&declaration.actor_id)
                        .cloned()
                        .ok_or_else(|| RuntimeError::UnknownEntity(declaration.actor_id.clone()))?;
                    let target_snapshot = session
                        .entities
                        .get(target_id)
                        .cloned()
                        .ok_or_else(|| RuntimeError::UnknownEntity(target_id.clone()))?;
                    let accuracy =
                        accuracy_rating(source.attributes.get(Attribute::Precision), 0, 0, 0);
                    let evasion = evasion_rating(
                        target_snapshot.attributes.get(Attribute::Dexterity),
                        0,
                        0,
                        0,
                    );
                    let reaction = session.reactions.get(target_id).cloned();
                    let parry = match &reaction {
                        Some(ReactionReservation::Parry { source_id, rating }) => {
                            Some(ParryReaction {
                                source_id: source_id.clone(),
                                rating: *rating,
                                ap_available: session.budgets.reserved(target_id) >= 50,
                            })
                        }
                        _ => None,
                    };
                    let attack = resolve_attack(
                        &AttackRequest {
                            source_id: declaration.actor_id.clone(),
                            target_id: target_id.clone(),
                            accuracy_rating: accuracy,
                            evasion_rating: evasion,
                            unavoidable: definition.tags.iter().any(|tag| tag == "unavoidable"),
                            unparryable: definition.tags.iter().any(|tag| tag == "unparryable"),
                            parry,
                        },
                        rng,
                    )?;
                    for event in &attack.events {
                        session.events.publish(0, event.clone())?;
                        result.events.push(event.clone());
                    }
                    last_connected |= attack.outcome == AttackOutcome::Connected;
                    last_attack = Some(attack.clone());
                    if !matches!(attack.outcome, AttackOutcome::Evaded)
                        && matches!(reaction, Some(ReactionReservation::Parry { .. }))
                    {
                        session.budgets.consume_reserved(target_id, 50)?;
                        session.reactions.remove(target_id);
                    }
                    if attack.outcome != AttackOutcome::Connected {
                        continue;
                    }
                    let sources = scaling
                        .iter()
                        .map(|source| ScalingSource {
                            source_id: definition.source_id.clone(),
                            attribute: source.attribute,
                            grade: source.grade,
                        })
                        .collect::<Vec<_>>();
                    let scaled =
                        calculate_scaling(*flat_damage, &source.attributes, rules, &sources)?;
                    let mut damage = DamageRequest::from_scaling(
                        declaration.actor_id.clone(),
                        target_id.clone(),
                        *damage_type,
                        scaled,
                    );
                    damage.blockable = definition.properties.blockable;
                    if definition.properties.blockable {
                        if let Some(ReactionReservation::Block { power, .. }) = reaction {
                            if session.budgets.reserved(target_id) >= 50 {
                                session.budgets.consume_reserved(target_id, 50)?;
                                session.reactions.remove(target_id);
                                damage.block_power = power;
                            }
                        }
                    }
                    if session.demon_states.contains_key(target_id) {
                        damage.hp_thresholds.push(50);
                    }
                    apply_status_modifiers_to_damage(
                        &source,
                        &target_snapshot,
                        statuses,
                        &mut damage,
                    )?;
                    let damage_result = resolve_damage(
                        session
                            .entities
                            .get_mut(target_id)
                            .ok_or_else(|| RuntimeError::UnknownEntity(target_id.clone()))?,
                        &damage,
                    )?;
                    track_pending_death(session, &damage_result);
                    for event in &damage_result.events {
                        session.events.publish(0, event.clone())?;
                        result.events.push(event.clone());
                    }
                    result.stages.extend(damage_result.stages.clone());
                    result.damage_results.push(damage_result);
                }
            }
            ActionStep::DirectDamage {
                damage_type,
                amount,
            } => {
                for target_id in &target_ids {
                    let request = DamageRequest {
                        source_id: declaration.actor_id.clone(),
                        target_id: target_id.clone(),
                        damage_type: *damage_type,
                        flat_base_damage: *amount,
                        scaling_contributions: Vec::new(),
                        pre_mitigation_damage: *amount,
                        offensive_modifiers: Vec::new(),
                        defensive_modifiers: Vec::new(),
                        hp_modifiers: Vec::new(),
                        tenacity_ignore_bps: 0,
                        conditional_defense: 0,
                        damage_type_penetration: 0,
                        universal_penetration: 0,
                        block_power: 0,
                        blockable: definition.properties.blockable,
                        tenacity_hp_share_bps: 5_000,
                        hp_thresholds: session
                            .demon_states
                            .contains_key(target_id)
                            .then_some(vec![50])
                            .unwrap_or_default(),
                    };
                    let damage_result = resolve_damage(
                        session
                            .entities
                            .get_mut(target_id)
                            .ok_or_else(|| RuntimeError::UnknownEntity(target_id.clone()))?,
                        &request,
                    )?;
                    track_pending_death(session, &damage_result);
                    for event in &damage_result.events {
                        session.events.publish(0, event.clone())?;
                        result.events.push(event.clone());
                    }
                    result.stages.extend(damage_result.stages.clone());
                    result.damage_results.push(damage_result);
                }
            }
            ActionStep::ApplyStatus {
                effect_id,
                potency,
                counter,
            } if last_connected => {
                for target_id in &target_ids {
                    apply_status(
                        session,
                        statuses,
                        rules,
                        &declaration.actor_id,
                        target_id,
                        effect_id,
                        *potency,
                        *counter,
                    )?;
                }
            }
            ActionStep::ApplyStatus { .. } => {}
            ActionStep::RemoveStatus { effect_id } => {
                for target_id in &target_ids {
                    if session
                        .entities
                        .get_mut(target_id)
                        .and_then(|target| target.statuses.remove(effect_id))
                        .is_some()
                    {
                        let event = CombatEvent::StatusRemoved {
                            target_id: target_id.clone(),
                            effect_id: effect_id.clone(),
                            source_id: declaration.actor_id.clone(),
                        };
                        session.events.publish(0, event.clone())?;
                        result.events.push(event);
                    }
                }
            }
            ActionStep::Heal { amount } => {
                for target_id in &target_ids {
                    let target = session
                        .entities
                        .get_mut(target_id)
                        .ok_or_else(|| RuntimeError::UnknownEntity(target_id.clone()))?;
                    let restored = target.hp.restore(*amount);
                    result.stages.push(DamageStage {
                        stage: "heal".into(),
                        input: *amount,
                        output: restored,
                        detail: format!("HP restored to {target_id}"),
                    });
                }
            }
            ActionStep::Shield { amount } => {
                for target_id in &target_ids {
                    let target = session
                        .entities
                        .get_mut(target_id)
                        .ok_or_else(|| RuntimeError::UnknownEntity(target_id.clone()))?;
                    let before = target.shield;
                    target.shield = target.shield.max((*amount).max(0));
                    result.stages.push(DamageStage {
                        stage: "shield".into(),
                        input: before,
                        output: target.shield,
                        detail: format!("Shield replaced on {target_id} when stronger"),
                    });
                }
            }
            ActionStep::ChangeMana { amount } => {
                for target_id in &target_ids {
                    let target = session
                        .entities
                        .get_mut(target_id)
                        .ok_or_else(|| RuntimeError::UnknownEntity(target_id.clone()))?;
                    if *amount >= 0 {
                        target.mana.restore(*amount);
                    } else {
                        target.mana.reduce(amount.saturating_abs());
                    }
                }
            }
            ActionStep::Flee => session.flee(),
        }
        result.resolved_steps += 1;
    }
    let trigger_order = process_queued_events(session, statuses, rules)?;
    let last_damage = result.damage_results.last().cloned();
    let explanation = ExplainLast {
        ruleset_version: context.ruleset_version.clone(),
        rng_seed: context.seed,
        rng_state_before,
        rng_state_after: rng.state(),
        actor_id: declaration.actor_id.clone(),
        action_id: declaration.action_id.clone(),
        target_ids: target_ids.clone(),
        declared_round: declaration.declared_round,
        resolved: true,
        rejection_reason: None,
        payment_ap: declaration.ap_cost,
        payment_mana: declaration.mana_cost,
        eligible_round: declaration.eligible_round,
        queue_position: declaration.queue_position,
        action_stages: result.stages.clone(),
        trigger_order,
        attack: last_attack,
        damage: last_damage,
        events: result.events.clone(),
    };
    session.last_explanation = Some(explanation.clone());
    explanations.replace(explanation);
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn apply_status(
    session: &mut CombatSession,
    registry: &StatusRegistry,
    rules: &AttributeRules,
    source_id: &str,
    target_id: &str,
    effect_id: &str,
    potency: GameInt,
    counter: GameInt,
) -> Result<(), RuntimeError> {
    let definition = registry
        .get(effect_id)
        .ok_or_else(|| StatusError::Unknown(effect_id.into()))?;
    let target = session
        .entities
        .get_mut(target_id)
        .ok_or_else(|| RuntimeError::UnknownEntity(target_id.into()))?;
    if definition
        .immune_target_tags
        .iter()
        .any(|tag| target.tags.contains(tag))
    {
        return Ok(());
    }
    if !definition.suppresses_categories.is_empty() {
        let removed = target
            .statuses
            .iter()
            .filter_map(|(id, _)| {
                registry.get(id).and_then(|active_definition| {
                    (id != effect_id
                        && definition
                            .suppresses_categories
                            .iter()
                            .any(|category| active_definition.has_category(*category)))
                    .then(|| id.clone())
                })
            })
            .collect::<Vec<_>>();
        for id in removed {
            target.statuses.remove(&id);
        }
    }
    let resistance = target.attributes.get(Attribute::Resistance);
    let application = registry.apply(
        &mut target.statuses,
        ActiveStatus::new(effect_id, source_id, potency, counter),
        resistance,
        rules,
    )?;
    if !matches!(
        application.competition,
        crate::effects::CompetitionResult::Discarded { .. }
    ) {
        session.events.publish(
            0,
            CombatEvent::StatusApplied {
                target_id: target_id.into(),
                effect_id: effect_id.into(),
                source_id: source_id.into(),
            },
        )?;
    }
    Ok(())
}

pub fn process_queued_events(
    session: &mut CombatSession,
    registry: &StatusRegistry,
    rules: &AttributeRules,
) -> Result<Vec<String>, RuntimeError> {
    let mut order = Vec::new();
    loop {
        while let Some((depth, event)) = session.events.pop_front() {
            session
                .processed_events
                .push(crate::session::ProcessedCombatEvent {
                    depth,
                    event: event.clone(),
                });
            order.push(format!("{depth}:{event:?}"));
            match event {
                CombatEvent::TurnStarted { entity_id, .. } => {
                    process_dread_of_society(session, registry, rules, &entity_id)?;
                }
                CombatEvent::OnHpBelow {
                    target_id,
                    threshold_percent: 50,
                } => process_broken_heart(session, registry, rules, &target_id)?,
                _ => {}
            }
        }
        if let Some(queued) = session.commands.pop_front() {
            apply_engine_command(session, registry, rules, queued.command, queued.depth)?;
        } else {
            break;
        }
    }
    Ok(order)
}

fn process_dread_of_society(
    session: &mut CombatSession,
    registry: &StatusRegistry,
    _rules: &AttributeRules,
    entity_id: &str,
) -> Result<(), RuntimeError> {
    let Some(demon) = session.entities.get(entity_id) else {
        return Ok(());
    };
    if !demon.tags.contains("racial:dread-of-society") {
        return Ok(());
    }
    let allegiance = demon.allegiance;
    let qualifying = session
        .entities
        .values()
        .filter(|enemy| enemy.is_alive() && enemy.allegiance != allegiance)
        .filter(|enemy| {
            enemy.statuses.iter().any(|(id, _)| {
                registry.get(id).is_some_and(|definition| {
                    definition.has_category(StatusCategory::Debuff)
                        || definition.has_category(StatusCategory::CrowdControl)
                        || definition.has_category(StatusCategory::DamageOverTime)
                })
            })
        })
        .count() as GameInt;
    let demon = session
        .entities
        .get_mut(entity_id)
        .ok_or_else(|| RuntimeError::UnknownEntity(entity_id.into()))?;
    let hp = demon.hp.maximum.saturating_mul(qualifying) / 100;
    let mana = demon.mana.maximum.saturating_mul(qualifying) / 100;
    demon.hp.restore(hp);
    demon.mana.restore(mana);
    for effect_id in ["base:haste", "base:rage"] {
        if qualifying == 0 {
            demon.statuses.remove(effect_id);
        } else {
            demon.statuses.set_runtime(ActiveStatus::new(
                effect_id,
                "base:dread-of-society",
                qualifying.saturating_mul(2),
                0,
            ));
        }
    }
    Ok(())
}

fn process_broken_heart(
    session: &mut CombatSession,
    registry: &StatusRegistry,
    rules: &AttributeRules,
    entity_id: &str,
) -> Result<(), RuntimeError> {
    let Some(state) = session.demon_states.get_mut(entity_id) else {
        return Ok(());
    };
    if state.hearts == 0 || !state.was_at_or_above_half {
        return Ok(());
    }
    let target = session
        .entities
        .get(entity_id)
        .ok_or_else(|| RuntimeError::UnknownEntity(entity_id.into()))?;
    if target.hp.current.saturating_mul(2) >= target.hp.maximum {
        return Ok(());
    }
    state.hearts -= 1;
    state.was_at_or_above_half = false;
    let shield = target.hp.maximum.saturating_mul(20) / 100;
    let target = session
        .entities
        .get_mut(entity_id)
        .ok_or_else(|| RuntimeError::UnknownEntity(entity_id.into()))?;
    target.shield = target.shield.max(shield);
    let resistance = target.attributes.get(Attribute::Resistance);
    registry.apply(
        &mut target.statuses,
        ActiveStatus::new("base:broken-heart", entity_id, 15, 3),
        resistance,
        rules,
    )?;
    Ok(())
}

pub fn apply_engine_command(
    session: &mut CombatSession,
    registry: &StatusRegistry,
    rules: &AttributeRules,
    command: EngineCommand,
    depth: u8,
) -> Result<(), RuntimeError> {
    match command {
        EngineCommand::Damage {
            source_id,
            target_id,
            amount,
            damage_type,
        } => {
            let request = DamageRequest {
                source_id,
                target_id: target_id.clone(),
                damage_type,
                flat_base_damage: amount,
                scaling_contributions: Vec::new(),
                pre_mitigation_damage: amount,
                offensive_modifiers: Vec::new(),
                defensive_modifiers: Vec::new(),
                hp_modifiers: Vec::new(),
                tenacity_ignore_bps: 0,
                conditional_defense: 0,
                damage_type_penetration: 0,
                universal_penetration: 0,
                block_power: 0,
                blockable: true,
                tenacity_hp_share_bps: 5_000,
                hp_thresholds: Vec::new(),
            };
            let result = resolve_damage(
                session
                    .entities
                    .get_mut(&target_id)
                    .ok_or_else(|| RuntimeError::UnknownEntity(target_id.clone()))?,
                &request,
            )?;
            track_pending_death(session, &result);
            for event in result.events {
                session.events.publish(depth, event)?;
            }
        }
        EngineCommand::Heal { target_id, amount } => {
            session
                .entities
                .get_mut(&target_id)
                .ok_or_else(|| RuntimeError::UnknownEntity(target_id.clone()))?
                .hp
                .restore(amount);
        }
        EngineCommand::ChangeMana { target_id, amount } => {
            let pool = &mut session
                .entities
                .get_mut(&target_id)
                .ok_or_else(|| RuntimeError::UnknownEntity(target_id.clone()))?
                .mana;
            if amount >= 0 {
                pool.restore(amount);
            } else {
                pool.reduce(amount.saturating_abs());
            }
        }
        EngineCommand::ApplyStatus { target_id, status } => apply_status(
            session,
            registry,
            rules,
            &status.source_id,
            &target_id,
            &status.effect_id,
            status.potency,
            status.counter,
        )?,
        EngineCommand::RemoveStatus {
            target_id,
            effect_id,
        } => {
            session
                .entities
                .get_mut(&target_id)
                .ok_or_else(|| RuntimeError::UnknownEntity(target_id.clone()))?
                .statuses
                .remove(&effect_id);
        }
    }
    Ok(())
}

pub fn process_end_round(
    session: &mut CombatSession,
    registry: &StatusRegistry,
    rules: &AttributeRules,
) -> Result<Vec<String>, RuntimeError> {
    session.events.begin_action();
    session.processed_events.clear();
    let entity_ids = session.entities.keys().cloned().collect::<Vec<_>>();
    for entity_id in entity_ids {
        let ticks = {
            let target = session
                .entities
                .get_mut(&entity_id)
                .ok_or_else(|| RuntimeError::UnknownEntity(entity_id.clone()))?;
            registry.end_turn_ticks(&mut target.statuses, 1)?
        };
        for tick in ticks {
            if tick.damage > 0 {
                match tick.damage_type {
                    Some(crate::effects::StatusDamageType::Physical) => {
                        session.commands.push_back(QueuedEngineCommand {
                            depth: 1,
                            command: EngineCommand::Damage {
                                source_id: tick.source_id.clone(),
                                target_id: entity_id.clone(),
                                amount: tick.damage,
                                damage_type: crate::combat::DamageType::Physical,
                            },
                        });
                    }
                    Some(crate::effects::StatusDamageType::Magical) => {
                        session.commands.push_back(QueuedEngineCommand {
                            depth: 1,
                            command: EngineCommand::Damage {
                                source_id: tick.source_id.clone(),
                                target_id: entity_id.clone(),
                                amount: tick.damage,
                                damage_type: crate::combat::DamageType::Magical,
                            },
                        });
                    }
                    Some(crate::effects::StatusDamageType::True) => {
                        session.commands.push_back(QueuedEngineCommand {
                            depth: 1,
                            command: EngineCommand::Damage {
                                source_id: tick.source_id.clone(),
                                target_id: entity_id.clone(),
                                amount: tick.damage,
                                damage_type: crate::combat::DamageType::True,
                            },
                        });
                    }
                    Some(crate::effects::StatusDamageType::Tenacity) => {
                        let target = session
                            .entities
                            .get_mut(&entity_id)
                            .ok_or_else(|| RuntimeError::UnknownEntity(entity_id.clone()))?;
                        let applied = target.tenacity.reduce(tick.damage);
                        if applied > 0 {
                            session.events.publish(
                                1,
                                CombatEvent::OnTenacityDamage {
                                    source_id: tick.source_id.clone(),
                                    target_id: entity_id.clone(),
                                    amount: applied,
                                },
                            )?;
                        }
                    }
                    None => {}
                }
            }
            if tick.resource_loss > 0 {
                session.commands.push_back(QueuedEngineCommand {
                    depth: 1,
                    command: EngineCommand::ChangeMana {
                        target_id: entity_id.clone(),
                        amount: -tick.resource_loss,
                    },
                });
            }
            if tick.healing_percent > 0 {
                let maximum = session
                    .entities
                    .get(&entity_id)
                    .ok_or_else(|| RuntimeError::UnknownEntity(entity_id.clone()))?
                    .hp
                    .maximum;
                session.commands.push_back(QueuedEngineCommand {
                    depth: 1,
                    command: EngineCommand::Heal {
                        target_id: entity_id.clone(),
                        amount: crate::effects::heal_amount(maximum, tick.healing_percent)?,
                    },
                });
            }
            if tick.shield_potency > 0 {
                let amount = crate::effects::shield_amount(tick.shield_potency)?;
                if let Some(target) = session.entities.get_mut(&entity_id) {
                    target.shield = target.shield.max(amount);
                }
            }
            if tick.expired {
                session.events.publish(
                    1,
                    CombatEvent::StatusRemoved {
                        target_id: entity_id.clone(),
                        effect_id: tick.effect_id,
                        source_id: tick.source_id,
                    },
                )?;
            }
        }
    }
    let order = process_queued_events(session, registry, rules)?;
    for (id, state) in &mut session.demon_states {
        if let Some(entity) = session.entities.get(id) {
            if entity.hp.current.saturating_mul(2) >= entity.hp.maximum {
                state.was_at_or_above_half = true;
            }
        }
    }
    Ok(order)
}

fn track_pending_death(session: &mut CombatSession, result: &DamageResult) {
    if result.resources_before.hp > 0 && result.resources_after.hp == 0 {
        session
            .pending_deaths
            .entry(result.target_id.clone())
            .or_insert_with(|| result.source_id.clone());
    }
}

pub fn confirm_pending_deaths(
    session: &mut CombatSession,
) -> Result<Vec<CombatEvent>, RuntimeError> {
    let pending = std::mem::take(&mut session.pending_deaths);
    let mut events = Vec::new();
    for (target_id, source_id) in pending {
        let Some(target) = session.entities.get_mut(&target_id) else {
            continue;
        };
        if target.active && target.hp.current == 0 {
            target.active = false;
            let event = CombatEvent::OnKill {
                source_id,
                target_id,
            };
            session.events.publish(0, event.clone())?;
            events.push(event);
        }
    }
    Ok(events)
}
