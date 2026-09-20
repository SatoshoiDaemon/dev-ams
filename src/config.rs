use serde::{Deserialize, Serialize};
use std::{fs, io, path::{Path, PathBuf}};
use thiserror::Error;
#[derive(Debug, Error)] pub enum ConfigError {
    #[error("failed to read configuration {path}: {source}")] Read { path: PathBuf, source: io::Error },
    #[error("failed to parse configuration {path}: {source}")] Parse { path: PathBuf, source: toml::de::Error },
    #[error("failed to create runtime directory {path}: {source}")] Directory { path: PathBuf, source: io::Error },
}
#[derive(Clone, Debug, Serialize, Deserialize)] #[serde(default)] pub struct AppConfig { pub schema_version: u32, pub seed: u64, pub paths: PathConfig, pub numeric: NumericConfig }
impl Default for AppConfig { fn default() -> Self { Self { schema_version: 1, seed: 0x1234_5678, paths: PathConfig::default(), numeric: NumericConfig::default() } } }
impl AppConfig { pub fn load(root: &Path) -> Result<Self, ConfigError> { let path = root.join("config.toml"); if !path.exists() { return Ok(Self::default()); } let text = fs::read_to_string(&path).map_err(|source| ConfigError::Read { path: path.clone(), source })?; toml::from_str(&text).map_err(|source| ConfigError::Parse { path, source }) } }
#[derive(Clone, Debug, Serialize, Deserialize)] #[serde(default)] pub struct PathConfig { pub saves: String, pub mods: String, pub data: String, pub gamemodes: String, pub logs: String }
impl Default for PathConfig { fn default() -> Self { Self { saves: "saves".into(), mods: "mods".into(), data: "data".into(), gamemodes: "gamemodes".into(), logs: "logs".into() } } }
impl PathConfig { pub fn resolve(&self, root: &Path) -> GamePaths { GamePaths { saves: root.join(&self.saves), mods: root.join(&self.mods), data: root.join(&self.data), gamemodes: root.join(&self.gamemodes), logs: root.join(&self.logs) } } }
#[derive(Clone, Debug, Serialize, Deserialize)] #[serde(default)] pub struct NumericConfig { pub hp_per_vigor: i32, pub tenacity_per_resistance: i32, pub mp_per_mana: i32, pub power_per_attribute: i32, pub tenacity_damage_share: f64 }
impl Default for NumericConfig { fn default() -> Self { Self { hp_per_vigor: 10, tenacity_per_resistance: 10, mp_per_mana: 10, power_per_attribute: 7, tenacity_damage_share: 0.5 } } }
#[derive(Clone, Debug)] pub struct GamePaths { pub saves: PathBuf, pub mods: PathBuf, pub data: PathBuf, pub gamemodes: PathBuf, pub logs: PathBuf }
impl GamePaths { pub fn create_all(&self) -> Result<(), ConfigError> { for path in [&self.saves, &self.mods, &self.data, &self.gamemodes, &self.logs] { fs::create_dir_all(path).map_err(|source| ConfigError::Directory { path: path.clone(), source })?; } Ok(()) } }
