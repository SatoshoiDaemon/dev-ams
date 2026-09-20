use std::{collections::VecDeque, path::PathBuf};
use serde::{Deserialize, Serialize};
use crate::combat::DeterministicRng;
pub type Tick = u64;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)] pub enum GameEvent { TurnStarted { tick: Tick }, TurnEnded { tick: Tick }, Damage { source: String, target: String, amount: i32 }, HpChanged { entity: String, amount: i32 }, EntityKilled { entity: String } }
#[derive(Clone, Debug)] pub struct GameState { pub seed: u64, pub tick: Tick, pub events: VecDeque<GameEvent>, pub log_path: Option<PathBuf>, pub rng: DeterministicRng }
impl GameState { pub fn new(seed: u64) -> Self { Self { seed, tick: 0, events: VecDeque::new(), log_path: None, rng: DeterministicRng::new(seed) } } pub fn emit(&mut self, event: GameEvent) { self.events.push_back(event); } }
