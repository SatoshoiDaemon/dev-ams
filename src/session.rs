use crate::{
    actions::{
        build_action_queue, declare_action, pay_action_costs, ActionBudgets, ActionDeclaration,
        ActionDefinition, ActionRejection, BonusActionTracker, BONUS_ACTION_AP,
    },
    attributes::AttributeRules,
    combat::{CombatOutcome, ExplainLast},
    effects::ActiveStatus,
    entities::{Allegiance, Entity},
    events::{CombatEvent, EventBus},
    numeric::GameInt,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReactionReservation {
    Block { source_id: String, power: GameInt },
    Parry { source_id: String, rating: GameInt },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CombatPhase {
    Opening,
    Declaration,
    Resolution,
    EndRound,
    Complete,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingCast {
    pub declaration: ActionDeclaration,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DemonCombatState {
    pub hearts: u8,
    pub was_at_or_above_half: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum EngineCommand {
    Damage {
        source_id: String,
        target_id: String,
        amount: GameInt,
        damage_type: crate::combat::DamageType,
    },
    Heal {
        target_id: String,
        amount: GameInt,
    },
    ChangeMana {
        target_id: String,
        amount: GameInt,
    },
    ApplyStatus {
        target_id: String,
        status: ActiveStatus,
    },
    RemoveStatus {
        target_id: String,
        effect_id: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueuedEngineCommand {
    pub depth: u8,
    pub command: EngineCommand,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessedCombatEvent {
    pub depth: u8,
    pub event: CombatEvent,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CombatSession {
    pub encounter_id: String,
    pub round: u64,
    pub phase: CombatPhase,
    pub entities: BTreeMap<String, Entity>,
    #[serde(default)]
    pub available_actions: BTreeMap<String, Vec<String>>,
    pub declarations: Vec<ActionDeclaration>,
    pub eligible_queue: VecDeque<ActionDeclaration>,
    pub pending_casts: Vec<PendingCast>,
    pub budgets: ActionBudgets,
    #[serde(default)]
    pub reactions: BTreeMap<String, ReactionReservation>,
    pub events: EventBus,
    #[serde(default)]
    pub processed_events: Vec<ProcessedCombatEvent>,
    pub commands: VecDeque<QueuedEngineCommand>,
    #[serde(default)]
    pub pending_deaths: BTreeMap<String, String>,
    pub bonus_actions: BonusActionTracker,
    pub demon_states: BTreeMap<String, DemonCombatState>,
    pub outcome: Option<CombatOutcome>,
    pub opening_advantage_available: bool,
    pub actions_resolved: u64,
    #[serde(default)]
    pub last_explanation: Option<ExplainLast>,
}

impl CombatSession {
    pub fn new(encounter_id: impl Into<String>, entities: Vec<Entity>) -> Self {
        let mut indexed = BTreeMap::new();
        let mut demon_states = BTreeMap::new();
        for entity in entities {
            let id = entity.id.as_str().to_owned();
            if entity.race_id.as_ref().map(|race| race.as_str()) == Some("base:demon") {
                demon_states.insert(
                    id.clone(),
                    DemonCombatState {
                        hearts: 7,
                        was_at_or_above_half: entity.hp.current.saturating_mul(2)
                            >= entity.hp.maximum,
                    },
                );
            }
            indexed.insert(id, entity);
        }
        Self {
            encounter_id: encounter_id.into(),
            round: 0,
            phase: CombatPhase::Opening,
            entities: indexed,
            available_actions: BTreeMap::new(),
            declarations: Vec::new(),
            eligible_queue: VecDeque::new(),
            pending_casts: Vec::new(),
            budgets: ActionBudgets::default(),
            reactions: BTreeMap::new(),
            events: EventBus::default(),
            processed_events: Vec::new(),
            commands: VecDeque::new(),
            pending_deaths: BTreeMap::new(),
            bonus_actions: BonusActionTracker::default(),
            demon_states,
            outcome: None,
            opening_advantage_available: true,
            actions_resolved: 0,
            last_explanation: None,
        }
    }

    pub fn set_available_actions(&mut self, entity_id: &str, action_ids: Vec<String>) {
        self.available_actions.insert(entity_id.into(), action_ids);
    }

    pub fn start_round(&mut self) -> Result<(), crate::events::EventError> {
        if self.outcome.is_some() {
            self.phase = CombatPhase::Complete;
            return Ok(());
        }
        self.round = self.round.saturating_add(1);
        self.phase = CombatPhase::Declaration;
        self.declarations.clear();
        self.eligible_queue.clear();
        self.bonus_actions.begin_round();
        self.reactions.clear();
        self.events.begin_action();
        self.processed_events.clear();
        for (id, entity) in &self.entities {
            if entity.is_alive() && !entity.tags.contains("passive") {
                self.budgets.begin_turn(id);
                self.events.publish(
                    0,
                    CombatEvent::TurnStarted {
                        round: self.round,
                        entity_id: id.clone(),
                    },
                )?;
            }
        }
        Ok(())
    }

    pub fn submit_action(
        &mut self,
        actor_id: &str,
        definition: &ActionDefinition,
        target_ids: &[String],
        rules: &AttributeRules,
    ) -> Result<ActionDeclaration, ActionRejection> {
        if self.phase != CombatPhase::Declaration {
            return Err(ActionRejection::ActorInactive);
        }
        let declaration = {
            let actor = self
                .entities
                .get(actor_id)
                .ok_or(ActionRejection::UnknownActor)?;
            let targets: Result<Vec<_>, _> = target_ids
                .iter()
                .map(|id| self.entities.get(id).ok_or(ActionRejection::UnknownTarget))
                .collect();
            declare_action(
                actor,
                definition,
                &targets?,
                self.budgets.available(actor_id),
                self.round,
                rules,
            )?
        };
        let actor = self
            .entities
            .get_mut(actor_id)
            .ok_or(ActionRejection::UnknownActor)?;
        pay_action_costs(actor, &mut self.budgets, &declaration)?;
        if declaration.cast == 0 {
            self.declarations.push(declaration.clone());
        } else {
            self.pending_casts.push(PendingCast {
                declaration: declaration.clone(),
            });
        }
        Ok(declaration)
    }

    pub fn reserve_reaction(
        &mut self,
        actor_id: &str,
        definition: &ActionDefinition,
    ) -> Result<(), ActionRejection> {
        if !self.entities.contains_key(actor_id) {
            return Err(ActionRejection::UnknownActor);
        }
        if self
            .entities
            .get(actor_id)
            .is_some_and(|entity| entity.tags.contains("passive"))
        {
            return Err(ActionRejection::ActorPassive);
        }
        if self.reactions.contains_key(actor_id) {
            return Err(ActionRejection::ReactionAlreadyReserved);
        }
        if !definition.tags.iter().any(|tag| tag == "reaction") {
            return Err(ActionRejection::MissingTag);
        }
        let reaction = if definition.tags.iter().any(|tag| tag == "parry") {
            ReactionReservation::Parry {
                source_id: definition.source_id.clone(),
                rating: definition.properties.reaction_power.unwrap_or(0),
            }
        } else if definition.tags.iter().any(|tag| tag == "block") {
            ReactionReservation::Block {
                source_id: definition.source_id.clone(),
                power: definition.properties.reaction_power.unwrap_or(0),
            }
        } else {
            return Err(ActionRejection::MissingTag);
        };
        self.budgets.reserve(actor_id, definition.ap_cost)?;
        self.reactions.insert(actor_id.into(), reaction);
        Ok(())
    }

    pub fn close_declarations(&mut self) {
        let mut remaining = Vec::new();
        for pending in self.pending_casts.drain(..) {
            if pending.declaration.eligible_round <= self.round {
                self.declarations.push(pending.declaration);
            } else {
                remaining.push(pending);
            }
        }
        self.pending_casts = remaining;
        let mut queue = build_action_queue(std::mem::take(&mut self.declarations));
        if self.opening_advantage_available {
            if let Some(index) = queue
                .iter()
                .position(|action| action.actor_allegiance == Allegiance::Player)
            {
                let opening = queue.remove(index);
                queue.insert(0, opening);
                self.opening_advantage_available = false;
            }
        }
        for (position, declaration) in queue.iter_mut().enumerate() {
            declaration.queue_position = Some(position);
        }
        self.eligible_queue = queue.into();
        self.phase = CombatPhase::Resolution;
    }

    pub fn next_action(&mut self) -> Option<ActionDeclaration> {
        let next = self.eligible_queue.pop_front();
        if next.is_some() {
            self.actions_resolved = self.actions_resolved.saturating_add(1);
        } else if self.phase == CombatPhase::Resolution {
            self.phase = CombatPhase::EndRound;
        }
        next
    }

    #[allow(clippy::too_many_arguments)]
    pub fn insert_bonus_action(
        &mut self,
        actor_id: &str,
        definition: &ActionDefinition,
        target_ids: &[String],
        source_id: &str,
        parent_action: u64,
        depth: u8,
        rules: &AttributeRules,
    ) -> Result<ActionDeclaration, ActionRejection> {
        if self.phase != CombatPhase::Resolution || !definition.properties.bonus_eligible {
            return Err(ActionRejection::ActorInactive);
        }
        let mut bonus = definition.clone();
        bonus.ap_cost = BONUS_ACTION_AP;
        bonus.cast = 0;
        let targets = target_ids
            .iter()
            .map(|id| self.entities.get(id).ok_or(ActionRejection::UnknownTarget))
            .collect::<Result<Vec<_>, _>>()?;
        let mut declaration = declare_action(
            self.entities
                .get(actor_id)
                .ok_or(ActionRejection::UnknownActor)?,
            &bonus,
            &targets,
            self.budgets.available(actor_id),
            self.round,
            rules,
        )?;
        let mut updated_tracker = self.bonus_actions.clone();
        updated_tracker.register(actor_id, parent_action, source_id, depth)?;
        pay_action_costs(
            self.entities
                .get_mut(actor_id)
                .ok_or(ActionRejection::UnknownActor)?,
            &mut self.budgets,
            &declaration,
        )?;
        self.bonus_actions = updated_tracker;
        declaration.bonus_depth = depth;
        declaration.parent_action = Some(parent_action);
        declaration.queue_position = Some(0);
        self.eligible_queue.push_front(declaration.clone());
        Ok(declaration)
    }

    pub fn finish_round(&mut self) -> Result<(), crate::events::EventError> {
        for (id, entity) in &self.entities {
            if entity.is_alive() {
                self.events.publish(
                    0,
                    CombatEvent::TurnEnded {
                        round: self.round,
                        entity_id: id.clone(),
                    },
                )?;
            }
        }
        self.update_outcome();
        self.phase = if self.outcome.is_some() {
            CombatPhase::Complete
        } else {
            CombatPhase::Declaration
        };
        Ok(())
    }

    pub fn flee(&mut self) {
        self.outcome = Some(CombatOutcome::Fled);
        self.eligible_queue.clear();
        self.pending_casts.clear();
        self.phase = CombatPhase::Complete;
    }

    pub fn update_outcome(&mut self) {
        if self.outcome.is_some() {
            return;
        }
        let player_alive = self
            .entities
            .values()
            .any(|entity| entity.allegiance == Allegiance::Player && entity.is_alive());
        let enemy_alive = self
            .entities
            .values()
            .any(|entity| entity.allegiance == Allegiance::Enemy && entity.is_alive());
        self.outcome = match (player_alive, enemy_alive) {
            (true, false) => Some(CombatOutcome::Victory),
            (false, true | false) => Some(CombatOutcome::Defeat),
            (true, true) => None,
        };
    }
}
