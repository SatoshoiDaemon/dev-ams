use crate::{
    attributes::{Attribute, AttributeRules},
    entities::{Allegiance, Entity},
    numeric::{checked_add, GameInt},
};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::BTreeMap};

pub const DEFAULT_AP: GameInt = 100;
pub const STANDARD_ACTION_AP: GameInt = 100;
pub const QUICK_ACTION_AP: GameInt = 50;
pub const FREE_ACTION_AP: GameInt = 0;
pub const BONUS_ACTION_AP: GameInt = 50;
pub const MAX_BONUS_ACTION_DEPTH: u8 = 2;
pub const MAX_BONUS_ACTIONS_PER_ROUND: u8 = 3;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetMode {
    SelfOnly,
    Single,
    Multiple { maximum: usize },
    All,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetRelation {
    SelfOnly,
    Ally,
    Enemy,
    Any,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetState {
    Active,
    Dead,
    Any,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActionRequirement {
    ActorActive,
    ActorHasStatus { status_id: String },
    ActorLacksStatus { status_id: String },
    TargetHasStatus { status_id: String },
    TargetHpAtOrBelowPercent { percent: GameInt },
    MinimumMana { amount: GameInt },
    RequiresTag { tag: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TargetSpec {
    pub mode: TargetMode,
    pub relation: TargetRelation,
    pub state: TargetState,
    #[serde(default)]
    pub filters: Vec<ActionRequirement>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionDefinition {
    pub id: String,
    pub source_id: String,
    pub target: TargetSpec,
    pub ap_cost: GameInt,
    pub mana_cost: GameInt,
    pub cast: u32,
    pub action_priority: GameInt,
    #[serde(default)]
    pub requirements: Vec<ActionRequirement>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionDeclaration {
    pub actor_id: String,
    pub action_id: String,
    pub source_id: String,
    pub target_ids: Vec<String>,
    pub ap_cost: GameInt,
    pub mana_cost: GameInt,
    pub cast: u32,
    pub declared_round: u64,
    pub eligible_round: u64,
    pub action_priority: GameInt,
    pub final_priority: GameInt,
    pub actor_allegiance: Allegiance,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionRejection {
    UnknownActor,
    ActorInactive,
    InsufficientAp,
    InsufficientMana,
    InvalidTargetCount,
    UnknownTarget,
    InvalidTargetState,
    InvalidTargetRelation,
    MissingStatus,
    ForbiddenStatus,
    HpThresholdNotMet,
    MissingTag,
    CastCancelled,
    BonusDepthExceeded,
    BonusLimitExceeded,
    RepeatedBonusSource,
    NumericOverflow,
    InvalidCost,
}

pub fn declare_action(
    actor: &Entity,
    definition: &ActionDefinition,
    targets: &[&Entity],
    available_ap: GameInt,
    round: u64,
    rules: &AttributeRules,
) -> Result<ActionDeclaration, ActionRejection> {
    if !actor.is_alive() {
        return Err(ActionRejection::ActorInactive);
    }
    if definition.ap_cost < 0 || definition.mana_cost < 0 {
        return Err(ActionRejection::InvalidCost);
    }
    validate_target_count(definition.target.mode, targets.len())?;
    for target in targets {
        validate_target(actor, target, &definition.target)?;
    }
    validate_requirements(actor, targets, &definition.requirements, &definition.tags)?;
    if available_ap < definition.ap_cost {
        return Err(ActionRejection::InsufficientAp);
    }
    if actor.mana.current < definition.mana_cost {
        return Err(ActionRejection::InsufficientMana);
    }
    let derived = rules
        .derive(&actor.attributes)
        .map_err(|_| ActionRejection::ActorInactive)?;
    let final_priority = checked_add(
        checked_add(
            checked_add(
                derived.priority,
                actor.equipment_priority,
                "equipment Priority",
            )
            .map_err(|_| ActionRejection::NumericOverflow)?,
            actor.status_priority,
            "status Priority",
        )
        .map_err(|_| ActionRejection::NumericOverflow)?,
        definition.action_priority,
        "action Priority",
    )
    .map_err(|_| ActionRejection::NumericOverflow)?;
    Ok(ActionDeclaration {
        actor_id: actor.id.as_str().into(),
        action_id: definition.id.clone(),
        source_id: definition.source_id.clone(),
        target_ids: targets
            .iter()
            .map(|target| target.id.as_str().into())
            .collect(),
        ap_cost: definition.ap_cost,
        mana_cost: definition.mana_cost,
        cast: definition.cast,
        declared_round: round,
        eligible_round: round + u64::from(definition.cast),
        action_priority: definition.action_priority,
        final_priority,
        actor_allegiance: actor.allegiance,
    })
}

fn validate_target_count(mode: TargetMode, count: usize) -> Result<(), ActionRejection> {
    let valid = match mode {
        TargetMode::SelfOnly | TargetMode::Single => count == 1,
        TargetMode::Multiple { maximum } => count > 0 && count <= maximum,
        TargetMode::All => count == 0,
    };
    valid
        .then_some(())
        .ok_or(ActionRejection::InvalidTargetCount)
}

fn validate_target(
    actor: &Entity,
    target: &Entity,
    specification: &TargetSpec,
) -> Result<(), ActionRejection> {
    let state_valid = match specification.state {
        TargetState::Active => target.is_alive(),
        TargetState::Dead => !target.is_alive(),
        TargetState::Any => true,
    };
    if !state_valid {
        return Err(ActionRejection::InvalidTargetState);
    }
    let is_self = actor.id == target.id;
    let relation_valid = match specification.relation {
        TargetRelation::SelfOnly => is_self,
        TargetRelation::Ally => !is_self && actor.allegiance == target.allegiance,
        TargetRelation::Enemy => actor.allegiance != target.allegiance,
        TargetRelation::Any => true,
    };
    if !relation_valid {
        return Err(ActionRejection::InvalidTargetRelation);
    }
    validate_requirements(actor, &[target], &specification.filters, &[])
}

fn validate_requirements(
    actor: &Entity,
    targets: &[&Entity],
    requirements: &[ActionRequirement],
    tags: &[String],
) -> Result<(), ActionRejection> {
    for requirement in requirements {
        match requirement {
            ActionRequirement::ActorActive if !actor.is_alive() => {
                return Err(ActionRejection::ActorInactive)
            }
            ActionRequirement::ActorHasStatus { status_id }
                if actor.statuses.get(status_id).is_none() =>
            {
                return Err(ActionRejection::MissingStatus)
            }
            ActionRequirement::ActorLacksStatus { status_id }
                if actor.statuses.get(status_id).is_some() =>
            {
                return Err(ActionRejection::ForbiddenStatus)
            }
            ActionRequirement::TargetHasStatus { status_id }
                if targets
                    .iter()
                    .any(|target| target.statuses.get(status_id).is_none()) =>
            {
                return Err(ActionRejection::MissingStatus)
            }
            ActionRequirement::TargetHpAtOrBelowPercent { percent }
                if targets.iter().any(|target| {
                    i128::from(target.hp.current) * 100
                        > i128::from(target.hp.maximum) * i128::from(*percent)
                }) =>
            {
                return Err(ActionRejection::HpThresholdNotMet)
            }
            ActionRequirement::MinimumMana { amount } if actor.mana.current < *amount => {
                return Err(ActionRejection::InsufficientMana)
            }
            ActionRequirement::RequiresTag { tag } if !tags.contains(tag) => {
                return Err(ActionRejection::MissingTag)
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn build_action_queue(mut actions: Vec<ActionDeclaration>) -> Vec<ActionDeclaration> {
    actions.sort_by(|left, right| {
        right
            .final_priority
            .cmp(&left.final_priority)
            .then_with(|| match (left.actor_allegiance, right.actor_allegiance) {
                (Allegiance::Player, Allegiance::Player)
                | (Allegiance::Enemy, Allegiance::Enemy)
                | (Allegiance::Neutral, Allegiance::Neutral) => Ordering::Equal,
                (Allegiance::Player, _) => Ordering::Less,
                (_, Allegiance::Player) => Ordering::Greater,
                (Allegiance::Enemy, Allegiance::Neutral) => Ordering::Less,
                (Allegiance::Neutral, Allegiance::Enemy) => Ordering::Greater,
            })
            .then_with(|| left.actor_id.cmp(&right.actor_id))
    });
    actions
}

#[derive(Clone, Debug, Default)]
pub struct ActionBudgets {
    ap: BTreeMap<String, GameInt>,
}
impl ActionBudgets {
    pub fn begin_turn(&mut self, entity_id: &str) {
        self.ap.insert(entity_id.into(), DEFAULT_AP);
    }
    pub fn available(&self, entity_id: &str) -> GameInt {
        self.ap.get(entity_id).copied().unwrap_or(0)
    }
    pub fn pay(&mut self, entity_id: &str, amount: GameInt) -> Result<(), ActionRejection> {
        if amount < 0 {
            return Err(ActionRejection::InvalidCost);
        }
        let available = self.available(entity_id);
        if available < amount {
            return Err(ActionRejection::InsufficientAp);
        }
        self.ap.insert(entity_id.into(), available - amount);
        Ok(())
    }
}

pub fn accuracy_rating(
    precision: GameInt,
    equipment: GameInt,
    effects: GameInt,
    ability: GameInt,
) -> GameInt {
    precision / 5 + equipment + effects + ability
}
pub fn evasion_rating(
    dexterity: GameInt,
    equipment: GameInt,
    effects: GameInt,
    ability: GameInt,
) -> GameInt {
    dexterity / 5 + equipment + effects + ability
}
pub fn actor_dexterity(entity: &Entity) -> GameInt {
    entity.attributes.get(Attribute::Dexterity)
}
