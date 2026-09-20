use crate::{
    attributes::AttributeRules,
    combat::{DeterministicRng, ExplanationStore},
    effects::StatusRegistry,
    events::CombatEvent,
    glyphs::GlyphRegistry,
};
use std::{collections::VecDeque, path::PathBuf};
pub type Tick = u64;
pub type GameEvent = CombatEvent;
#[derive(Clone, Debug)]
pub struct GameState {
    pub seed: u64,
    pub tick: Tick,
    pub events: VecDeque<GameEvent>,
    pub log_path: Option<PathBuf>,
    pub rng: DeterministicRng,
    pub explanations: ExplanationStore,
    pub attribute_rules: AttributeRules,
    pub status_registry: StatusRegistry,
    pub glyph_registry: GlyphRegistry,
    pub loaded_mods: Vec<String>,
}
impl GameState {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            tick: 0,
            events: VecDeque::new(),
            log_path: None,
            rng: DeterministicRng::new(seed),
            explanations: ExplanationStore::default(),
            attribute_rules: AttributeRules::default(),
            status_registry: StatusRegistry::base(),
            glyph_registry: GlyphRegistry::default(),
            loaded_mods: Vec::new(),
        }
    }
    pub fn emit(&mut self, event: GameEvent) {
        self.events.push_back(event);
    }
}
