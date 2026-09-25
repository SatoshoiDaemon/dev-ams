use crate::{entities::Entity, session::CombatSession};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use thiserror::Error;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

pub const SAVE_FORMAT_VERSION: u32 = 2;
const METADATA_ENTRY: &str = "metadata.json";
const RULESET_ENTRY: &str = "ruleset.json";
const ENTITIES_ENTRY: &str = "entities.json";
const COMBAT_ENTRY: &str = "combat.json";
const MOD_DATA_ENTRY: &str = "mod_data.json";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequiredMod {
    pub id: String,
    pub version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SaveMetadata {
    pub save_format_version: u32,
    pub ruleset_version: String,
    #[serde(default)]
    pub required_mods: Vec<RequiredMod>,
    #[serde(default = "default_gamemode")]
    pub gamemode_id: String,
    #[serde(default)]
    pub player_entity_ids: Vec<String>,
    #[serde(default = "default_rng_state")]
    pub rng_state: u64,
}

fn default_gamemode() -> String {
    "base:standard".into()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SaveData {
    pub metadata: SaveMetadata,
    #[serde(default)]
    pub ruleset: Value,
    pub entities: Vec<Entity>,
    #[serde(default)]
    pub combat: Option<CombatSession>,
    #[serde(default)]
    pub opaque_mod_data: BTreeMap<String, Value>,
    #[serde(default, skip_serializing)]
    pub migration_warnings: Vec<String>,
}

fn default_rng_state() -> u64 {
    1
}

#[derive(Debug, Error)]
pub enum SaveError {
    #[error("failed to access save {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
    #[error("invalid ZIP save {path}: {source}")]
    Zip {
        path: PathBuf,
        source: zip::result::ZipError,
    },
    #[error("invalid JSON entry {entry} in save {path}: {source}")]
    Json {
        path: PathBuf,
        entry: &'static str,
        source: serde_json::Error,
    },
    #[error(
        "save {path} uses unsupported format version {received}; supported version is {supported}"
    )]
    UnsupportedVersion {
        path: PathBuf,
        received: u32,
        supported: u32,
    },
    #[error("invalid save name {name:?}: use 1-64 ASCII letters, digits, '_' or '-'")]
    InvalidName { name: String },
}

pub fn write_save(path: &Path, data: &SaveData) -> Result<(), SaveError> {
    let file = File::create(path).map_err(|source| SaveError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut archive = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    write_json(&mut archive, path, METADATA_ENTRY, &data.metadata, options)?;
    write_json(&mut archive, path, RULESET_ENTRY, &data.ruleset, options)?;
    write_json(&mut archive, path, ENTITIES_ENTRY, &data.entities, options)?;
    if let Some(combat) = &data.combat {
        write_json(&mut archive, path, COMBAT_ENTRY, combat, options)?;
    }
    write_json(
        &mut archive,
        path,
        MOD_DATA_ENTRY,
        &data.opaque_mod_data,
        options,
    )?;
    archive.finish().map_err(|source| SaveError::Zip {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(())
}

fn write_json<T: Serialize>(
    archive: &mut ZipWriter<File>,
    path: &Path,
    entry: &'static str,
    value: &T,
    options: SimpleFileOptions,
) -> Result<(), SaveError> {
    let json = serde_json::to_vec_pretty(value).map_err(|source| SaveError::Json {
        path: path.to_path_buf(),
        entry,
        source,
    })?;
    archive
        .start_file(entry, options)
        .map_err(|source| SaveError::Zip {
            path: path.to_path_buf(),
            source,
        })?;
    archive.write_all(&json).map_err(|source| SaveError::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub fn read_save(path: &Path) -> Result<SaveData, SaveError> {
    let file = File::open(path).map_err(|source| SaveError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut archive = ZipArchive::new(file).map_err(|source| SaveError::Zip {
        path: path.to_path_buf(),
        source,
    })?;
    let metadata: SaveMetadata = read_json(&mut archive, path, METADATA_ENTRY)?;
    if metadata.save_format_version == 0 || metadata.save_format_version > SAVE_FORMAT_VERSION {
        return Err(SaveError::UnsupportedVersion {
            path: path.to_path_buf(),
            received: metadata.save_format_version,
            supported: SAVE_FORMAT_VERSION,
        });
    }
    let entities = read_json(&mut archive, path, ENTITIES_ENTRY)?;
    let opaque_mod_data = read_json(&mut archive, path, MOD_DATA_ENTRY)?;
    let ruleset = if metadata.save_format_version >= 2 {
        read_json(&mut archive, path, RULESET_ENTRY)?
    } else {
        serde_json::json!({"ruleset_version": metadata.ruleset_version})
    };
    let combat = if metadata.save_format_version >= 2 {
        read_optional_json(&mut archive, path, COMBAT_ENTRY)?
    } else {
        None
    };
    let migrated_from = metadata.save_format_version;
    let mut metadata = metadata;
    metadata.save_format_version = SAVE_FORMAT_VERSION;
    Ok(SaveData {
        metadata,
        ruleset,
        entities,
        combat,
        opaque_mod_data,
        migration_warnings: (migrated_from == 1)
            .then(|| {
                "Save v1 had no ruleset snapshot; the current referenced ruleset was selected"
                    .to_owned()
            })
            .into_iter()
            .collect(),
    })
}

fn read_optional_json<T: for<'de> Deserialize<'de>>(
    archive: &mut ZipArchive<File>,
    path: &Path,
    entry: &'static str,
) -> Result<Option<T>, SaveError> {
    match archive.by_name(entry) {
        Ok(mut item) => {
            let mut text = String::new();
            item.read_to_string(&mut text)
                .map_err(|source| SaveError::Io {
                    path: path.to_path_buf(),
                    source,
                })?;
            serde_json::from_str(&text)
                .map(Some)
                .map_err(|source| SaveError::Json {
                    path: path.to_path_buf(),
                    entry,
                    source,
                })
        }
        Err(zip::result::ZipError::FileNotFound) => Ok(None),
        Err(source) => Err(SaveError::Zip {
            path: path.to_path_buf(),
            source,
        }),
    }
}

pub fn validate_save_name(name: &str) -> Result<(), SaveError> {
    let valid = !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'));
    valid
        .then_some(())
        .ok_or_else(|| SaveError::InvalidName { name: name.into() })
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnresolvedContentReference {
    pub id: String,
    pub owner_namespace: String,
    pub context: String,
}

fn read_json<T: for<'de> Deserialize<'de>>(
    archive: &mut ZipArchive<File>,
    path: &Path,
    entry: &'static str,
) -> Result<T, SaveError> {
    let mut item = archive.by_name(entry).map_err(|source| SaveError::Zip {
        path: path.to_path_buf(),
        source,
    })?;
    let mut text = String::new();
    item.read_to_string(&mut text)
        .map_err(|source| SaveError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    serde_json::from_str(&text).map_err(|source| SaveError::Json {
        path: path.to_path_buf(),
        entry,
        source,
    })
}

pub fn read_json_entry(save: &Path, entry: &str) -> io::Result<String> {
    let file = File::open(save)?;
    let mut archive = ZipArchive::new(file).map_err(io::Error::other)?;
    let mut item = archive.by_name(entry).map_err(io::Error::other)?;
    let mut text = String::new();
    item.read_to_string(&mut text)?;
    Ok(text)
}
