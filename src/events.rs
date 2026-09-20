use crate::numeric::GameInt;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const MAX_TRIGGER_DEPTH: u8 = 16;
pub const MAX_EVENTS_PER_ACTION: usize = 256;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum CombatEvent {
    TurnStarted {
        round: u64,
        entity_id: String,
    },
    TurnEnded {
        round: u64,
        entity_id: String,
    },
    OnEvade {
        source_id: String,
        target_id: String,
    },
    OnParry {
        source_id: String,
        target_id: String,
    },
    OnBlock {
        source_id: String,
        target_id: String,
        amount: GameInt,
    },
    OnAttackReceived {
        source_id: String,
        target_id: String,
    },
    OnDamageReceived {
        source_id: String,
        target_id: String,
        amount: GameInt,
    },
    OnShieldDamage {
        source_id: String,
        target_id: String,
        amount: GameInt,
    },
    OnTenacityDamage {
        source_id: String,
        target_id: String,
        amount: GameInt,
    },
    OnHpDamage {
        source_id: String,
        target_id: String,
        amount: GameInt,
    },
    OnHpBelow {
        target_id: String,
        threshold_percent: GameInt,
    },
    OnKill {
        source_id: String,
        target_id: String,
    },
    StatusApplied {
        target_id: String,
        effect_id: String,
        source_id: String,
    },
    StatusRemoved {
        target_id: String,
        effect_id: String,
        source_id: String,
    },
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum EventError {
    #[error("combat trigger depth {received} exceeds maximum {maximum}")]
    TriggerDepth { received: u8, maximum: u8 },
    #[error("combat event count exceeds maximum {maximum} for one action")]
    EventCount { maximum: usize },
}

/// Deterministic bounded FIFO event bus. Presentation and Lua integrations can
/// consume events without becoming responsible for game rules.
#[derive(Clone, Debug, Default)]
pub struct EventBus {
    queue: VecDeque<(u8, CombatEvent)>,
    published_for_action: usize,
}
impl EventBus {
    pub fn begin_action(&mut self) {
        self.published_for_action = 0;
    }
    pub fn publish(&mut self, depth: u8, event: CombatEvent) -> Result<(), EventError> {
        if depth > MAX_TRIGGER_DEPTH {
            return Err(EventError::TriggerDepth {
                received: depth,
                maximum: MAX_TRIGGER_DEPTH,
            });
        }
        if self.published_for_action >= MAX_EVENTS_PER_ACTION {
            return Err(EventError::EventCount {
                maximum: MAX_EVENTS_PER_ACTION,
            });
        }
        self.published_for_action += 1;
        self.queue.push_back((depth, event));
        Ok(())
    }
    pub fn drain(&mut self) -> impl Iterator<Item = (u8, CombatEvent)> + '_ {
        self.queue.drain(..)
    }
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
