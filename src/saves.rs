use crate::{
    campaign::{CampaignProgress, PlayerCharacter},
    entities::Entity,
    session::CombatSession,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use thiserror::Error;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

pub const SAVE_FORMAT_VERSION: u32 = 3;
static TEMP_SAVE_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const METADATA_ENTRY: &str = "metadata.json";
const RULESET_ENTRY: &str = "ruleset.json";
const ENTITIES_ENTRY: &str = "entities.json";
const COMBAT_ENTRY: &str = "combat.json";
const MOD_DATA_ENTRY: &str = "mod_data.json";
const CHARACTER_ENTRY: &str = "character.json";
const CAMPAIGN_ENTRY: &str = "campaign.json";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequiredMod {
    pub id: String,
    pub version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SaveMetadata {
    pub save_format_version: u32,
    #[serde(default)]
    pub save_id: String,
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
    pub character: Option<PlayerCharacter>,
    #[serde(default)]
    pub campaign: Option<CampaignProgress>,
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
    #[error("invalid save name {name:?}: use 1-64 Unicode characters without path separators, control characters, or Windows-reserved filename characters")]
    InvalidName { name: String },
    #[error("invalid player character in save {path}: {reason}")]
    InvalidCharacter { path: PathBuf, reason: String },
}

pub fn write_save(path: &Path, data: &SaveData) -> Result<(), SaveError> {
    if let Some(character) = &data.character {
        character
            .validate()
            .map_err(|reason| SaveError::InvalidCharacter {
                path: path.to_path_buf(),
                reason,
            })?;
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let filename = path.file_name().unwrap_or_default().to_string_lossy();
    let sequence = TEMP_SAVE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temp_path = parent.join(format!(".{filename}.{}-{sequence}.tmp", std::process::id()));
    let _cleanup = TempCleanup(temp_path.clone());
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)
        .map_err(|source| SaveError::Io {
            path: temp_path.clone(),
            source,
        })?;
    let mut archive = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    write_json(&mut archive, path, METADATA_ENTRY, &data.metadata, options)?;
    write_json(&mut archive, path, RULESET_ENTRY, &data.ruleset, options)?;
    write_json(&mut archive, path, ENTITIES_ENTRY, &data.entities, options)?;
    if let Some(character) = &data.character {
        write_json(&mut archive, path, CHARACTER_ENTRY, character, options)?;
    }
    if let Some(campaign) = &data.campaign {
        write_json(&mut archive, path, CAMPAIGN_ENTRY, campaign, options)?;
    }
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
    // Unix rename replaces atomically. On Windows, preserve the previous file
    // through a backup rename before installing the completed archive.
    if path.exists() {
        let backup = parent.join(format!(".{filename}.{}-{sequence}.bak", std::process::id()));
        std::fs::rename(path, &backup).map_err(|source| SaveError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if let Err(source) = std::fs::rename(&temp_path, path) {
            let _ = std::fs::rename(&backup, path);
            let _ = std::fs::remove_file(&temp_path);
            return Err(SaveError::Io {
                path: path.to_path_buf(),
                source,
            });
        }
        std::fs::remove_file(backup).map_err(|source| SaveError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    } else {
        std::fs::rename(&temp_path, path).map_err(|source| SaveError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    }
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
    let character: Option<PlayerCharacter> = if metadata.save_format_version >= 3 {
        read_optional_json(&mut archive, path, CHARACTER_ENTRY)?
    } else {
        None
    };
    if let Some(character) = &character {
        character
            .validate()
            .map_err(|reason| SaveError::InvalidCharacter {
                path: path.to_path_buf(),
                reason,
            })?;
    }
    let campaign: Option<CampaignProgress> = if metadata.save_format_version >= 3 {
        read_optional_json(&mut archive, path, CAMPAIGN_ENTRY)?
    } else {
        None
    };
    let migrated_from = metadata.save_format_version;
    let mut metadata = metadata;
    if metadata.save_id.is_empty() {
        metadata.save_id = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
    }
    metadata.save_format_version = SAVE_FORMAT_VERSION;
    Ok(SaveData {
        metadata,
        ruleset,
        entities,
        character,
        campaign,
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

struct TempCleanup(PathBuf);
impl Drop for TempCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
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
    let reserved = ["CON", "PRN", "AUX", "NUL"];
    let stem = name
        .split('.')
        .next()
        .unwrap_or_default()
        .trim_end_matches([' ', '.'])
        .to_ascii_uppercase();
    let reserved_device = reserved.iter().any(|value| stem == *value)
        || ["COM", "LPT"].iter().any(|prefix| {
            stem.strip_prefix(prefix).is_some_and(|suffix| {
                suffix.len() == 1 && matches!(suffix.as_bytes()[0], b'1'..=b'9')
            })
        });
    let valid = !name.trim().is_empty()
        && name.chars().count() <= 64
        && name != "."
        && name != ".."
        && !matches!(name.chars().last(), Some('.' | ' '))
        && !reserved_device
        && !name.chars().any(|character| {
            character.is_control()
                || matches!(
                    character,
                    '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'
                )
        });
    valid
        .then_some(())
        .ok_or_else(|| SaveError::InvalidName { name: name.into() })
}

#[cfg(test)]
mod save_name_tests {
    use super::validate_save_name;

    #[test]
    fn accepts_unicode_names_without_transliteration() {
        for name in ["João", "Éowyn", "Dragão Negro"] {
            assert!(validate_save_name(name).is_ok(), "{name:?}");
        }
    }

    #[test]
    fn rejects_names_that_are_not_portable_path_components() {
        for name in [
            "", "   ", ".", "..", "../save", "a/b", "a\\b", "CON", "nul.txt", "name.", "name ",
            "bad:name", "a\nb",
        ] {
            assert!(validate_save_name(name).is_err(), "{name:?}");
        }
    }

    #[test]
    fn enforces_limit_in_unicode_characters() {
        assert!(validate_save_name(&"é".repeat(64)).is_ok());
        assert!(validate_save_name(&"é".repeat(65)).is_err());
    }
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
