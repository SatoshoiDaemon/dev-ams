use crate::engine::GameEvent;
use std::collections::VecDeque;

/// Deterministic FIFO event bus. Presentation and Lua integrations can consume
/// events without becoming responsible for game rules.
#[derive(Default)]
pub struct EventBus { queue: VecDeque<GameEvent> }
impl EventBus {
    pub fn publish(&mut self, event: GameEvent) { self.queue.push_back(event); }
    pub fn drain(&mut self) -> impl Iterator<Item = GameEvent> + '_ { self.queue.drain(..) }
    pub fn len(&self) -> usize { self.queue.len() }
}
