use crate::{
    attributes::{Attribute, AttributeError, AttributeRules, AttributeSet},
    numeric::{checked_add, GameInt, NumericError, Ratio},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScalingGrade {
    D,
    C,
    B,
    A,
    S,
    SS,
    SSS,
}

impl ScalingGrade {
    pub const fn ratio(self) -> Ratio {
        match self {
            Self::D => Ratio::new(1, 4),
            Self::C => Ratio::new(1, 2),
            Self::B => Ratio::new(3, 4),
            Self::A => Ratio::new(1, 1),
            Self::S => Ratio::new(3, 2),
            Self::SS => Ratio::new(2, 1),
            Self::SSS => Ratio::new(5, 2),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScalingSource {
    pub source_id: String,
    pub attribute: Attribute,
    pub grade: ScalingGrade,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScalingContribution {
    pub source_id: String,
    pub attribute: Attribute,
    pub power: GameInt,
    pub grade: ScalingGrade,
    pub multiplier: Ratio,
    pub contribution: GameInt,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScalingResult {
    pub flat_base_damage: GameInt,
    pub contributions: Vec<ScalingContribution>,
    pub total: GameInt,
}

#[derive(Debug, thiserror::Error)]
pub enum ScalingError {
    #[error(transparent)]
    Attribute(#[from] AttributeError),
    #[error(transparent)]
    Numeric(#[from] NumericError),
}

pub fn calculate_scaling(
    flat_base_damage: GameInt,
    attributes: &AttributeSet,
    rules: &AttributeRules,
    sources: &[ScalingSource],
) -> Result<ScalingResult, ScalingError> {
    let mut total = flat_base_damage;
    let mut contributions = Vec::with_capacity(sources.len());
    for source in sources {
        let power = rules.power(attributes, source.attribute)?;
        let multiplier = source.grade.ratio();
        let contribution = multiplier.apply(power, "Scaling contribution")?;
        total = checked_add(total, contribution, "pre-mitigation damage")?;
        contributions.push(ScalingContribution {
            source_id: source.source_id.clone(),
            attribute: source.attribute,
            power,
            grade: source.grade,
            multiplier,
            contribution,
        });
    }
    Ok(ScalingResult {
        flat_base_damage,
        contributions,
        total,
    })
}
