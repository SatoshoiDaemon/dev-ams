use crate::{
    content::{load_json_registry, ContentError, Registry, StableId},
    lua::MOD_API_VERSION,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModManifest {
    pub id: StableId,
    pub name: String,
    pub version: String,
    pub api_version: u32,
    #[serde(default)]
    pub dependencies: Vec<StableId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveredMod {
    pub root: PathBuf,
    pub manifest: ModManifest,
}

#[derive(Debug, Error)]
pub enum ModError {
    #[error("failed to enumerate mod directory {path}: {source}")]
    ReadDir { path: PathBuf, source: io::Error },
    #[error("failed to read mod manifest {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("invalid mod manifest {path}: {source}")]
    Manifest {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("mod {mod_id} requires API version {received}, engine provides {supported}")]
    ApiVersion {
        mod_id: StableId,
        received: u32,
        supported: u32,
    },
    #[error("duplicate mod ID {0}")]
    Duplicate(StableId),
    #[error("mod {mod_id} is missing dependency {dependency}")]
    MissingDependency {
        mod_id: StableId,
        dependency: StableId,
    },
    #[error("mod dependency cycle contains: {0:?}")]
    DependencyCycle(Vec<String>),
}

pub fn discover_mods(root: &Path) -> Result<Vec<DiscoveredMod>, ModError> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut directories = Vec::new();
    for entry in fs::read_dir(root).map_err(|source| ModError::ReadDir {
        path: root.to_path_buf(),
        source,
    })? {
        let path = entry
            .map_err(|source| ModError::ReadDir {
                path: root.to_path_buf(),
                source,
            })?
            .path();
        if path.is_dir() {
            directories.push(path);
        }
    }
    directories.sort();
    let mut mods = BTreeMap::new();
    for directory in directories {
        let path = directory.join("manifest.json");
        let text = fs::read_to_string(&path).map_err(|source| ModError::Read {
            path: path.clone(),
            source,
        })?;
        let manifest: ModManifest =
            serde_json::from_str(&text).map_err(|source| ModError::Manifest {
                path: path.clone(),
                source,
            })?;
        if manifest.api_version != MOD_API_VERSION {
            return Err(ModError::ApiVersion {
                mod_id: manifest.id,
                received: manifest.api_version,
                supported: MOD_API_VERSION,
            });
        }
        if mods.contains_key(&manifest.id) {
            return Err(ModError::Duplicate(manifest.id));
        }
        mods.insert(
            manifest.id.clone(),
            DiscoveredMod {
                root: directory,
                manifest,
            },
        );
    }
    dependency_order(mods)
}

fn dependency_order(
    mut pending: BTreeMap<StableId, DiscoveredMod>,
) -> Result<Vec<DiscoveredMod>, ModError> {
    let available: BTreeSet<_> = pending.keys().cloned().collect();
    for discovered in pending.values() {
        if let Some(dependency) = discovered
            .manifest
            .dependencies
            .iter()
            .find(|dependency| !available.contains(*dependency))
        {
            return Err(ModError::MissingDependency {
                mod_id: discovered.manifest.id.clone(),
                dependency: dependency.clone(),
            });
        }
    }
    let mut loaded = BTreeSet::new();
    let mut ordered = Vec::new();
    loop {
        let ready = pending
            .iter()
            .find(|(_, discovered)| {
                discovered
                    .manifest
                    .dependencies
                    .iter()
                    .all(|dependency| loaded.contains(dependency))
            })
            .map(|(id, _)| id.clone());
        match ready {
            Some(id) => {
                let discovered = pending.remove(&id).expect("selected pending mod exists");
                loaded.insert(id);
                ordered.push(discovered);
            }
            None if pending.is_empty() => return Ok(ordered),
            None => {
                return Err(ModError::DependencyCycle(
                    pending.keys().map(|id| id.as_str().to_owned()).collect(),
                ))
            }
        }
    }
}

pub fn load_mod_content(path: &Path) -> Result<Registry, ContentError> {
    load_json_registry(&path.join("content"))
}
