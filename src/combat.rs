use crate::{
    effects::{StatusDamageType, StatusModifier, StatusRegistry},
    entities::Entity,
    events::CombatEvent,
    numeric::{
        apply_modifiers, checked_add, checked_sub, mul_div_floor, GameInt, Modifier, ModifierStep,
        NumericError, BASIS_POINTS,
    },
    scaling::{ScalingContribution, ScalingResult},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeterministicRng {
    state: u64,
}
impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
    pub fn range(&mut self, upper: u64) -> u64 {
        if upper == 0 {
            0
        } else {
            self.next_u64() % upper
        }
    }
    pub fn state(&self) -> u64 {
        self.state
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DamageType {
    Physical,
    Magical,
    True,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DamageRequest {
    pub source_id: String,
    pub target_id: String,
    pub damage_type: DamageType,
    pub flat_base_damage: GameInt,
    #[serde(default)]
    pub scaling_contributions: Vec<ScalingContribution>,
    pub pre_mitigation_damage: GameInt,
    #[serde(default)]
    pub offensive_modifiers: Vec<Modifier>,
    #[serde(default)]
    pub defensive_modifiers: Vec<Modifier>,
    #[serde(default)]
    pub hp_modifiers: Vec<Modifier>,
    pub tenacity_ignore_bps: GameInt,
    pub conditional_defense: GameInt,
    pub damage_type_penetration: GameInt,
    pub universal_penetration: GameInt,
    pub block_power: GameInt,
    pub blockable: bool,
    pub tenacity_hp_share_bps: GameInt,
    #[serde(default)]
    pub hp_thresholds: Vec<GameInt>,
}

impl DamageRequest {
    pub fn from_scaling(
        source_id: impl Into<String>,
        target_id: impl Into<String>,
        damage_type: DamageType,
        scaling: ScalingResult,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            target_id: target_id.into(),
            damage_type,
            flat_base_damage: scaling.flat_base_damage,
            scaling_contributions: scaling.contributions,
            pre_mitigation_damage: scaling.total,
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
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DamageStage {
    pub stage: String,
    pub input: GameInt,
    pub output: GameInt,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceSnapshot {
    pub shield: GameInt,
    pub tenacity: GameInt,
    pub hp: GameInt,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DamageResult {
    pub source_id: String,
    pub target_id: String,
    pub damage_type: DamageType,
    pub resources_before: ResourceSnapshot,
    pub resources_after: ResourceSnapshot,
    pub effective_defense: GameInt,
    pub blocked_damage: GameInt,
    pub shield_damage: GameInt,
    pub tenacity_damage: GameInt,
    pub hp_damage: GameInt,
    pub stages: Vec<DamageStage>,
    pub offensive_modifier_steps: Vec<ModifierStep>,
    pub defensive_modifier_steps: Vec<ModifierStep>,
    pub hp_modifier_steps: Vec<ModifierStep>,
    pub events: Vec<CombatEvent>,
}

#[derive(Debug, thiserror::Error)]
pub enum CombatError {
    #[error("damage request target {requested} does not match entity {actual}")]
    WrongTarget { requested: String, actual: String },
    #[error("damage cannot be negative, received {0}")]
    NegativeDamage(GameInt),
    #[error(transparent)]
    Numeric(#[from] NumericError),
}

pub fn resolve_damage(
    target: &mut Entity,
    request: &DamageRequest,
) -> Result<DamageResult, CombatError> {
    if target.id.as_str() != request.target_id {
        return Err(CombatError::WrongTarget {
            requested: request.target_id.clone(),
            actual: target.id.as_str().into(),
        });
    }
    if request.pre_mitigation_damage < 0 {
        return Err(CombatError::NegativeDamage(request.pre_mitigation_damage));
    }
    let before = ResourceSnapshot {
        shield: target.shield,
        tenacity: target.tenacity.current,
        hp: target.hp.current,
    };
    let mut stages = vec![DamageStage {
        stage: "base_and_scaling".into(),
        input: request.flat_base_damage,
        output: request.pre_mitigation_damage,
        detail: format!(
            "{} Scaling contribution(s)",
            request.scaling_contributions.len()
        ),
    }];

    let (offensive, offensive_modifier_steps) =
        apply_modifiers(request.pre_mitigation_damage, &request.offensive_modifiers)?;
    let offensive = offensive.max(0);
    stages.push(DamageStage {
        stage: "offensive_modifiers".into(),
        input: request.pre_mitigation_damage,
        output: offensive,
        detail: format!("{} modifier operation(s)", offensive_modifier_steps.len()),
    });

    let (defensive, defensive_modifier_steps) =
        apply_modifiers(offensive, &request.defensive_modifiers)?;
    let defensive = defensive.max(0);
    stages.push(DamageStage {
        stage: "defensive_modifiers".into(),
        input: offensive,
        output: defensive,
        detail: format!("{} modifier operation(s)", defensive_modifier_steps.len()),
    });

    let effective_defense = if request.damage_type == DamageType::True {
        0
    } else {
        checked_sub(
            checked_sub(
                checked_add(
                    target.defense,
                    request.conditional_defense,
                    "conditional Defense",
                )?,
                request.damage_type_penetration,
                "damage-type penetration",
            )?,
            request.universal_penetration,
            "universal penetration",
        )?
        .max(0)
    };
    let mitigated = if request.damage_type == DamageType::True || defensive == 0 {
        defensive
    } else {
        mul_div_floor(
            defensive,
            BASIS_POINTS,
            checked_add(BASIS_POINTS, effective_defense, "Defense denominator")?,
            "Defense mitigation",
        )?
        .max(1)
    };
    stages.push(DamageStage {
        stage: "defense_and_damage_type".into(),
        input: defensive,
        output: mitigated,
        detail: if request.damage_type == DamageType::True {
            "True Damage bypassed Defense".into()
        } else {
            format!("effective Defense {effective_defense}")
        },
    });

    let blocked_damage = if request.blockable {
        request.block_power.max(0).min(mitigated)
    } else {
        0
    };
    let after_block = mitigated - blocked_damage;
    stages.push(DamageStage {
        stage: "block".into(),
        input: mitigated,
        output: after_block,
        detail: format!("absorbed {blocked_damage}"),
    });

    let shield_damage = after_block.min(target.shield.max(0));
    target.shield -= shield_damage;
    let after_shield = after_block - shield_damage;
    stages.push(DamageStage {
        stage: "shield".into(),
        input: after_block,
        output: after_shield,
        detail: format!("absorbed {shield_damage}"),
    });

    let (mut requested_hp, mut requested_tenacity) =
        if request.damage_type == DamageType::True || target.tenacity.current <= 0 {
            (after_shield, 0)
        } else {
            damage::split_tenacity_damage_with_share(after_shield, request.tenacity_hp_share_bps)?
        };
    if requested_tenacity > 0 && request.tenacity_ignore_bps > 0 {
        let ignored = crate::numeric::apply_basis_points(
            requested_tenacity,
            request.tenacity_ignore_bps.min(BASIS_POINTS),
            "Tenacity ignore",
        )?;
        requested_tenacity -= ignored;
        requested_hp = checked_add(requested_hp, ignored, "ignored Tenacity to HP")?;
    }
    let tenacity_damage = target.tenacity.reduce(requested_tenacity);
    let tenacity_overflow = requested_tenacity - tenacity_damage;
    let hp_before_modifiers =
        checked_add(requested_hp, tenacity_overflow, "Tenacity overflow to HP")?;
    let (hp_request, hp_modifier_steps) =
        apply_modifiers(hp_before_modifiers, &request.hp_modifiers)?;
    let hp_request = hp_request.max(0);
    let hp_damage = target.hp.reduce(hp_request);
    stages.push(DamageStage {
        stage: "tenacity".into(),
        input: after_shield,
        output: after_shield - tenacity_damage,
        detail: if request.damage_type == DamageType::True {
            "True Damage bypassed Tenacity".into()
        } else {
            format!("received {tenacity_damage}; overflow {tenacity_overflow}")
        },
    });
    stages.push(DamageStage {
        stage: "hp_modifiers".into(),
        input: hp_before_modifiers,
        output: hp_request,
        detail: format!("{} modifier operation(s)", hp_modifier_steps.len()),
    });
    stages.push(DamageStage {
        stage: "hp".into(),
        input: hp_request,
        output: hp_damage,
        detail: format!("HP received {hp_damage}"),
    });

    let mut events = vec![CombatEvent::OnAttackReceived {
        source_id: request.source_id.clone(),
        target_id: request.target_id.clone(),
    }];
    if blocked_damage > 0 {
        events.push(CombatEvent::OnBlock {
            source_id: request.source_id.clone(),
            target_id: request.target_id.clone(),
            amount: blocked_damage,
        });
    }
    if shield_damage > 0 {
        events.push(CombatEvent::OnShieldDamage {
            source_id: request.source_id.clone(),
            target_id: request.target_id.clone(),
            amount: shield_damage,
        });
    }
    if tenacity_damage > 0 {
        events.push(CombatEvent::OnTenacityDamage {
            source_id: request.source_id.clone(),
            target_id: request.target_id.clone(),
            amount: tenacity_damage,
        });
    }
    let receiving_damage = checked_add(tenacity_damage, hp_damage, "receiving damage total")?;
    if receiving_damage > 0 {
        events.push(CombatEvent::OnDamageReceived {
            source_id: request.source_id.clone(),
            target_id: request.target_id.clone(),
            amount: receiving_damage,
        });
    }
    if hp_damage > 0 {
        events.push(CombatEvent::OnHpDamage {
            source_id: request.source_id.clone(),
            target_id: request.target_id.clone(),
            amount: hp_damage,
        });
    }
    let mut thresholds = request.hp_thresholds.clone();
    thresholds.sort_unstable();
    thresholds.dedup();
    for threshold in thresholds {
        if i128::from(before.hp) * 100 > i128::from(target.hp.maximum) * i128::from(threshold)
            && i128::from(target.hp.current) * 100
                <= i128::from(target.hp.maximum) * i128::from(threshold)
        {
            events.push(CombatEvent::OnHpBelow {
                target_id: request.target_id.clone(),
                threshold_percent: threshold,
            });
        }
    }
    if before.hp > 0 && target.hp.current == 0 {
        target.active = false;
        events.push(CombatEvent::OnKill {
            source_id: request.source_id.clone(),
            target_id: request.target_id.clone(),
        });
    }

    let after = ResourceSnapshot {
        shield: target.shield,
        tenacity: target.tenacity.current,
        hp: target.hp.current,
    };
    Ok(DamageResult {
        source_id: request.source_id.clone(),
        target_id: request.target_id.clone(),
        damage_type: request.damage_type,
        resources_before: before,
        resources_after: after,
        effective_defense,
        blocked_damage,
        shield_damage,
        tenacity_damage,
        hp_damage,
        stages,
        offensive_modifier_steps,
        defensive_modifier_steps,
        hp_modifier_steps,
        events,
    })
}

/// Converts active status definitions into named damage-pipeline modifiers.
pub fn apply_status_modifiers_to_damage(
    source: &Entity,
    target: &Entity,
    registry: &StatusRegistry,
    request: &mut DamageRequest,
) -> Result<(), CombatError> {
    for (_, status) in source.statuses.iter() {
        let Some(definition) = registry.get(&status.effect_id) else {
            continue;
        };
        for modifier in &definition.modifiers {
            if let StatusModifier::DamageDealtPercent {
                damage_type,
                per_potency_bps,
            } = modifier
            {
                if damage_type.map_or(true, |kind| {
                    status_damage_matches(kind, request.damage_type)
                }) {
                    request.offensive_modifiers.push(Modifier {
                        source_id: format!("{}@{}", status.effect_id, status.source_id),
                        operation: crate::numeric::ModifierOperation::AdditivePercent {
                            basis_points: per_potency_bps.checked_mul(status.potency).ok_or(
                                NumericError::Overflow {
                                    operation: "status damage dealt modifier",
                                    left: *per_potency_bps,
                                    right: status.potency,
                                },
                            )?,
                        },
                    });
                }
            }
        }
    }
    for (_, status) in target.statuses.iter() {
        let Some(definition) = registry.get(&status.effect_id) else {
            continue;
        };
        if status.effect_id == "base:charm" && source.id.as_str() != status.source_id {
            continue;
        }
        for modifier in &definition.modifiers {
            match modifier {
                StatusModifier::DamageReceivedPercent {
                    hp_only,
                    per_potency_bps,
                } => {
                    let entry = Modifier {
                        source_id: format!("{}@{}", status.effect_id, status.source_id),
                        operation: crate::numeric::ModifierOperation::AdditivePercent {
                            basis_points: per_potency_bps.checked_mul(status.potency).ok_or(
                                NumericError::Overflow {
                                    operation: "status damage received modifier",
                                    left: *per_potency_bps,
                                    right: status.potency,
                                },
                            )?,
                        },
                    };
                    if *hp_only {
                        request.hp_modifiers.push(entry);
                    } else {
                        request.defensive_modifiers.push(entry);
                    }
                }
                StatusModifier::TenacityIgnorePercent { per_potency_bps } => {
                    let contribution = per_potency_bps.checked_mul(status.potency).ok_or(
                        NumericError::Overflow {
                            operation: "status Tenacity ignore modifier",
                            left: *per_potency_bps,
                            right: status.potency,
                        },
                    )?;
                    request.tenacity_ignore_bps = checked_add(
                        request.tenacity_ignore_bps,
                        contribution,
                        "status Tenacity ignore total",
                    )?;
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn status_damage_matches(status_type: StatusDamageType, damage_type: DamageType) -> bool {
    matches!(
        (status_type, damage_type),
        (StatusDamageType::Physical, DamageType::Physical)
            | (StatusDamageType::Magical, DamageType::Magical)
            | (StatusDamageType::True, DamageType::True)
    )
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParryReaction {
    pub source_id: String,
    pub rating: GameInt,
    pub ap_available: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttackRequest {
    pub source_id: String,
    pub target_id: String,
    pub accuracy_rating: GameInt,
    pub evasion_rating: GameInt,
    pub unavoidable: bool,
    pub unparryable: bool,
    pub parry: Option<ParryReaction>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AttackOutcome {
    Evaded,
    Parried { parry_source_id: String },
    Connected,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttackResult {
    pub outcome: AttackOutcome,
    pub evasion_chance_bps: GameInt,
    pub evasion_roll: Option<GameInt>,
    pub parry_chance_bps: GameInt,
    pub parry_roll: Option<GameInt>,
    pub events: Vec<CombatEvent>,
}

pub fn resolve_attack(
    request: &AttackRequest,
    rng: &mut DeterministicRng,
) -> Result<AttackResult, CombatError> {
    let effective_evasion = (request.evasion_rating - request.accuracy_rating).max(0);
    let evasion_chance_bps = if request.unavoidable || request.evasion_rating <= 0 {
        0
    } else {
        mul_div_floor(
            effective_evasion,
            BASIS_POINTS,
            effective_evasion + 200,
            "Evasion chance",
        )?
    };
    let evasion_roll = (evasion_chance_bps > 0).then(|| rng.range(BASIS_POINTS as u64) as GameInt);
    if evasion_roll.is_some_and(|roll| roll < evasion_chance_bps) {
        return Ok(AttackResult {
            outcome: AttackOutcome::Evaded,
            evasion_chance_bps,
            evasion_roll,
            parry_chance_bps: 0,
            parry_roll: None,
            events: vec![CombatEvent::OnEvade {
                source_id: request.source_id.clone(),
                target_id: request.target_id.clone(),
            }],
        });
    }
    let effective_parry = request
        .parry
        .as_ref()
        .map_or(0, |parry| (parry.rating - request.accuracy_rating).max(0));
    let parry_chance_bps = if request.unparryable
        || request
            .parry
            .as_ref()
            .map_or(true, |parry| !parry.ap_available)
    {
        0
    } else {
        mul_div_floor(
            effective_parry,
            BASIS_POINTS,
            effective_parry + 150,
            "Parry chance",
        )?
        .min(7_500)
    };
    let parry_roll = (parry_chance_bps > 0).then(|| rng.range(BASIS_POINTS as u64) as GameInt);
    if parry_roll.is_some_and(|roll| roll < parry_chance_bps) {
        let source = request
            .parry
            .as_ref()
            .expect("positive Parry chance requires a reaction")
            .source_id
            .clone();
        return Ok(AttackResult {
            outcome: AttackOutcome::Parried {
                parry_source_id: source,
            },
            evasion_chance_bps,
            evasion_roll,
            parry_chance_bps,
            parry_roll,
            events: vec![CombatEvent::OnParry {
                source_id: request.source_id.clone(),
                target_id: request.target_id.clone(),
            }],
        });
    }
    Ok(AttackResult {
        outcome: AttackOutcome::Connected,
        evasion_chance_bps,
        evasion_roll,
        parry_chance_bps,
        parry_roll,
        events: Vec::new(),
    })
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CombatOutcome {
    Victory,
    Defeat,
    Fled,
    Aborted,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExplainLast {
    pub ruleset_version: String,
    pub rng_seed: u64,
    pub rng_state_after: u64,
    pub actor_id: String,
    pub action_id: String,
    pub target_ids: Vec<String>,
    pub declared_round: u64,
    pub resolved: bool,
    pub rejection_reason: Option<String>,
    pub attack: Option<AttackResult>,
    pub damage: Option<DamageResult>,
    pub events: Vec<CombatEvent>,
}

#[derive(Clone, Debug, Default)]
pub struct ExplanationStore {
    last: Option<ExplainLast>,
}
impl ExplanationStore {
    pub fn replace(&mut self, explanation: ExplainLast) {
        self.last = Some(explanation);
    }
    pub fn last(&self) -> Option<&ExplainLast> {
        self.last.as_ref()
    }
    pub fn render_json(&self) -> Result<Option<String>, serde_json::Error> {
        self.last
            .as_ref()
            .map(serde_json::to_string_pretty)
            .transpose()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedAttackAction {
    pub attack: AttackResult,
    pub damage: Option<DamageResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionTraceContext {
    pub seed: u64,
    pub ruleset_version: String,
    pub round: u64,
    pub action_id: String,
}

/// Resolves one complete attack action and atomically replaces `explain last`.
pub fn resolve_attack_action(
    explanations: &mut ExplanationStore,
    rng: &mut DeterministicRng,
    context: &ActionTraceContext,
    target: &mut Entity,
    attack_request: &AttackRequest,
    damage_request: &DamageRequest,
) -> Result<ResolvedAttackAction, CombatError> {
    let attack = resolve_attack(attack_request, rng)?;
    let damage = if attack.outcome == AttackOutcome::Connected {
        Some(resolve_damage(target, damage_request)?)
    } else {
        None
    };
    let mut events = attack.events.clone();
    if let Some(result) = &damage {
        events.extend(result.events.clone());
    }
    explanations.replace(ExplainLast {
        ruleset_version: context.ruleset_version.clone(),
        rng_seed: context.seed,
        rng_state_after: rng.state(),
        actor_id: attack_request.source_id.clone(),
        action_id: context.action_id.clone(),
        target_ids: vec![attack_request.target_id.clone()],
        declared_round: context.round,
        resolved: true,
        rejection_reason: None,
        attack: Some(attack.clone()),
        damage: damage.clone(),
        events,
    });
    Ok(ResolvedAttackAction { attack, damage })
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RejectionTrace {
    pub context: ActionTraceContext,
    pub rng_state: u64,
    pub actor_id: String,
    pub target_ids: Vec<String>,
    pub reason: String,
}

pub fn explain_rejection(explanations: &mut ExplanationStore, rejection: RejectionTrace) {
    explanations.replace(ExplainLast {
        ruleset_version: rejection.context.ruleset_version,
        rng_seed: rejection.context.seed,
        rng_state_after: rejection.rng_state,
        actor_id: rejection.actor_id,
        action_id: rejection.context.action_id,
        target_ids: rejection.target_ids,
        declared_round: rejection.context.round,
        resolved: false,
        rejection_reason: Some(rejection.reason),
        attack: None,
        damage: None,
        events: Vec::new(),
    });
}

pub mod damage {
    use crate::numeric::{mul_div_floor, GameInt, NumericError, BASIS_POINTS};
    /// Returns `(HP share, Tenacity share)`. Odd points go to HP.
    pub fn split_tenacity_damage(damage: GameInt) -> (GameInt, GameInt) {
        split_tenacity_damage_with_share(damage, 5_000)
            .expect("the canonical 5000 basis-point share is valid")
    }

    pub fn split_tenacity_damage_with_share(
        damage: GameInt,
        hp_share_bps: GameInt,
    ) -> Result<(GameInt, GameInt), NumericError> {
        let damage = damage.max(0);
        let hp_share_bps = hp_share_bps.clamp(0, BASIS_POINTS);
        let tenacity = mul_div_floor(
            damage,
            BASIS_POINTS - hp_share_bps,
            BASIS_POINTS,
            "Tenacity damage share",
        )?;
        Ok((damage - tenacity, tenacity))
    }
}
