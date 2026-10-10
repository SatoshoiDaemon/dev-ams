use crate::{
    attributes::{Attribute, AttributeRules},
    combat::DamageType,
    effects::StatusRegistry,
    entities::{Allegiance, Entity},
    numeric::{checked_add, GameInt},
    scaling::ScalingGrade,
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

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
    None,
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
    ActorHasTag { tag: String },
    ActorHasStatus { status_id: String },
    ActorLacksStatus { status_id: String },
    TargetHasStatus { status_id: String },
    TargetLacksStatus { status_id: String },
    TargetHpAtOrBelowPercent { percent: GameInt },
    MinimumMana { amount: GameInt },
    RequiresTag { tag: String },
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionAvailability {
    Exploration,
    #[default]
    Combat,
    Both,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ActionProperties {
    pub offensive: bool,
    pub blockable: bool,
    pub interruptible: bool,
    pub bonus_eligible: bool,
    pub magical: bool,
    pub weapon_dependent: bool,
    pub refundable: bool,
    pub reaction_power: Option<GameInt>,
}

impl Default for ActionProperties {
    fn default() -> Self {
        Self {
            offensive: false,
            blockable: true,
            interruptible: true,
            bonus_eligible: false,
            magical: false,
            weapon_dependent: false,
            refundable: false,
            reaction_power: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionScaling {
    pub attribute: Attribute,
    pub grade: ScalingGrade,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActionStep {
    Attack {
        damage_type: DamageType,
        flat_damage: GameInt,
        #[serde(default)]
        scaling: Vec<ActionScaling>,
    },
    DirectDamage {
        damage_type: DamageType,
        amount: GameInt,
    },
    ApplyStatus {
        effect_id: String,
        potency: GameInt,
        counter: GameInt,
    },
    RemoveStatus {
        effect_id: String,
    },
    Heal {
        amount: GameInt,
    },
    Shield {
        amount: GameInt,
    },
    ChangeMana {
        amount: GameInt,
    },
    Flee,
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
    #[serde(default)]
    pub replacement_of: Option<String>,
    pub source_id: String,
    pub target: TargetSpec,
    pub ap_cost: GameInt,
    pub mana_cost: GameInt,
    pub cast: u32,
    pub action_priority: GameInt,
    #[serde(default)]
    pub availability: ActionAvailability,
    #[serde(default)]
    pub properties: ActionProperties,
    #[serde(default)]
    pub requirements: Vec<ActionRequirement>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub steps: Vec<ActionStep>,
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
    #[serde(default)]
    pub bonus_depth: u8,
    #[serde(default)]
    pub parent_action: Option<u64>,
    #[serde(default)]
    pub queue_position: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionRejection {
    UnknownActor,
    ActorInactive,
    ActorPassive,
    InsufficientAp,
    InsufficientMana,
    InvalidTargetCount,
    UnknownTarget,
    InvalidTargetState,
    InvalidTargetRelation,
    MissingStatus,
    ForbiddenStatus,
    MissingActorTag,
    HpThresholdNotMet,
    MissingTag,
    CastCancelled,
    BonusDepthExceeded,
    BonusLimitExceeded,
    RepeatedBonusSource,
    ReactionAlreadyReserved,
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
    if actor.tags.contains("passive") {
        return Err(ActionRejection::ActorPassive);
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
        bonus_depth: 0,
        parent_action: None,
        queue_position: None,
    })
}

fn validate_target_count(mode: TargetMode, count: usize) -> Result<(), ActionRejection> {
    let valid = match mode {
        TargetMode::None => count == 0,
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
            ActionRequirement::ActorHasTag { tag } if !actor.tags.contains(tag) => {
                return Err(ActionRejection::MissingActorTag)
            }
            ActionRequirement::TargetHasStatus { status_id }
                if targets
                    .iter()
                    .any(|target| target.statuses.get(status_id).is_none()) =>
            {
                return Err(ActionRejection::MissingStatus)
            }
            ActionRequirement::TargetLacksStatus { status_id }
                if targets
                    .iter()
                    .any(|target| target.statuses.get(status_id).is_some()) =>
            {
                return Err(ActionRejection::ForbiddenStatus)
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

pub fn resolve_action_targets(
    actor: &Entity,
    definition: &ActionDefinition,
    selected_ids: &[String],
    entities: &BTreeMap<String, Entity>,
) -> Result<Vec<String>, ActionRejection> {
    let candidates: Vec<&Entity> = match definition.target.mode {
        TargetMode::All => entities
            .values()
            .filter(|target| validate_target(actor, target, &definition.target).is_ok())
            .collect(),
        _ => selected_ids
            .iter()
            .filter_map(|id| entities.get(id))
            .filter(|target| validate_target(actor, target, &definition.target).is_ok())
            .collect(),
    };
    let minimum_met = match definition.target.mode {
        TargetMode::None => candidates.is_empty(),
        TargetMode::SelfOnly | TargetMode::Single => candidates.len() == 1,
        TargetMode::Multiple { .. } | TargetMode::All => !candidates.is_empty(),
    };
    if !minimum_met {
        return Err(ActionRejection::InvalidTargetState);
    }
    validate_requirements(
        actor,
        &candidates,
        &definition.requirements,
        &definition.tags,
    )?;
    Ok(candidates
        .into_iter()
        .map(|target| target.id.as_str().to_owned())
        .collect())
}

#[derive(Clone, Debug, Default)]
pub struct ActionRegistry {
    definitions: BTreeMap<String, ActionDefinition>,
}

#[derive(Debug, thiserror::Error)]
pub enum ActionContentError {
    #[error("failed to read action definitions {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("invalid action definitions {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("duplicate action ID {0}")]
    Duplicate(String),
    #[error("action {id} declares replacement of missing or different target {target}")]
    InvalidReplacement { id: String, target: String },
    #[error("action {action_id} references unknown status {status_id}")]
    UnknownStatus {
        action_id: String,
        status_id: String,
    },
}

impl ActionRegistry {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ActionContentError> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|source| ActionContentError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let definitions: Vec<ActionDefinition> =
            serde_json::from_str(&text).map_err(|source| ActionContentError::Json {
                path: path.to_path_buf(),
                source,
            })?;
        let mut registry = Self::default();
        for definition in definitions {
            registry.register(definition)?;
        }
        Ok(registry)
    }

    pub fn register(&mut self, definition: ActionDefinition) -> Result<(), ActionContentError> {
        if self.definitions.contains_key(&definition.id) {
            if definition.replacement_of.as_deref() != Some(definition.id.as_str()) {
                return Err(ActionContentError::Duplicate(definition.id));
            }
        } else if let Some(target) = &definition.replacement_of {
            return Err(ActionContentError::InvalidReplacement {
                id: definition.id,
                target: target.clone(),
            });
        }
        self.definitions.insert(definition.id.clone(), definition);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&ActionDefinition> {
        self.definitions.get(id)
    }

    pub fn validate(&self, statuses: &StatusRegistry) -> Result<(), ActionContentError> {
        for definition in self.definitions.values() {
            for step in &definition.steps {
                if let ActionStep::ApplyStatus { effect_id, .. }
                | ActionStep::RemoveStatus { effect_id } = step
                {
                    if statuses.get(effect_id).is_none() {
                        return Err(ActionContentError::UnknownStatus {
                            action_id: definition.id.clone(),
                            status_id: effect_id.clone(),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    pub fn as_map(&self) -> &BTreeMap<String, ActionDefinition> {
        &self.definitions
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &ActionDefinition)> {
        self.definitions.iter()
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionBudgets {
    ap: BTreeMap<String, GameInt>,
    reserved: BTreeMap<String, GameInt>,
}
impl ActionBudgets {
    pub fn begin_turn(&mut self, entity_id: &str) {
        self.ap.insert(entity_id.into(), DEFAULT_AP);
        self.reserved.remove(entity_id);
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

    pub fn reserve(&mut self, entity_id: &str, amount: GameInt) -> Result<(), ActionRejection> {
        if amount < 0 {
            return Err(ActionRejection::InvalidCost);
        }
        let available = self.available(entity_id);
        if available < amount {
            return Err(ActionRejection::InsufficientAp);
        }
        let reserved = self.reserved.get(entity_id).copied().unwrap_or(0);
        let updated = reserved
            .checked_add(amount)
            .ok_or(ActionRejection::NumericOverflow)?;
        self.ap.insert(entity_id.into(), available - amount);
        self.reserved.insert(entity_id.into(), updated);
        Ok(())
    }

    pub fn reserved(&self, entity_id: &str) -> GameInt {
        self.reserved.get(entity_id).copied().unwrap_or(0)
    }

    pub fn consume_reserved(
        &mut self,
        entity_id: &str,
        amount: GameInt,
    ) -> Result<(), ActionRejection> {
        let available = self.reserved(entity_id);
        if amount < 0 {
            return Err(ActionRejection::InvalidCost);
        }
        if available < amount {
            return Err(ActionRejection::InsufficientAp);
        }
        self.reserved.insert(entity_id.into(), available - amount);
        Ok(())
    }
}

pub fn pay_action_costs(
    actor: &mut Entity,
    budgets: &mut ActionBudgets,
    declaration: &ActionDeclaration,
) -> Result<(), ActionRejection> {
    if budgets.available(actor.id.as_str()) < declaration.ap_cost {
        return Err(ActionRejection::InsufficientAp);
    }
    if actor.mana.current < declaration.mana_cost {
        return Err(ActionRejection::InsufficientMana);
    }
    budgets.pay(actor.id.as_str(), declaration.ap_cost)?;
    actor.mana.reduce(declaration.mana_cost);
    Ok(())
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BonusActionTracker {
    per_actor: BTreeMap<String, u8>,
    used_sources: BTreeMap<u64, Vec<String>>,
}

impl BonusActionTracker {
    pub fn register(
        &mut self,
        actor_id: &str,
        parent_action: u64,
        source_id: &str,
        depth: u8,
    ) -> Result<(), ActionRejection> {
        if depth > MAX_BONUS_ACTION_DEPTH {
            return Err(ActionRejection::BonusDepthExceeded);
        }
        let count = self.per_actor.get(actor_id).copied().unwrap_or(0);
        if count >= MAX_BONUS_ACTIONS_PER_ROUND {
            return Err(ActionRejection::BonusLimitExceeded);
        }
        let sources = self.used_sources.entry(parent_action).or_default();
        if sources.iter().any(|existing| existing == source_id) {
            return Err(ActionRejection::RepeatedBonusSource);
        }
        sources.push(source_id.into());
        self.per_actor.insert(actor_id.into(), count + 1);
        Ok(())
    }

    pub fn begin_round(&mut self) {
        self.per_actor.clear();
        self.used_sources.clear();
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
