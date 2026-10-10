use crate::campaign::{LocationRegistry, RaceRegistry, SceneRegistry};
use crate::{
    actions::ActionRegistry,
    attributes::AttributeRules,
    combat::{DeterministicRng, ExplanationStore},
    config::{GameModeRegistry, GamePaths, NumericConfig},
    effects::StatusRegistry,
    encounters::EncounterRegistry,
    events::CombatEvent,
    glyphs::GlyphRegistry,
    logging::Logger,
    magic::SpellTemplateRegistry,
    modding::ModLoadReport,
    session::CombatSession,
};
use std::collections::BTreeMap;
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
    pub mod_load_report: ModLoadReport,
    pub mod_roots: BTreeMap<String, PathBuf>,
    pub ruleset_version: String,
    pub action_registry: ActionRegistry,
    pub encounter_registry: EncounterRegistry,
    pub spell_template_registry: SpellTemplateRegistry,
    pub combat: Option<CombatSession>,
    pub game_modes: GameModeRegistry,
    pub global_numeric: NumericConfig,
    pub lua_max_instructions: u64,
    pub loaded_mod_versions: BTreeMap<String, String>,
    pub paths: Option<GamePaths>,
    pub logger: Option<Logger>,
    pub scene_registry: SceneRegistry,
    pub location_registry: LocationRegistry,
    pub race_registry: RaceRegistry,
    pub visual_effects: String,
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
            mod_load_report: ModLoadReport::default(),
            mod_roots: BTreeMap::new(),
            ruleset_version: "base:standard@1".into(),
            action_registry: ActionRegistry::default(),
            encounter_registry: EncounterRegistry::default(),
            spell_template_registry: SpellTemplateRegistry::default(),
            combat: None,
            game_modes: GameModeRegistry::default(),
            global_numeric: NumericConfig::default(),
            lua_max_instructions: crate::lua::DEFAULT_MAX_INSTRUCTIONS,
            loaded_mod_versions: BTreeMap::new(),
            paths: None,
            logger: None,
            scene_registry: SceneRegistry::default(),
            location_registry: LocationRegistry::default(),
            race_registry: RaceRegistry::default(),
            visual_effects: "reduced".into(),
        }
    }
    pub fn emit(&mut self, event: GameEvent) {
        self.events.push_back(event);
    }
}
