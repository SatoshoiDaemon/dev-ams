//! Data-driven status definitions and deterministic status lifecycle.

use crate::{
    attributes::{AttributeError, AttributeRules},
    numeric::{apply_basis_points, checked_add, mul_div_floor, GameInt, NumericError},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum StatusCategory {
    DamageOverTime,
    CrowdControl,
    Buff,
    Debuff,
    Neutral,
    Area,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StatusDamageType {
    Physical,
    Magical,
    True,
    Tenacity,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TickBehavior {
    None,
    DamagePotency { damage_type: StatusDamageType },
    Poison,
    ResourceLossPotency,
    Electrified,
    HealPercentMaxHp,
    CreateShield,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DecayBehavior {
    None,
    Counter { amount: GameInt },
    Potency { amount: GameInt },
    Both { potency: GameInt, counter: GameInt },
    HalveCounter,
    HalveBoth,
    CounterByPotency,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StatusModifier {
    DamageDealtPercent {
        damage_type: Option<StatusDamageType>,
        per_potency_bps: GameInt,
    },
    DamageReceivedPercent {
        hp_only: bool,
        per_potency_bps: GameInt,
    },
    EvasionPercent {
        per_potency_bps: GameInt,
    },
    PriorityPercent {
        per_potency_bps: GameInt,
    },
    ManaRegenerationPercent {
        per_potency_bps: GameInt,
    },
    HealingAndShieldPercent {
        per_potency_bps: GameInt,
    },
    TenacityIgnorePercent {
        per_potency_bps: GameInt,
    },
    ActionRestriction {
        tag: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct StatusDefinition {
    pub id: String,
    #[serde(default)]
    pub replacement_of: Option<String>,
    pub name: String,
    pub categories: BTreeSet<StatusCategory>,
    pub uses_potency: bool,
    pub uses_counter: bool,
    pub potency_min: GameInt,
    pub potency_max: Option<GameInt>,
    pub counter_min: GameInt,
    pub counter_max: Option<GameInt>,
    pub fixed_potency: Option<GameInt>,
    pub tick: TickBehavior,
    pub decay: DecayBehavior,
    #[serde(default)]
    pub modifiers: Vec<StatusModifier>,
    #[serde(default)]
    pub tags: BTreeSet<String>,
    #[serde(default)]
    pub immune_target_tags: BTreeSet<String>,
    #[serde(default)]
    pub suppresses_categories: BTreeSet<StatusCategory>,
}

impl Default for StatusDefinition {
    fn default() -> Self {
        Self {
            id: String::new(),
            replacement_of: None,
            name: String::new(),
            categories: BTreeSet::new(),
            uses_potency: true,
            uses_counter: true,
            potency_min: 0,
            potency_max: Some(100),
            counter_min: 1,
            counter_max: Some(100),
            fixed_potency: None,
            tick: TickBehavior::None,
            decay: DecayBehavior::None,
            modifiers: Vec::new(),
            tags: BTreeSet::new(),
            immune_target_tags: BTreeSet::new(),
            suppresses_categories: BTreeSet::new(),
        }
    }
}

impl StatusDefinition {
    pub fn has_category(&self, category: StatusCategory) -> bool {
        self.categories.contains(&category)
    }

    fn normalize(
        &self,
        potency: GameInt,
        counter: GameInt,
    ) -> Result<(GameInt, GameInt), StatusError> {
        let potency = if !self.uses_potency {
            0
        } else if let Some(fixed) = self.fixed_potency {
            fixed
        } else {
            clamp_checked(
                "potency",
                potency,
                self.potency_min,
                self.potency_max,
                &self.id,
            )?
        };
        let counter = if self.uses_counter {
            clamp_checked(
                "counter",
                counter,
                self.counter_min,
                self.counter_max,
                &self.id,
            )?
        } else {
            0
        };
        Ok((potency, counter))
    }
}

fn clamp_checked(
    field: &'static str,
    value: GameInt,
    minimum: GameInt,
    maximum: Option<GameInt>,
    id: &str,
) -> Result<GameInt, StatusError> {
    if value < minimum {
        return Err(StatusError::OutOfRange {
            id: id.into(),
            field,
            received: value,
            minimum,
            maximum,
        });
    }
    Ok(maximum.map_or(value, |maximum| value.min(maximum)))
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusOperation {
    pub operation: String,
    pub potency_before: GameInt,
    pub counter_before: GameInt,
    pub potency_after: GameInt,
    pub counter_after: GameInt,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActiveStatus {
    pub effect_id: String,
    pub source_id: String,
    pub potency: GameInt,
    pub counter: GameInt,
    #[serde(default)]
    pub turns_active: GameInt,
    #[serde(default)]
    pub glyph_ids: Vec<String>,
    #[serde(default)]
    pub history: Vec<StatusOperation>,
    #[serde(default)]
    pub potency_decay_skips: GameInt,
    #[serde(default)]
    pub full_decay_skips: GameInt,
}

impl ActiveStatus {
    pub fn new(
        effect_id: impl Into<String>,
        source_id: impl Into<String>,
        potency: GameInt,
        counter: GameInt,
    ) -> Self {
        Self {
            effect_id: effect_id.into(),
            source_id: source_id.into(),
            potency,
            counter,
            turns_active: 0,
            glyph_ids: Vec::new(),
            history: Vec::new(),
            potency_decay_skips: 0,
            full_decay_skips: 0,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusCollection {
    active: BTreeMap<String, ActiveStatus>,
}

impl StatusCollection {
    pub fn get(&self, effect_id: &str) -> Option<&ActiveStatus> {
        self.active.get(effect_id)
    }

    pub fn get_mut(&mut self, effect_id: &str) -> Option<&mut ActiveStatus> {
        self.active.get_mut(effect_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &ActiveStatus)> {
        self.active.iter()
    }

    pub fn remove(&mut self, effect_id: &str) -> Option<ActiveStatus> {
        self.active.remove(effect_id)
    }

    pub fn set_runtime(&mut self, status: ActiveStatus) {
        self.active.insert(status.effect_id.clone(), status);
    }

    pub fn len(&self) -> usize {
        self.active.len()
    }

    pub fn is_empty(&self) -> bool {
        self.active.is_empty()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompetitionResult {
    Applied,
    Replaced { previous_source_id: String },
    Discarded { retained_source_id: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusApplication {
    pub effect_id: String,
    pub source_id: String,
    pub input_potency: GameInt,
    pub input_counter: GameInt,
    pub counter_after_resistance: GameInt,
    pub final_potency: GameInt,
    pub final_counter: GameInt,
    pub competition: CompetitionResult,
    pub operations: Vec<StatusOperation>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusTick {
    pub effect_id: String,
    pub source_id: String,
    pub potency_before: GameInt,
    pub counter_before: GameInt,
    pub damage: GameInt,
    pub damage_type: Option<StatusDamageType>,
    pub resource_loss: GameInt,
    pub healing_percent: GameInt,
    pub shield_potency: GameInt,
    pub potency_after: GameInt,
    pub counter_after: GameInt,
    pub expired: bool,
}

#[derive(Debug, Error)]
pub enum StatusError {
    #[error("failed to read status catalog {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("invalid JSON in status catalog {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("unknown status effect {0}")]
    Unknown(String),
    #[error("duplicate status effect ID {0}")]
    Duplicate(String),
    #[error("status {id} declares replacement of missing or different target {target}")]
    InvalidReplacement { id: String, target: String },
    #[error(
        "status {id} field {field} must be >= {minimum} and <= {maximum:?}, received {received}"
    )]
    OutOfRange {
        id: String,
        field: &'static str,
        received: GameInt,
        minimum: GameInt,
        maximum: Option<GameInt>,
    },
    #[error(transparent)]
    Attribute(#[from] AttributeError),
    #[error(transparent)]
    Numeric(#[from] NumericError),
}

#[derive(Clone, Debug, Default)]
pub struct StatusRegistry {
    definitions: BTreeMap<String, StatusDefinition>,
}

impl StatusRegistry {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, StatusError> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|source| StatusError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let definitions: Vec<StatusDefinition> =
            serde_json::from_str(&text).map_err(|source| StatusError::Json {
                path: path.to_path_buf(),
                source,
            })?;
        let mut registry = Self::default();
        for definition in definitions {
            registry.register(definition)?;
        }
        Ok(registry)
    }

    pub fn register(&mut self, definition: StatusDefinition) -> Result<(), StatusError> {
        if self.definitions.contains_key(&definition.id) {
            if definition.replacement_of.as_deref() != Some(definition.id.as_str()) {
                return Err(StatusError::Duplicate(definition.id));
            }
        } else if let Some(target) = &definition.replacement_of {
            return Err(StatusError::InvalidReplacement {
                id: definition.id,
                target: target.clone(),
            });
        }
        self.definitions.insert(definition.id.clone(), definition);
        Ok(())
    }

    pub fn get(&self, effect_id: &str) -> Option<&StatusDefinition> {
        self.definitions.get(effect_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &StatusDefinition)> {
        self.definitions.iter()
    }

    pub fn base() -> Self {
        let mut registry = Self::default();
        for definition in base_definitions() {
            // The static table is covered by a duplicate-ID regression test.
            registry
                .register(definition)
                .expect("base status IDs must be unique");
        }
        registry
    }

    pub fn apply(
        &self,
        statuses: &mut StatusCollection,
        mut incoming: ActiveStatus,
        resistance: GameInt,
        attribute_rules: &AttributeRules,
    ) -> Result<StatusApplication, StatusError> {
        let definition = self
            .get(&incoming.effect_id)
            .ok_or_else(|| StatusError::Unknown(incoming.effect_id.clone()))?;
        let input_potency = incoming.potency;
        let input_counter = incoming.counter;
        let (potency, normalized_counter) =
            definition.normalize(incoming.potency, incoming.counter)?;
        let counter = if definition.uses_counter && !definition.has_category(StatusCategory::Buff) {
            attribute_rules.reduce_debuff_counter(normalized_counter, resistance)?
        } else {
            normalized_counter
        };
        incoming.potency = potency;
        incoming.counter = counter;

        // The canonical Poison replacement rule preserves half the old Counter only
        // when the incoming Potency is strictly greater.
        if incoming.effect_id == "base:poison" {
            if let Some(existing) = statuses.get("base:poison") {
                if incoming.potency > existing.potency {
                    incoming.counter = checked_add(
                        incoming.counter,
                        (existing.counter / 2).min(20),
                        "Poison replacement Counter",
                    )?
                    .min(100);
                }
            }
        }

        let competition = if !definition.uses_counter {
            CompetitionResult::Applied
        } else {
            match statuses.get(&incoming.effect_id) {
                None => {
                    statuses
                        .active
                        .insert(incoming.effect_id.clone(), incoming.clone());
                    CompetitionResult::Applied
                }
                Some(existing)
                    if (incoming.potency, incoming.counter)
                        > (existing.potency, existing.counter) =>
                {
                    let previous_source_id = existing.source_id.clone();
                    statuses
                        .active
                        .insert(incoming.effect_id.clone(), incoming.clone());
                    CompetitionResult::Replaced { previous_source_id }
                }
                Some(existing) => CompetitionResult::Discarded {
                    retained_source_id: existing.source_id.clone(),
                },
            }
        };

        Ok(StatusApplication {
            effect_id: incoming.effect_id,
            source_id: incoming.source_id,
            input_potency,
            input_counter,
            counter_after_resistance: counter,
            final_potency: potency,
            final_counter: incoming.counter,
            competition,
            operations: incoming.history,
        })
    }

    /// Applies the canonical Curse transformation once to another debuff.
    pub fn apply_curse(&self, status: &mut ActiveStatus) -> Result<StatusOperation, StatusError> {
        let definition = self
            .get(&status.effect_id)
            .ok_or_else(|| StatusError::Unknown(status.effect_id.clone()))?;
        if status.effect_id == "base:curse" || !definition.has_category(StatusCategory::Debuff) {
            return Ok(StatusOperation {
                operation: "curse_inapplicable".into(),
                potency_before: status.potency,
                counter_before: status.counter,
                potency_after: status.potency,
                counter_after: status.counter,
            });
        }
        let before_potency = status.potency;
        let before_counter = status.counter;
        status.potency = apply_basis_points(status.potency, 8_000, "Curse potency")?;
        status.counter = apply_basis_points(status.counter, 8_000, "Curse counter")?;
        Ok(StatusOperation {
            operation: "curse_minus_20_percent".into(),
            potency_before: before_potency,
            counter_before: before_counter,
            potency_after: status.potency,
            counter_after: status.counter,
        })
    }

    pub fn end_turn_ticks(
        &self,
        statuses: &mut StatusCollection,
        electrified_targets: GameInt,
    ) -> Result<Vec<StatusTick>, StatusError> {
        let ids: Vec<_> = statuses.active.keys().cloned().collect();
        let mut ticks = Vec::with_capacity(ids.len());
        for id in ids {
            let definition = self
                .get(&id)
                .ok_or_else(|| StatusError::Unknown(id.clone()))?;
            let status = statuses
                .active
                .get_mut(&id)
                .expect("status ID came from the same map");
            let potency_before = status.potency;
            let counter_before = status.counter;
            status.turns_active = checked_add(status.turns_active, 1, "status turns active")?;
            let (damage, damage_type, resource_loss, healing_percent, shield_potency) =
                match definition.tick {
                    TickBehavior::None => (0, None, 0, 0, 0),
                    TickBehavior::DamagePotency { damage_type } => {
                        (status.potency, Some(damage_type), 0, 0, 0)
                    }
                    TickBehavior::Poison => (
                        status.potency.checked_mul(status.turns_active).ok_or(
                            NumericError::Overflow {
                                operation: "Poison tick",
                                left: status.potency,
                                right: status.turns_active,
                            },
                        )?,
                        Some(StatusDamageType::Magical),
                        0,
                        0,
                        0,
                    ),
                    TickBehavior::ResourceLossPotency => (0, None, status.potency, 0, 0),
                    TickBehavior::Electrified => (
                        status.potency.checked_mul(electrified_targets).ok_or(
                            NumericError::Overflow {
                                operation: "Electrified tick",
                                left: status.potency,
                                right: electrified_targets,
                            },
                        )?,
                        Some(StatusDamageType::Magical),
                        0,
                        0,
                        0,
                    ),
                    TickBehavior::HealPercentMaxHp => (0, None, 0, status.potency, 0),
                    TickBehavior::CreateShield => (0, None, 0, 0, status.potency),
                };
            decay(status, &definition.decay);
            let expired = (definition.uses_counter && status.counter <= 0)
                || (definition.uses_potency && status.potency <= 0);
            ticks.push(StatusTick {
                effect_id: id.clone(),
                source_id: status.source_id.clone(),
                potency_before,
                counter_before,
                damage,
                damage_type,
                resource_loss,
                healing_percent,
                shield_potency,
                potency_after: status.potency,
                counter_after: status.counter,
                expired,
            });
            if expired {
                statuses.active.remove(&id);
            }
        }
        Ok(ticks)
    }
}

fn decay(status: &mut ActiveStatus, behavior: &DecayBehavior) {
    if status.full_decay_skips > 0 {
        status.full_decay_skips -= 1;
        return;
    }
    let skip_potency = status.potency_decay_skips > 0;
    if skip_potency {
        status.potency_decay_skips -= 1;
    }
    match behavior {
        DecayBehavior::None => {}
        DecayBehavior::Counter { amount } => {
            status.counter = status.counter.saturating_sub(*amount)
        }
        DecayBehavior::Potency { amount } if !skip_potency => {
            status.potency = status.potency.saturating_sub(*amount)
        }
        DecayBehavior::Potency { .. } => {}
        DecayBehavior::Both { potency, counter } => {
            if !skip_potency {
                status.potency = status.potency.saturating_sub(*potency);
            }
            status.counter = status.counter.saturating_sub(*counter);
        }
        DecayBehavior::HalveCounter => status.counter /= 2,
        DecayBehavior::HalveBoth => {
            if !skip_potency {
                status.potency /= 2;
            }
            status.counter /= 2;
        }
        DecayBehavior::CounterByPotency => {
            status.counter = status.counter.saturating_sub(status.potency)
        }
    }
}

fn categories(values: &[StatusCategory]) -> BTreeSet<StatusCategory> {
    values.iter().copied().collect()
}

fn status(
    id: &str,
    name: &str,
    category: StatusCategory,
    tick: TickBehavior,
    decay: DecayBehavior,
) -> StatusDefinition {
    StatusDefinition {
        id: id.into(),
        replacement_of: None,
        name: name.into(),
        categories: categories(&[category]),
        uses_potency: true,
        uses_counter: true,
        potency_min: 0,
        potency_max: Some(100),
        counter_min: 1,
        counter_max: Some(100),
        fixed_potency: None,
        tick,
        decay,
        modifiers: Vec::new(),
        tags: BTreeSet::new(),
        immune_target_tags: BTreeSet::new(),
        suppresses_categories: BTreeSet::new(),
    }
}

fn base_definitions() -> Vec<StatusDefinition> {
    use DecayBehavior as Decay;
    use StatusCategory as Category;
    use StatusDamageType as Damage;
    use StatusModifier as Modifier;
    use TickBehavior as Tick;

    let mut burn = status(
        "base:burn",
        "Burn",
        Category::DamageOverTime,
        Tick::DamagePotency {
            damage_type: Damage::Magical,
        },
        Decay::Both {
            potency: 1,
            counter: 1,
        },
    );
    burn.tags.insert("tick_capable".into());
    let mut poison = status(
        "base:poison",
        "Poison",
        Category::DamageOverTime,
        Tick::Poison,
        Decay::HalveCounter,
    );
    poison.tags.insert("tick_capable".into());
    let sinking = status(
        "base:sinking",
        "Sinking",
        Category::DamageOverTime,
        Tick::ResourceLossPotency,
        Decay::Counter { amount: 1 },
    );
    let electrified = status(
        "base:electrified",
        "Electrified",
        Category::DamageOverTime,
        Tick::Electrified,
        Decay::Counter { amount: 1 },
    );
    let bleed = status(
        "base:bleed",
        "Bleed",
        Category::DamageOverTime,
        Tick::None,
        Decay::HalveBoth,
    );

    let crowd_control = |id: &str, name: &str, restriction: &str| {
        let mut definition = status(
            id,
            name,
            Category::CrowdControl,
            Tick::None,
            Decay::Counter { amount: 1 },
        );
        definition.fixed_potency = Some(1);
        definition.potency_min = 1;
        definition.potency_max = Some(1);
        definition.counter_max = Some(5);
        definition.modifiers.push(Modifier::ActionRestriction {
            tag: restriction.into(),
        });
        definition
    };

    let mut fear = crowd_control("base:fear", "Fear", "cannot_attack_source");
    fear.modifiers.push(Modifier::EvasionPercent {
        per_potency_bps: -100,
    });
    let mut taunt = crowd_control("base:taunt", "Taunt", "attack_only_source");
    taunt.modifiers.push(Modifier::DamageDealtPercent {
        damage_type: None,
        per_potency_bps: -100,
    });
    let mut charm = crowd_control("base:charm", "Charm", "cannot_attack_source");
    charm.modifiers.push(Modifier::DamageReceivedPercent {
        hp_only: false,
        per_potency_bps: 100,
    });

    let mut protect = status(
        "base:protect",
        "Protect",
        Category::Buff,
        Tick::None,
        Decay::Counter { amount: 1 },
    );
    protect.modifiers.push(Modifier::DamageReceivedPercent {
        hp_only: false,
        per_potency_bps: -100,
    });
    let mut haste = status(
        "base:haste",
        "Haste",
        Category::Buff,
        Tick::None,
        Decay::Counter { amount: 1 },
    );
    haste.modifiers.push(Modifier::EvasionPercent {
        per_potency_bps: 100,
    });
    haste.modifiers.push(Modifier::PriorityPercent {
        per_potency_bps: 100,
    });
    let mut rage = status(
        "base:rage",
        "Rage",
        Category::Buff,
        Tick::None,
        Decay::Counter { amount: 1 },
    );
    rage.modifiers.push(Modifier::DamageDealtPercent {
        damage_type: Some(Damage::Physical),
        per_potency_bps: 100,
    });
    let mut overdrive = status(
        "base:overdrive",
        "Overdrive",
        Category::Buff,
        Tick::None,
        Decay::Counter { amount: 1 },
    );
    overdrive.modifiers.push(Modifier::ManaRegenerationPercent {
        per_potency_bps: 100,
    });
    let mut sage = status(
        "base:sage",
        "Sage",
        Category::Buff,
        Tick::None,
        Decay::Counter { amount: 1 },
    );
    sage.modifiers.push(Modifier::DamageDealtPercent {
        damage_type: Some(Damage::Magical),
        per_potency_bps: 100,
    });
    let mut heal = status(
        "base:heal",
        "Heal",
        Category::Buff,
        Tick::HealPercentMaxHp,
        Decay::None,
    );
    heal.uses_counter = false;
    heal.counter_min = 0;
    heal.counter_max = Some(0);
    let mut shield = status(
        "base:shield",
        "Shield",
        Category::Buff,
        Tick::CreateShield,
        Decay::None,
    );
    shield.uses_counter = false;
    shield.counter_min = 0;
    shield.counter_max = Some(0);
    shield.potency_max = None;
    let mut blessing = status(
        "base:blessing",
        "Blessing",
        Category::Buff,
        Tick::None,
        Decay::Counter { amount: 1 },
    );
    blessing.modifiers.push(Modifier::HealingAndShieldPercent {
        per_potency_bps: 100,
    });

    let mut fragile = status(
        "base:fragile",
        "Fragile",
        Category::Debuff,
        Tick::None,
        Decay::CounterByPotency,
    );
    fragile.modifiers.push(Modifier::DamageReceivedPercent {
        hp_only: true,
        per_potency_bps: 100,
    });
    let mut tremor = status(
        "base:tremor",
        "Tremor",
        Category::Debuff,
        Tick::None,
        Decay::Counter { amount: 1 },
    );
    tremor.modifiers.push(Modifier::DamageReceivedPercent {
        hp_only: false,
        per_potency_bps: 100,
    });
    let mut rupture = status(
        "base:rupture",
        "Rupture",
        Category::Debuff,
        Tick::None,
        Decay::Potency { amount: 1 },
    );
    rupture.modifiers.push(Modifier::TenacityIgnorePercent {
        per_potency_bps: 100,
    });
    let mut exhaust = status(
        "base:exhaust",
        "Exhaust",
        Category::Debuff,
        Tick::None,
        Decay::Counter { amount: 1 },
    );
    exhaust.modifiers.push(Modifier::EvasionPercent {
        per_potency_bps: -100,
    });
    exhaust.modifiers.push(Modifier::PriorityPercent {
        per_potency_bps: -100,
    });
    let mut attack_down = status(
        "base:attack-down",
        "Attack Down",
        Category::Debuff,
        Tick::None,
        Decay::Counter { amount: 1 },
    );
    attack_down.modifiers.push(Modifier::DamageDealtPercent {
        damage_type: None,
        per_potency_bps: -100,
    });
    let curse = status(
        "base:curse",
        "Curse",
        Category::Debuff,
        Tick::None,
        Decay::Counter { amount: 1 },
    );

    let mut dark_flame = status(
        "base:dark-flame",
        "Dark Flame",
        Category::DamageOverTime,
        Tick::DamagePotency {
            damage_type: Damage::True,
        },
        Decay::Counter { amount: 1 },
    );
    dark_flame.categories.insert(Category::Debuff);
    dark_flame.tags.extend(["darkness".into(), "fire".into()]);
    dark_flame.immune_target_tags.insert("race:demon".into());
    dark_flame
        .suppresses_categories
        .insert(Category::DamageOverTime);

    let mut broken_heart = status(
        "base:broken-heart",
        "Broken Heart",
        Category::Debuff,
        Tick::None,
        Decay::Counter { amount: 1 },
    );
    broken_heart.potency_min = 15;
    broken_heart.potency_max = Some(15);
    broken_heart.fixed_potency = Some(15);
    broken_heart.modifiers.push(Modifier::DamageDealtPercent {
        damage_type: None,
        per_potency_bps: -100,
    });

    let persistent = |id: &str,
                      name: &str,
                      categories_list: &[Category],
                      potency_max: GameInt,
                      counter_max: GameInt| {
        StatusDefinition {
            id: id.into(),
            replacement_of: None,
            name: name.into(),
            categories: categories(categories_list),
            uses_potency: true,
            uses_counter: true,
            potency_min: 0,
            potency_max: Some(potency_max),
            counter_min: 1,
            counter_max: Some(counter_max),
            fixed_potency: None,
            tick: Tick::None,
            decay: Decay::Counter { amount: 1 },
            modifiers: Vec::new(),
            tags: BTreeSet::new(),
            immune_target_tags: BTreeSet::new(),
            suppresses_categories: BTreeSet::new(),
        }
    };

    vec![
        dark_flame,
        broken_heart,
        burn,
        poison,
        sinking,
        electrified,
        bleed,
        crowd_control("base:stun", "Stun", "cannot_attack_or_evade"),
        crowd_control("base:binding", "Binding", "cannot_move_or_use_physical"),
        crowd_control("base:silent", "Silent", "cannot_cast_magic"),
        fear,
        taunt,
        charm,
        protect,
        haste,
        rage,
        overdrive,
        sage,
        heal,
        shield,
        blessing,
        fragile,
        tremor,
        rupture,
        exhaust,
        attack_down,
        curse,
        persistent("base:reveal-lock", "Reveal Lock", &[Category::Debuff], 1, 3),
        persistent(
            "base:shadow-debt",
            "Shadow Debt",
            &[Category::Debuff],
            20,
            10,
        ),
        persistent("base:erosion", "Erosion", &[Category::Debuff], 50, 3),
        persistent("base:resonance", "Resonance", &[Category::Buff], 3, 3),
        persistent("base:cryostasis", "Cryostasis", &[Category::Neutral], 1, 2),
        persistent(
            "base:molten-ground",
            "Molten Ground",
            &[Category::Debuff, Category::Area],
            50,
            3,
        ),
        persistent(
            "base:electrified-network",
            "Electrified Network",
            &[Category::Debuff],
            6,
            3,
        ),
        persistent("base:plant-growth", "Plant Growth", &[Category::Buff], 5, 5),
    ]
}

pub fn status_budget(
    effect_id: &str,
    potency: GameInt,
    counter: GameInt,
) -> Result<GameInt, NumericError> {
    let (potency_rate, counter_rate) = match effect_id {
        "base:burn" => (4, 5),
        "base:poison" => (1, 1),
        "base:bleed" | "base:electrified" => (2, 1),
        "base:tremor" => (1, 2),
        _ => (1, 1),
    };
    let potency_value = potency
        .checked_mul(potency_rate)
        .ok_or(NumericError::Overflow {
            operation: "status budget potency",
            left: potency,
            right: potency_rate,
        })?;
    let counter_value = counter
        .checked_mul(counter_rate)
        .ok_or(NumericError::Overflow {
            operation: "status budget counter",
            left: counter,
            right: counter_rate,
        })?;
    potency_value
        .checked_add(counter_value)
        .ok_or(NumericError::Overflow {
            operation: "status budget total",
            left: potency_value,
            right: counter_value,
        })
}

pub fn heal_amount(max_hp: GameInt, potency: GameInt) -> Result<GameInt, NumericError> {
    mul_div_floor(max_hp, potency, 100, "Heal")
}

pub fn shield_amount(potency: GameInt) -> Result<GameInt, NumericError> {
    potency.checked_mul(10).ok_or(NumericError::Overflow {
        operation: "Shield HP",
        left: potency,
        right: 10,
    })
}
