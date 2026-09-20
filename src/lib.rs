//! Engine foundation. Simulation modules do not depend on terminal rendering.
pub mod actions;
pub mod app;
pub mod attributes;
pub mod combat;
pub mod config;
pub mod content;
pub mod effects;
pub mod engine;
pub mod entities;
pub mod events;
pub mod glyphs;
pub mod logging;
pub mod lua;
pub mod modding;
pub mod numeric;
pub mod saves;
pub mod scaling;
pub mod terminal;
pub use config::{AppConfig, GamePaths};
pub use engine::{GameEvent, GameState};
use std::path::Path;
pub fn initialize(root: impl AsRef<Path>) -> Result<GameState, anyhowless::Error> {
    let root = root.as_ref();
    let config = AppConfig::load(root)?;
    let paths = config.paths.resolve(root);
    paths.create_all()?;
    let logger = logging::Logger::open(&paths.logs)?;
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
    let discovered_mods = modding::discover_mods(&paths.mods)?;
    logger.info(&format!("Found {} mods", discovered_mods.len()))?;
    let mut state = GameState::new(config.seed);
    state.log_path = Some(paths.logs.join("latest.log"));
    state.attribute_rules = config.numeric.attribute_rules();
    state.glyph_registry = glyph_registry;
    state.loaded_mods = discovered_mods
        .iter()
        .map(|discovered| discovered.manifest.id.as_str().to_owned())
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
    }
    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Io(e) => e.fmt(f),
                Self::Config(e) => e.fmt(f),
                Self::Glyph(e) => e.fmt(f),
                Self::Mod(e) => e.fmt(f),
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
}
