//! Deterministic gameplay arithmetic shared by every system.
//!
//! Persistent gameplay values never use floating point. Intermediate products use
//! `i128`, then are checked before returning to the canonical `i64` domain.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type GameInt = i64;
pub const BASIS_POINTS: GameInt = 10_000;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ratio {
    pub numerator: GameInt,
    pub denominator: GameInt,
}

impl Ratio {
    pub const fn new(numerator: GameInt, denominator: GameInt) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    pub fn apply(self, value: GameInt, operation: &'static str) -> Result<GameInt, NumericError> {
        mul_div_floor(value, self.numerator, self.denominator, operation)
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum NumericError {
    #[error("{operation}: denominator must be positive, received {denominator}")]
    InvalidDenominator {
        operation: &'static str,
        denominator: GameInt,
    },
    #[error("{operation}: result is outside the signed 64-bit gameplay range (operands: {left}, {right})")]
    Overflow {
        operation: &'static str,
        left: GameInt,
        right: GameInt,
    },
}

/// Computes `floor(value * numerator / denominator)` exactly.
pub fn mul_div_floor(
    value: GameInt,
    numerator: GameInt,
    denominator: GameInt,
    operation: &'static str,
) -> Result<GameInt, NumericError> {
    if denominator <= 0 {
        return Err(NumericError::InvalidDenominator {
            operation,
            denominator,
        });
    }
    let product =
        i128::from(value)
            .checked_mul(i128::from(numerator))
            .ok_or(NumericError::Overflow {
                operation,
                left: value,
                right: numerator,
            })?;
    let quotient = product.div_euclid(i128::from(denominator));
    GameInt::try_from(quotient).map_err(|_| NumericError::Overflow {
        operation,
        left: value,
        right: numerator,
    })
}

pub fn apply_basis_points(
    value: GameInt,
    basis_points: GameInt,
    operation: &'static str,
) -> Result<GameInt, NumericError> {
    mul_div_floor(value, basis_points, BASIS_POINTS, operation)
}

pub fn checked_add(
    left: GameInt,
    right: GameInt,
    operation: &'static str,
) -> Result<GameInt, NumericError> {
    left.checked_add(right).ok_or(NumericError::Overflow {
        operation,
        left,
        right,
    })
}

pub fn checked_sub(
    left: GameInt,
    right: GameInt,
    operation: &'static str,
) -> Result<GameInt, NumericError> {
    left.checked_sub(right).ok_or(NumericError::Overflow {
        operation,
        left,
        right,
    })
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum ModifierOperation {
    FlatAdd { value: GameInt },
    FlatSubtract { value: GameInt },
    AdditivePercent { basis_points: GameInt },
    MultiplicativeRatio { ratio: Ratio },
    Minimum { value: GameInt },
    Maximum { value: GameInt },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Modifier {
    pub source_id: String,
    #[serde(flatten)]
    pub operation: ModifierOperation,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModifierStep {
    pub source_id: String,
    pub operation: ModifierOperation,
    pub before: GameInt,
    pub after: GameInt,
}

/// Applies the canonical stage order. Multiplicative operations use stable source
/// ID order so registry insertion order cannot change a result.
pub fn apply_modifiers(
    input: GameInt,
    modifiers: &[Modifier],
) -> Result<(GameInt, Vec<ModifierStep>), NumericError> {
    let mut value = input;
    let mut steps = Vec::new();

    for modifier in modifiers {
        match modifier.operation {
            ModifierOperation::FlatAdd { value: amount } => {
                let after = checked_add(value, amount, "flat addition")?;
                push_step(&mut value, &mut steps, modifier, after);
            }
            ModifierOperation::FlatSubtract { value: amount } => {
                let after = checked_add(value, -amount, "flat subtraction")?;
                push_step(&mut value, &mut steps, modifier, after);
            }
            _ => {}
        }
    }

    let additive_bps = modifiers.iter().try_fold(0, |sum, modifier| {
        if let ModifierOperation::AdditivePercent { basis_points } = modifier.operation {
            checked_add(sum, basis_points, "additive percentage sum")
        } else {
            Ok(sum)
        }
    })?;
    if additive_bps != 0 {
        let synthetic = Modifier {
            source_id: "engine:additive-percent-sum".into(),
            operation: ModifierOperation::AdditivePercent {
                basis_points: additive_bps,
            },
        };
        let factor = checked_add(BASIS_POINTS, additive_bps, "additive percentage factor")?;
        let after = apply_basis_points(value, factor, "additive percentage")?;
        push_step(&mut value, &mut steps, &synthetic, after);
    }

    let mut multiplicative: Vec<_> = modifiers
        .iter()
        .filter(|modifier| {
            matches!(
                modifier.operation,
                ModifierOperation::MultiplicativeRatio { .. }
            )
        })
        .collect();
    multiplicative.sort_by(|left, right| left.source_id.cmp(&right.source_id));
    for modifier in multiplicative {
        if let ModifierOperation::MultiplicativeRatio { ratio } = modifier.operation {
            let after = ratio.apply(value, "multiplicative modifier")?;
            push_step(&mut value, &mut steps, modifier, after);
        }
    }

    for modifier in modifiers {
        match modifier.operation {
            ModifierOperation::Minimum { value: minimum } => {
                let after = value.max(minimum);
                push_step(&mut value, &mut steps, modifier, after);
            }
            ModifierOperation::Maximum { value: maximum } => {
                let after = value.min(maximum);
                push_step(&mut value, &mut steps, modifier, after);
            }
            _ => {}
        }
    }

    Ok((value, steps))
}

fn push_step(
    value: &mut GameInt,
    steps: &mut Vec<ModifierStep>,
    modifier: &Modifier,
    after: GameInt,
) {
    let before = *value;
    *value = after;
    steps.push(ModifierStep {
        source_id: modifier.source_id.clone(),
        operation: modifier.operation.clone(),
        before,
        after,
    });
}
