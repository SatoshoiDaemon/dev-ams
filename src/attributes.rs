use crate::numeric::{checked_add, mul_div_floor, GameInt, NumericError};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt, str::FromStr};
use thiserror::Error;

pub const ATTRIBUTE_DATA_MAX: GameInt = 1_000_000;
pub const ATTRIBUTE_ENHANCEMENT_SOURCE_LIMIT: usize = 4;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Attribute {
    Vigor,
    Resistance,
    Strength,
    Dexterity,
    Precision,
    Intelligence,
    Mana,
}

impl Attribute {
    pub const ALL: [Self; 7] = [
        Self::Vigor,
        Self::Resistance,
        Self::Strength,
        Self::Dexterity,
        Self::Precision,
        Self::Intelligence,
        Self::Mana,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Vigor => "vigor",
            Self::Resistance => "resistance",
            Self::Strength => "strength",
            Self::Dexterity => "dexterity",
            Self::Precision => "precision",
            Self::Intelligence => "intelligence",
            Self::Mana => "mana",
        }
    }

    pub const fn is_offensive(self) -> bool {
        matches!(
            self,
            Self::Strength | Self::Dexterity | Self::Precision | Self::Intelligence
        )
    }
}

impl fmt::Display for Attribute {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.id())
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("unknown attribute ID {received:?}")]
pub struct AttributeParseError {
    pub received: String,
}

impl FromStr for Attribute {
    type Err = AttributeParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "vigor" => Ok(Self::Vigor),
            "resistance" => Ok(Self::Resistance),
            "strength" => Ok(Self::Strength),
            "dexterity" => Ok(Self::Dexterity),
            "precision" => Ok(Self::Precision),
            "intelligence" | "spirit" => Ok(Self::Intelligence),
            _ => Err(AttributeParseError {
                received: value.into(),
            }),
        }
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum AttributeError {
    #[error("attribute {attribute} must be between 0 and {maximum}, received {received}")]
    OutOfRange {
        attribute: Attribute,
        received: GameInt,
        maximum: GameInt,
    },
    #[error(transparent)]
    Numeric(#[from] NumericError),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct AttributeSet {
    values: BTreeMap<Attribute, GameInt>,
}

impl Default for AttributeSet {
    fn default() -> Self {
        Self {
            values: Attribute::ALL
                .into_iter()
                .map(|attribute| (attribute, 0))
                .collect(),
        }
    }
}

impl AttributeSet {
    pub fn get(&self, attribute: Attribute) -> GameInt {
        self.values.get(&attribute).copied().unwrap_or(0)
    }

    pub fn set(&mut self, attribute: Attribute, value: GameInt) -> Result<(), AttributeError> {
        if !(0..=ATTRIBUTE_DATA_MAX).contains(&value) {
            return Err(AttributeError::OutOfRange {
                attribute,
                received: value,
                maximum: ATTRIBUTE_DATA_MAX,
            });
        }
        self.values.insert(attribute, value);
        Ok(())
    }

    pub fn validate(&self) -> Result<(), AttributeError> {
        for attribute in Attribute::ALL {
            let value = self.get(attribute);
            if !(0..=ATTRIBUTE_DATA_MAX).contains(&value) {
                return Err(AttributeError::OutOfRange {
                    attribute,
                    received: value,
                    maximum: ATTRIBUTE_DATA_MAX,
                });
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttributeRules {
    pub base_hp: GameInt,
    pub base_tenacity: GameInt,
    pub base_mana: GameInt,
    pub hp_per_vigor: GameInt,
    pub tenacity_per_resistance: GameInt,
    pub mana_per_mana: GameInt,
    pub power_per_attribute: GameInt,
    pub priority_base: GameInt,
    pub priority_per_dexterity: GameInt,
}

impl Default for AttributeRules {
    fn default() -> Self {
        Self {
            base_hp: 100,
            base_tenacity: 50,
            base_mana: 30,
            hp_per_vigor: 10,
            tenacity_per_resistance: 10,
            mana_per_mana: 10,
            power_per_attribute: 7,
            priority_base: 100,
            priority_per_dexterity: 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DerivedAttributes {
    pub max_hp: GameInt,
    pub max_tenacity: GameInt,
    pub max_mana: GameInt,
    pub priority: GameInt,
}

impl AttributeRules {
    pub fn derive(&self, attributes: &AttributeSet) -> Result<DerivedAttributes, AttributeError> {
        attributes.validate()?;
        Ok(DerivedAttributes {
            max_hp: linear(
                self.base_hp,
                attributes.get(Attribute::Vigor),
                self.hp_per_vigor,
                "max HP",
            )?,
            max_tenacity: linear(
                self.base_tenacity,
                attributes.get(Attribute::Resistance),
                self.tenacity_per_resistance,
                "max Tenacity",
            )?,
            max_mana: linear(
                self.base_mana,
                attributes.get(Attribute::Mana),
                self.mana_per_mana,
                "max Mana",
            )?,
            priority: linear(
                self.priority_base,
                attributes.get(Attribute::Dexterity),
                self.priority_per_dexterity,
                "Priority",
            )?,
        })
    }

    pub fn power(
        &self,
        attributes: &AttributeSet,
        attribute: Attribute,
    ) -> Result<GameInt, AttributeError> {
        if !attribute.is_offensive() {
            return Ok(0);
        }
        mul_div_floor(
            attributes.get(attribute),
            self.power_per_attribute,
            1,
            "attribute Power",
        )
        .map_err(Into::into)
    }

    pub fn reduce_debuff_counter(
        &self,
        base_counter: GameInt,
        resistance: GameInt,
    ) -> Result<GameInt, AttributeError> {
        if base_counter <= 0 {
            return Ok(0);
        }
        let denominator = checked_add(
            1_000,
            resistance.checked_mul(10).ok_or(NumericError::Overflow {
                operation: "Resistance counter denominator",
                left: resistance,
                right: 10,
            })?,
            "Resistance counter denominator",
        )?;
        Ok(mul_div_floor(
            base_counter,
            1_000,
            denominator,
            "Resistance debuff reduction",
        )?
        .max(1))
    }
}

fn linear(
    base: GameInt,
    points: GameInt,
    rate: GameInt,
    operation: &'static str,
) -> Result<GameInt, NumericError> {
    let contribution = mul_div_floor(points, rate, 1, operation)?;
    checked_add(base, contribution, operation)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttributeEnhancement {
    pub source_id: String,
    pub amount: GameInt,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttributeEnhancements {
    sources: BTreeMap<Attribute, BTreeMap<String, GameInt>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnhancementResult {
    Applied,
    Replaced,
    IgnoredSourceLimit { retained_sources: Vec<String> },
}

impl AttributeEnhancements {
    pub fn add(
        &mut self,
        attribute: Attribute,
        enhancement: AttributeEnhancement,
    ) -> EnhancementResult {
        let sources = self.sources.entry(attribute).or_default();
        if let std::collections::btree_map::Entry::Occupied(mut entry) =
            sources.entry(enhancement.source_id.clone())
        {
            entry.insert(enhancement.amount);
            return EnhancementResult::Replaced;
        }
        if sources.len() >= ATTRIBUTE_ENHANCEMENT_SOURCE_LIMIT {
            return EnhancementResult::IgnoredSourceLimit {
                retained_sources: sources.keys().cloned().collect(),
            };
        }
        sources.insert(enhancement.source_id, enhancement.amount);
        EnhancementResult::Applied
    }

    pub fn total(&self, attribute: Attribute) -> Result<GameInt, NumericError> {
        self.sources
            .get(&attribute)
            .into_iter()
            .flat_map(|sources| sources.values())
            .try_fold(0, |sum, value| {
                checked_add(sum, *value, "attribute enhancement total")
            })
    }

    pub fn source_ids(&self, attribute: Attribute) -> Vec<&str> {
        self.sources
            .get(&attribute)
            .into_iter()
            .flat_map(|sources| sources.keys().map(String::as_str))
            .collect()
    }
}
