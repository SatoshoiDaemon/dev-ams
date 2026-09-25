//! Engine foundation. Simulation modules do not depend on terminal rendering.
pub mod actions;
pub mod app;
pub mod attributes;
pub mod combat;
pub mod config;
pub mod content;
pub mod effects;
pub mod encounters;
pub mod engine;
pub mod entities;
pub mod events;
pub mod glyphs;
pub mod logging;
pub mod lua;
pub mod magic;
pub mod modding;
pub mod numeric;
pub mod runtime;
pub mod saves;
pub mod scaling;
pub mod session;
pub mod terminal;
pub use config::{AppConfig, GamePaths};
pub use engine::{GameEvent, GameState};
use std::path::Path;
pub fn initialize(root: impl AsRef<Path>) -> Result<GameState, anyhowless::Error> {
    let root = root.as_ref();
    let config = AppConfig::load(root)?;
    let paths = config.paths.resolve(root);
    paths.create_all()?;
    let logger = logging::Logger::open_with_config(&paths.logs, &config.logging)?;
    logging::install_panic_hook(logger.clone());
    logger.info("Game starting")?;
    logger.info(&format!("Loading {}", root.join("config.toml").display()))?;
    let glyph_path = paths.data.join("glyphs.json");
    logger.info(&format!("Loading {}", glyph_path.display()))?;
    let glyph_registry = glyphs::GlyphRegistry::load(&glyph_path)?;
    logger.info(&format!(
        "Loaded {} Glyph definitions",
        glyph_registry.len()
    ))?;
    logger.info("Discovering mods...")?;
    let mod_report = modding::discover_mods_report(&paths.mods)?;
    logger.info(&format!("Found {} loadable mods", mod_report.loaded.len()))?;
    for failure in &mod_report.failed {
        logger.error(&format!(
            "Failed to load mod at {} during {}: {}",
            failure.path.display(),
            failure.stage,
            failure.reason
        ))?;
    }
    logger.info(&format!(
        "Loading game modes from {}",
        paths.gamemodes.display()
    ))?;
    let game_modes = config::GameModeRegistry::load(&paths.gamemodes)?;
    let standard_mode = game_modes.get("base:standard");
    let resolved_numeric = standard_mode
        .map(|mode| mode.resolve_numeric(&config.numeric))
        .unwrap_or_else(|| config.numeric.clone());
    let mut state = GameState::new(config.seed);
    state.log_path = Some(paths.logs.join("latest.log"));
    state.paths = Some(paths.clone());
    state.logger = Some(logger.clone());
    state.global_numeric = config.numeric.clone();
    state.lua_max_instructions = config.lua.max_instructions_per_callback;
    state.attribute_rules = resolved_numeric.attribute_rules();
    state.ruleset_version = standard_mode
        .map(|mode| mode.ruleset_version.clone())
        .unwrap_or_else(|| config.ruleset_version.clone());
    state.game_modes = game_modes;
    state.glyph_registry = glyph_registry;
    let content_root = paths.data.join("content");
    let status_path = content_root.join("statuses.json");
    logger.info(&format!("Loading {}", status_path.display()))?;
    state.status_registry = effects::StatusRegistry::load(&status_path)?;
    let action_path = content_root.join("actions.json");
    logger.info(&format!("Loading {}", action_path.display()))?;
    state.action_registry = actions::ActionRegistry::load(&action_path)?;
    state.action_registry.validate(&state.status_registry)?;
    logger.info(&format!(
        "Loaded {} action definitions",
        state.action_registry.len()
    ))?;
    state.encounter_registry = encounters::EncounterRegistry::load(&content_root)?;
    state.encounter_registry.validate(&state.action_registry)?;
    state.spell_template_registry = magic::SpellTemplateRegistry::load(
        &content_root.join("spell_templates.json"),
        &state.action_registry,
        &state.glyph_registry,
    )?;
    state.loaded_mods = mod_report
        .loaded
        .iter()
        .map(|discovered| discovered.manifest.id.as_str().to_owned())
        .collect();
    state.mod_roots = mod_report
        .loaded
        .iter()
        .map(|discovered| {
            (
                discovered.manifest.id.as_str().to_owned(),
                discovered.root.clone(),
            )
        })
        .collect();
    state.loaded_mod_versions = mod_report
        .loaded
        .iter()
        .map(|discovered| {
            (
                discovered.manifest.id.as_str().to_owned(),
                discovered.manifest.version.clone(),
            )
        })
        .collect();
    Ok(state)
}
pub mod anyhowless {
    use std::{fmt, io};
    #[derive(Debug)]
    pub enum Error {
        Io(io::Error),
        Config(Box<crate::config::ConfigError>),
        Glyph(Box<crate::glyphs::GlyphError>),
        Mod(Box<crate::modding::ModError>),
        Action(Box<crate::actions::ActionContentError>),
        Encounter(Box<crate::encounters::EncounterError>),
        Status(Box<crate::effects::StatusError>),
        SpellTemplate(Box<crate::magic::SpellTemplateError>),
    }
    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Io(e) => e.fmt(f),
                Self::Config(e) => e.fmt(f),
                Self::Glyph(e) => e.fmt(f),
                Self::Mod(e) => e.fmt(f),
                Self::Action(e) => e.fmt(f),
                Self::Encounter(e) => e.fmt(f),
                Self::Status(e) => e.fmt(f),
                Self::SpellTemplate(e) => e.fmt(f),
            }
        }
    }
    impl std::error::Error for Error {}
    impl From<io::Error> for Error {
        fn from(e: io::Error) -> Self {
            Self::Io(e)
        }
    }
    impl From<crate::config::ConfigError> for Error {
        fn from(e: crate::config::ConfigError) -> Self {
            Self::Config(Box::new(e))
        }
    }
    impl From<crate::glyphs::GlyphError> for Error {
        fn from(e: crate::glyphs::GlyphError) -> Self {
            Self::Glyph(Box::new(e))
        }
    }
    impl From<crate::modding::ModError> for Error {
        fn from(e: crate::modding::ModError) -> Self {
            Self::Mod(Box::new(e))
        }
    }
    impl From<crate::actions::ActionContentError> for Error {
        fn from(e: crate::actions::ActionContentError) -> Self {
            Self::Action(Box::new(e))
        }
    }
    impl From<crate::encounters::EncounterError> for Error {
        fn from(e: crate::encounters::EncounterError) -> Self {
            Self::Encounter(Box::new(e))
        }
    }
    impl From<crate::effects::StatusError> for Error {
        fn from(e: crate::effects::StatusError) -> Self {
            Self::Status(Box::new(e))
        }
    }
    impl From<crate::magic::SpellTemplateError> for Error {
        fn from(e: crate::magic::SpellTemplateError) -> Self {
            Self::SpellTemplate(Box::new(e))
        }
    }
}
