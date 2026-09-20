use crate::entities::Entity;
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

pub const SAVE_FORMAT_VERSION: u32 = 1;
const METADATA_ENTRY: &str = "metadata.json";
const ENTITIES_ENTRY: &str = "entities.json";
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
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SaveData {
    pub metadata: SaveMetadata,
    pub entities: Vec<Entity>,
    #[serde(default)]
    pub opaque_mod_data: BTreeMap<String, Value>,
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
}

pub fn write_save(path: &Path, data: &SaveData) -> Result<(), SaveError> {
    let file = File::create(path).map_err(|source| SaveError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut archive = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    write_json(&mut archive, path, METADATA_ENTRY, &data.metadata, options)?;
    write_json(&mut archive, path, ENTITIES_ENTRY, &data.entities, options)?;
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
    if metadata.save_format_version != SAVE_FORMAT_VERSION {
        return Err(SaveError::UnsupportedVersion {
            path: path.to_path_buf(),
            received: metadata.save_format_version,
            supported: SAVE_FORMAT_VERSION,
        });
    }
    let entities = read_json(&mut archive, path, ENTITIES_ENTRY)?;
    let opaque_mod_data = read_json(&mut archive, path, MOD_DATA_ENTRY)?;
    Ok(SaveData {
        metadata,
        entities,
        opaque_mod_data,
    })
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
