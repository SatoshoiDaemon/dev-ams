//! Spell construction and declarative Glyph catalog support.

use crate::{
    effects::{status_budget, ActiveStatus, StatusOperation, StatusRegistry},
    numeric::{apply_basis_points, checked_add, mul_div_floor, GameInt, NumericError},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;

pub const MAX_SPELL_TARGETS: usize = 8;
pub const MAX_SECONDARY_EFFECTS: usize = 3;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Element {
    Fire,
    Water,
    Earth,
    Air,
    Light,
    Darkness,
    Poison,
    Ice,
    Sound,
    Metal,
    Plant,
    Lava,
    Lightning,
}

impl Element {
    pub const ALL: [Self; 13] = [
        Self::Fire,
        Self::Water,
        Self::Earth,
        Self::Air,
        Self::Light,
        Self::Darkness,
        Self::Poison,
        Self::Ice,
        Self::Sound,
        Self::Metal,
        Self::Plant,
        Self::Lava,
        Self::Lightning,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Fire => "Fogo",
            Self::Water => "Água",
            Self::Earth => "Terra",
            Self::Air => "Ar",
            Self::Light => "Luz",
            Self::Darkness => "Trevas",
            Self::Poison => "Veneno",
            Self::Ice => "Gelo",
            Self::Sound => "Som",
            Self::Metal => "Metal",
            Self::Plant => "Plantas",
            Self::Lava => "Lava",
            Self::Lightning => "Raio",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlyphCard {
    pub id: String,
    pub name: String,
    pub weight: GameInt,
    #[serde(default)]
    pub element: Option<Element>,
    #[serde(default)]
    pub requirements: BTreeSet<String>,
    pub rule: String,
}

#[derive(Clone, Debug, Default)]
pub struct GlyphRegistry {
    cards: BTreeMap<String, GlyphCard>,
}

#[derive(Debug, Error)]
pub enum GlyphError {
    #[error("failed to read Glyph catalog {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("invalid JSON in Glyph catalog {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("duplicate Glyph ID {0}")]
    Duplicate(String),
    #[error("unknown Glyph ID {0}")]
    Unknown(String),
    #[error("Glyph {glyph_id} requires element {element:?} in the same branch")]
    MissingElement { glyph_id: String, element: Element },
    #[error("Glyph {glyph_id} requires capability {requirement}")]
    MissingRequirement {
        glyph_id: String,
        requirement: String,
    },
    #[error("spell selects {received} targets; maximum is {maximum}")]
    TooManyTargets { received: usize, maximum: usize },
    #[error("spell emits {received} secondary effects; maximum is {maximum}")]
    TooManySecondaryEffects { received: usize, maximum: usize },
    #[error("spell Weight {weight} exceeds capacity {capacity}")]
    OverCapacity { weight: GameInt, capacity: GameInt },
    #[error("invalid non-negative spell field {field}: {received}")]
    Negative {
        field: &'static str,
        received: GameInt,
    },
    #[error("Glyph {glyph_id} already activated for event {event_id} and target {target_id}")]
    RepeatedActivation {
        glyph_id: String,
        event_id: u64,
        target_id: String,
    },
    #[error("status {effect_id} does not support Glyph {glyph_id}")]
    InvalidStatusGlyph { effect_id: String, glyph_id: String },
    #[error("Glyph {glyph_id} needs at least {required} Counter, received {received}")]
    InsufficientCounter {
        glyph_id: String,
        required: GameInt,
        received: GameInt,
    },
    #[error(transparent)]
    Numeric(#[from] NumericError),
}

impl GlyphRegistry {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, GlyphError> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|source| GlyphError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let cards: Vec<GlyphCard> =
            serde_json::from_str(&text).map_err(|source| GlyphError::Json {
                path: path.to_path_buf(),
                source,
            })?;
        let mut registry = Self::default();
        for card in cards {
            registry.register(card)?;
        }
        Ok(registry)
    }

    pub fn register(&mut self, card: GlyphCard) -> Result<(), GlyphError> {
        if self.cards.contains_key(&card.id) {
            return Err(GlyphError::Duplicate(card.id));
        }
        self.cards.insert(card.id.clone(), card);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&GlyphCard> {
        self.cards.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &GlyphCard)> {
        self.cards.iter()
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Source,
    Type,
    Form,
    Effect,
    Modifier,
    Sacrifice,
}

impl NodeKind {
    pub const fn weight(self) -> GameInt {
        match self {
            Self::Source | Self::Sacrifice => 0,
            Self::Type | Self::Modifier => 1,
            Self::Form | Self::Effect => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Sacrifice {
    CastTime,
    Cooldown,
    Cost,
    HpCost,
    SelfStatus,
    Condition,
    Tribute,
    MagicSlots,
}

impl Sacrifice {
    pub const fn credit(self) -> GameInt {
        match self {
            Self::CastTime | Self::Cooldown | Self::Cost | Self::SelfStatus | Self::Condition => 2,
            Self::HpCost | Self::Tribute => 3,
            Self::MagicSlots => 4,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpellBlueprint {
    pub id: String,
    pub nodes: Vec<NodeKind>,
    #[serde(default)]
    pub parameters: Vec<GameInt>,
    #[serde(default)]
    pub glyph_ids: Vec<String>,
    #[serde(default)]
    pub sacrifices: Vec<Sacrifice>,
    #[serde(default)]
    pub elements: BTreeSet<Element>,
    #[serde(default)]
    pub capabilities: BTreeSet<String>,
    pub target_count: usize,
    pub secondary_effect_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpellEvaluation {
    pub spell_id: String,
    pub node_weight: GameInt,
    pub parameter_weight: GameInt,
    pub glyph_weight: GameInt,
    pub sacrifice_credit: GameInt,
    pub final_weight: GameInt,
    pub capacity: GameInt,
    pub mana_cost: GameInt,
    pub valid: bool,
    pub rejection_reason: Option<String>,
}

pub fn parameter_weight(value: GameInt) -> Result<GameInt, GlyphError> {
    if value <= 0 {
        return Err(GlyphError::Negative {
            field: "parameter",
            received: value,
        });
    }
    Ok(1 + (value - 1) / 10)
}

pub fn spell_capacity(
    intelligence: GameInt,
    catalyst_tier: GameInt,
    grimoire_rank: GameInt,
) -> Result<GameInt, GlyphError> {
    for (field, value) in [
        ("intelligence", intelligence),
        ("catalyst_tier", catalyst_tier),
        ("grimoire_rank", grimoire_rank),
    ] {
        if value < 0 {
            return Err(GlyphError::Negative {
                field,
                received: value,
            });
        }
    }
    let intelligence_capacity = intelligence.checked_mul(2).ok_or(NumericError::Overflow {
        operation: "spell Intelligence capacity",
        left: intelligence,
        right: 2,
    })?;
    let catalyst_capacity = catalyst_tier.checked_mul(5).ok_or(NumericError::Overflow {
        operation: "spell Catalyst capacity",
        left: catalyst_tier,
        right: 5,
    })?;
    let grimoire_capacity = grimoire_rank.checked_mul(3).ok_or(NumericError::Overflow {
        operation: "spell Grimoire capacity",
        left: grimoire_rank,
        right: 3,
    })?;
    Ok(checked_add(
        checked_add(
            checked_add(20, intelligence_capacity, "spell capacity")?,
            catalyst_capacity,
            "spell capacity",
        )?,
        grimoire_capacity,
        "spell capacity",
    )?)
}

pub fn evaluate_spell(
    blueprint: &SpellBlueprint,
    glyphs: &GlyphRegistry,
    intelligence: GameInt,
    catalyst_tier: GameInt,
    grimoire_rank: GameInt,
) -> Result<SpellEvaluation, GlyphError> {
    if blueprint.target_count > MAX_SPELL_TARGETS {
        return Err(GlyphError::TooManyTargets {
            received: blueprint.target_count,
            maximum: MAX_SPELL_TARGETS,
        });
    }
    if blueprint.secondary_effect_count > MAX_SECONDARY_EFFECTS {
        return Err(GlyphError::TooManySecondaryEffects {
            received: blueprint.secondary_effect_count,
            maximum: MAX_SECONDARY_EFFECTS,
        });
    }

    let node_weight = blueprint.nodes.iter().try_fold(0, |sum, node| {
        checked_add(sum, node.weight(), "spell node Weight")
    })?;
    let parameter_weight = blueprint.parameters.iter().try_fold(0, |sum, value| {
        checked_add(sum, parameter_weight(*value)?, "spell parameter Weight")
            .map_err(GlyphError::from)
    })?;
    let mut glyph_weight = 0;
    for glyph_id in &blueprint.glyph_ids {
        let card = glyphs
            .get(glyph_id)
            .ok_or_else(|| GlyphError::Unknown(glyph_id.clone()))?;
        if let Some(element) = card.element {
            if !blueprint.elements.contains(&element) {
                return Err(GlyphError::MissingElement {
                    glyph_id: card.id.clone(),
                    element,
                });
            }
        }
        if let Some(requirement) = card
            .requirements
            .iter()
            .find(|requirement| !blueprint.capabilities.contains(*requirement))
        {
            return Err(GlyphError::MissingRequirement {
                glyph_id: card.id.clone(),
                requirement: requirement.clone(),
            });
        }
        glyph_weight = checked_add(glyph_weight, card.weight, "spell Glyph Weight")?;
    }
    let sacrifice_credit = blueprint.sacrifices.iter().try_fold(0, |sum, sacrifice| {
        checked_add(sum, sacrifice.credit(), "spell Sacrifice credit")
    })?;
    let gross = checked_add(
        checked_add(node_weight, parameter_weight, "spell Weight")?,
        glyph_weight,
        "spell Weight",
    )?;
    let final_weight = gross.saturating_sub(sacrifice_credit).max(1);
    let capacity = spell_capacity(intelligence, catalyst_tier, grimoire_rank)?;
    let valid = final_weight <= capacity;
    Ok(SpellEvaluation {
        spell_id: blueprint.id.clone(),
        node_weight,
        parameter_weight,
        glyph_weight,
        sacrifice_credit,
        final_weight,
        capacity,
        mana_cost: 5 + final_weight,
        valid,
        rejection_reason: (!valid)
            .then(|| format!("spell Weight {final_weight} exceeds capacity {capacity}")),
    })
}

#[derive(Clone, Debug, Default)]
pub struct GlyphActivationGuard {
    activations: BTreeSet<(u64, String, String)>,
}

impl GlyphActivationGuard {
    pub fn activate(
        &mut self,
        event_id: u64,
        glyph_id: &str,
        target_id: &str,
    ) -> Result<(), GlyphError> {
        let key = (event_id, glyph_id.to_owned(), target_id.to_owned());
        if !self.activations.insert(key) {
            return Err(GlyphError::RepeatedActivation {
                glyph_id: glyph_id.into(),
                event_id,
                target_id: target_id.into(),
            });
        }
        Ok(())
    }

    pub fn clear_before(&mut self, event_id: u64) {
        self.activations
            .retain(|(stored_event, _, _)| *stored_event >= event_id);
    }
}

pub fn amplify_status(
    status: &mut ActiveStatus,
    bonus_bps: GameInt,
    glyph_id: &str,
) -> Result<StatusOperation, GlyphError> {
    amplify_status_repeated(status, &[(glyph_id.to_owned(), bonus_bps)])
}

/// Combines repeated amplification additively, then floors once.
pub fn amplify_status_repeated(
    status: &mut ActiveStatus,
    operations: &[(String, GameInt)],
) -> Result<StatusOperation, GlyphError> {
    let potency_before = status.potency;
    let counter_before = status.counter;
    let total_bps = operations.iter().try_fold(0, |total, (_, bonus)| {
        checked_add(total, *bonus, "combined Glyph amplification")
    })?;
    status.potency = apply_basis_points(
        status.potency,
        checked_add(10_000, total_bps, "Glyph amplification factor")?,
        "Glyph status Potency",
    )?
    .min(100);
    status.counter = apply_basis_points(
        status.counter,
        checked_add(10_000, total_bps, "Glyph amplification factor")?,
        "Glyph status Counter",
    )?
    .min(100);
    status
        .glyph_ids
        .extend(operations.iter().map(|(glyph_id, _)| glyph_id.clone()));
    let operation = StatusOperation {
        operation: format!("combined_amplification_{total_bps}_bps"),
        potency_before,
        counter_before,
        potency_after: status.potency,
        counter_after: status.counter,
    };
    status.history.push(operation.clone());
    Ok(operation)
}

pub fn condensed(status: &mut ActiveStatus) -> Result<StatusOperation, GlyphError> {
    let potency_before = status.potency;
    let counter_before = status.counter;
    if status.effect_id == "base:burn" {
        status.counter = mul_div_floor(counter_before, 2, 5, "Condensed Burn Counter")?.max(1);
        // Burn budget is 4 per Potency and 5 per Counter. Convert the removed
        // Counter budget exactly, matching the canonical 20/20 -> 35/8 vector.
        let removed_counter = counter_before - status.counter;
        let added_potency = mul_div_floor(removed_counter, 5, 4, "Condensed Burn Potency")?;
        status.potency =
            checked_add(status.potency, added_potency, "Condensed Burn Potency")?.min(100);
    } else {
        status.potency =
            checked_add(status.potency, status.counter / 2, "Condensed Potency")?.min(100);
        status.counter = ((status.counter + 1) / 2).max(1);
    }
    ensure_budget_not_increased(status, potency_before, counter_before, "base:condensed")?;
    status.glyph_ids.push("base:condensed".into());
    let operation = StatusOperation {
        operation: "base:condensed".into(),
        potency_before,
        counter_before,
        potency_after: status.potency,
        counter_after: status.counter,
    };
    status.history.push(operation.clone());
    Ok(operation)
}

pub fn lingering(status: &mut ActiveStatus) -> Result<StatusOperation, GlyphError> {
    let potency_before = status.potency;
    let counter_before = status.counter;
    if status.effect_id == "base:burn" {
        status.counter = checked_add(
            status.counter,
            mul_div_floor(status.potency, 3, 5, "Lingering Burn Counter")?,
            "Lingering Burn Counter",
        )?
        .min(100);
        status.potency = mul_div_floor(status.potency, 2, 5, "Lingering Burn Potency")?.max(1);
    } else {
        status.counter =
            checked_add(status.counter, status.potency / 2, "Lingering Counter")?.min(100);
        status.potency = ((status.potency + 1) / 2).max(1);
    }
    status.glyph_ids.push("base:lingering".into());
    let operation = StatusOperation {
        operation: "base:lingering".into(),
        potency_before,
        counter_before,
        potency_after: status.potency,
        counter_after: status.counter,
    };
    status.history.push(operation.clone());
    Ok(operation)
}

pub fn stable(status: &mut ActiveStatus) {
    status.potency_decay_skips = 2;
    status.glyph_ids.push("base:stable".into());
}

pub fn permafrost(status: &mut ActiveStatus) {
    status.full_decay_skips = 1;
    status.glyph_ids.push("base:permafrost".into());
}

pub fn accelerated(status: &mut ActiveStatus, registry: &StatusRegistry) -> Result<(), GlyphError> {
    if status.counter < 2 {
        return Err(GlyphError::InsufficientCounter {
            glyph_id: "base:accelerated".into(),
            required: 2,
            received: status.counter,
        });
    }
    let definition =
        registry
            .get(&status.effect_id)
            .ok_or_else(|| GlyphError::InvalidStatusGlyph {
                effect_id: status.effect_id.clone(),
                glyph_id: "base:accelerated".into(),
            })?;
    if !definition.tags.contains("tick_capable") {
        return Err(GlyphError::InvalidStatusGlyph {
            effect_id: status.effect_id.clone(),
            glyph_id: "base:accelerated".into(),
        });
    }
    status.counter -= 2;
    status.glyph_ids.push("base:accelerated".into());
    Ok(())
}

fn ensure_budget_not_increased(
    status: &ActiveStatus,
    potency_before: GameInt,
    counter_before: GameInt,
    glyph_id: &str,
) -> Result<(), GlyphError> {
    let before = status_budget(&status.effect_id, potency_before, counter_before)?;
    let after = status_budget(&status.effect_id, status.potency, status.counter)?;
    if after > before {
        return Err(GlyphError::InvalidStatusGlyph {
            effect_id: status.effect_id.clone(),
            glyph_id: glyph_id.into(),
        });
    }
    Ok(())
}
