use crate::{attributes::AttributeRules, numeric::GameInt};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read configuration {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("failed to parse configuration {path}: {source}")]
    Parse {
        path: PathBuf,
        source: Box<toml::de::Error>,
    },
    #[error("failed to create runtime directory {path}: {source}")]
    Directory { path: PathBuf, source: io::Error },
    #[error("invalid configuration field {field}: {reason}")]
    Validation { field: &'static str, reason: String },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub schema_version: u32,
    pub seed: u64,
    pub ruleset_version: String,
    pub paths: PathConfig,
    pub numeric: NumericConfig,
    pub logging: LoggingConfig,
    pub lua: LuaConfig,
}
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            seed: 0x1234_5678,
            ruleset_version: "base:standard@1".into(),
            paths: PathConfig::default(),
            numeric: NumericConfig::default(),
            logging: LoggingConfig::default(),
            lua: LuaConfig::default(),
        }
    }
}
impl AppConfig {
    pub fn load(root: &Path) -> Result<Self, ConfigError> {
        let path = root.join("config.toml");
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = fs::read_to_string(&path).map_err(|source| ConfigError::Read {
            path: path.clone(),
            source,
        })?;
        let config: Self = toml::from_str(&text).map_err(|source| ConfigError::Parse {
            path,
            source: Box::new(source),
        })?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        for (field, value) in [
            ("numeric.base_hp", self.numeric.base_hp),
            ("numeric.base_tenacity", self.numeric.base_tenacity),
            ("numeric.base_mana", self.numeric.base_mana),
            ("numeric.hp_per_vigor", self.numeric.hp_per_vigor),
            (
                "numeric.tenacity_per_resistance",
                self.numeric.tenacity_per_resistance,
            ),
            ("numeric.mp_per_mana", self.numeric.mp_per_mana),
            (
                "numeric.power_per_attribute",
                self.numeric.power_per_attribute,
            ),
        ] {
            if value < 0 {
                return Err(ConfigError::Validation {
                    field,
                    reason: format!("must be non-negative, received {value}"),
                });
            }
        }
        if !(0..=10_000).contains(&self.numeric.tenacity_hp_share_bps) {
            return Err(ConfigError::Validation {
                field: "numeric.tenacity_hp_share_bps",
                reason: format!(
                    "must be between 0 and 10000 basis points, received {}",
                    self.numeric.tenacity_hp_share_bps
                ),
            });
        }
        if self.lua.max_instructions_per_callback == 0 {
            return Err(ConfigError::Validation {
                field: "lua.max_instructions_per_callback",
                reason: "must be greater than zero".into(),
            });
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PathConfig {
    pub saves: String,
    pub mods: String,
    pub data: String,
    pub gamemodes: String,
    pub logs: String,
}
impl Default for PathConfig {
    fn default() -> Self {
        Self {
            saves: "saves".into(),
            mods: "mods".into(),
            data: "data".into(),
            gamemodes: "gamemodes".into(),
            logs: "logs".into(),
        }
    }
}
impl PathConfig {
    pub fn resolve(&self, root: &Path) -> GamePaths {
        GamePaths {
            saves: root.join(&self.saves),
            mods: root.join(&self.mods),
            data: root.join(&self.data),
            gamemodes: root.join(&self.gamemodes),
            logs: root.join(&self.logs),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct NumericConfig {
    pub base_hp: GameInt,
    pub base_tenacity: GameInt,
    pub base_mana: GameInt,
    pub hp_per_vigor: GameInt,
    pub tenacity_per_resistance: GameInt,
    pub mp_per_mana: GameInt,
    pub power_per_attribute: GameInt,
    pub tenacity_hp_share_bps: GameInt,
}
impl Default for NumericConfig {
    fn default() -> Self {
        Self {
            base_hp: 100,
            base_tenacity: 50,
            base_mana: 30,
            hp_per_vigor: 10,
            tenacity_per_resistance: 10,
            mp_per_mana: 10,
            power_per_attribute: 7,
            tenacity_hp_share_bps: 5_000,
        }
    }
}
impl NumericConfig {
    pub fn attribute_rules(&self) -> AttributeRules {
        AttributeRules {
            base_hp: self.base_hp,
            base_tenacity: self.base_tenacity,
            base_mana: self.base_mana,
            hp_per_vigor: self.hp_per_vigor,
            tenacity_per_resistance: self.tenacity_per_resistance,
            mana_per_mana: self.mp_per_mana,
            power_per_attribute: self.power_per_attribute,
            priority_base: 100,
            priority_per_dexterity: 2,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub level: String,
    pub combat_debug: bool,
    pub keep_sessions: Option<usize>,
    pub keep_crash_reports: Option<usize>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".into(),
            combat_debug: false,
            keep_sessions: Some(20),
            keep_crash_reports: Some(50),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LuaConfig {
    pub max_instructions_per_callback: u64,
}

impl Default for LuaConfig {
    fn default() -> Self {
        Self {
            max_instructions_per_callback: 100_000,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct NumericOverrides {
    pub base_hp: Option<GameInt>,
    pub base_tenacity: Option<GameInt>,
    pub base_mana: Option<GameInt>,
    pub hp_per_vigor: Option<GameInt>,
    pub tenacity_per_resistance: Option<GameInt>,
    pub mp_per_mana: Option<GameInt>,
    pub power_per_attribute: Option<GameInt>,
    pub tenacity_hp_share_bps: Option<GameInt>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameMode {
    pub id: String,
    pub ruleset_version: String,
    #[serde(default)]
    pub numeric: NumericOverrides,
}

impl GameMode {
    pub fn resolve_numeric(&self, global: &NumericConfig) -> NumericConfig {
        NumericConfig {
            base_hp: self.numeric.base_hp.unwrap_or(global.base_hp),
            base_tenacity: self.numeric.base_tenacity.unwrap_or(global.base_tenacity),
            base_mana: self.numeric.base_mana.unwrap_or(global.base_mana),
            hp_per_vigor: self.numeric.hp_per_vigor.unwrap_or(global.hp_per_vigor),
            tenacity_per_resistance: self
                .numeric
                .tenacity_per_resistance
                .unwrap_or(global.tenacity_per_resistance),
            mp_per_mana: self.numeric.mp_per_mana.unwrap_or(global.mp_per_mana),
            power_per_attribute: self
                .numeric
                .power_per_attribute
                .unwrap_or(global.power_per_attribute),
            tenacity_hp_share_bps: self
                .numeric
                .tenacity_hp_share_bps
                .unwrap_or(global.tenacity_hp_share_bps),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct GameModeRegistry {
    modes: BTreeMap<String, GameMode>,
}

impl GameModeRegistry {
    pub fn load(root: &Path) -> Result<Self, ConfigError> {
        let mut paths = Vec::new();
        for entry in fs::read_dir(root).map_err(|source| ConfigError::Read {
            path: root.to_path_buf(),
            source,
        })? {
            let path = entry
                .map_err(|source| ConfigError::Read {
                    path: root.to_path_buf(),
                    source,
                })?
                .path();
            if path.extension().and_then(|value| value.to_str()) == Some("toml") {
                paths.push(path);
            }
        }
        paths.sort();
        let mut registry = Self::default();
        for path in paths {
            let text = fs::read_to_string(&path).map_err(|source| ConfigError::Read {
                path: path.clone(),
                source,
            })?;
            let mode: GameMode = toml::from_str(&text).map_err(|source| ConfigError::Parse {
                path: path.clone(),
                source: Box::new(source),
            })?;
            if registry.modes.insert(mode.id.clone(), mode).is_some() {
                return Err(ConfigError::Validation {
                    field: "gamemode.id",
                    reason: format!("duplicate game mode ID in {}", path.display()),
                });
            }
        }
        Ok(registry)
    }

    pub fn get(&self, id: &str) -> Option<&GameMode> {
        self.modes.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &GameMode)> {
        self.modes.iter()
    }
}
#[derive(Clone, Debug)]
pub struct GamePaths {
    pub saves: PathBuf,
    pub mods: PathBuf,
    pub data: PathBuf,
    pub gamemodes: PathBuf,
    pub logs: PathBuf,
}
impl GamePaths {
    pub fn create_all(&self) -> Result<(), ConfigError> {
        for path in [
            &self.saves,
            &self.mods,
            &self.data,
            &self.gamemodes,
            &self.logs,
        ] {
            fs::create_dir_all(path).map_err(|source| ConfigError::Directory {
                path: path.clone(),
                source,
            })?;
        }
        Ok(())
    }
}
