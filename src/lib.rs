//! Engine foundation. Simulation modules do not depend on terminal rendering.
pub mod app;
pub mod combat;
pub mod config;
pub mod content;
pub mod engine;
pub mod entities;
pub mod events;
pub mod logging;
pub mod lua;
pub mod modding;
pub mod saves;
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
    let mut state = GameState::new(config.seed);
    state.log_path = Some(paths.logs.join("latest.log"));
    Ok(state)
}
pub mod anyhowless {
    use std::{fmt, io};
    #[derive(Debug)] pub enum Error { Io(io::Error), Config(crate::config::ConfigError) }
    impl fmt::Display for Error { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { match self { Self::Io(e) => e.fmt(f), Self::Config(e) => e.fmt(f) } } }
    impl std::error::Error for Error {}
    impl From<io::Error> for Error { fn from(e: io::Error) -> Self { Self::Io(e) } }
    impl From<crate::config::ConfigError> for Error { fn from(e: crate::config::ConfigError) -> Self { Self::Config(e) } }
}
