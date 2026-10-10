use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StableId(String);
impl StableId {
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        let mut parts = value.split(':');
        let namespace = parts.next().unwrap_or_default();
        let local = parts.next().unwrap_or_default();
        if namespace.is_empty() || local.is_empty() || parts.next().is_some() {
            return Err(format!(
                "stable ID must contain exactly one separator as namespace:name, got {value:?}"
            ));
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl std::fmt::Display for StableId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}
impl Serialize for StableId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}
impl<'de> Deserialize<'de> for StableId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}
#[derive(Debug, Error)]
pub enum ContentError {
    #[error("failed to enumerate content directory {path}: {source}")]
    ReadDir { path: PathBuf, source: io::Error },
    #[error("failed to read content file {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("invalid JSON in content file {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("content file {path} has invalid stable id: {reason}")]
    Id { path: PathBuf, reason: String },
    #[error("content file {path} duplicates stable ID {id}")]
    Duplicate { path: PathBuf, id: StableId },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContentRecord {
    pub id: StableId,
    #[serde(flatten)]
    pub fields: BTreeMap<String, Value>,
}
#[derive(Clone, Debug, Default)]
pub struct Registry {
    records: BTreeMap<StableId, ContentRecord>,
}
impl Registry {
    pub fn insert(&mut self, record: ContentRecord) -> Result<(), StableId> {
        if self.records.contains_key(&record.id) {
            return Err(record.id);
        }
        self.records.insert(record.id.clone(), record);
        Ok(())
    }
    pub fn get(&self, id: &StableId) -> Option<&ContentRecord> {
        self.records.get(id)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&StableId, &ContentRecord)> {
        self.records.iter()
    }
    pub fn len(&self) -> usize {
        self.records.len()
    }
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}
pub fn load_json_registry(root: &Path) -> Result<Registry, ContentError> {
    let mut files = Vec::new();
    collect_json(root, &mut files)?;
    files.sort();
    let mut registry = Registry::default();
    for path in files {
        let text = fs::read_to_string(&path).map_err(|source| ContentError::Read {
            path: path.clone(),
            source,
        })?;
        let value: Value = serde_json::from_str(&text).map_err(|source| ContentError::Json {
            path: path.clone(),
            source,
        })?;
        let id = value
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| ContentError::Id {
                path: path.clone(),
                reason: "missing string field id".into(),
            })?;
        let id = StableId::new(id).map_err(|reason| ContentError::Id {
            path: path.clone(),
            reason,
        })?;
        let fields = value
            .as_object()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect();
        registry
            .insert(ContentRecord { id, fields })
            .map_err(|id| ContentError::Duplicate {
                path: path.clone(),
                id,
            })?;
    }
    Ok(registry)
}
fn collect_json(root: &Path, output: &mut Vec<PathBuf>) -> Result<(), ContentError> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root).map_err(|source| ContentError::ReadDir {
        path: root.to_path_buf(),
        source,
    })? {
        let path = entry
            .map_err(|source| ContentError::ReadDir {
                path: root.to_path_buf(),
                source,
            })?
            .path();
        if path.is_dir() {
            collect_json(&path, output)?;
        } else if path.extension().and_then(|x| x.to_str()) == Some("json") {
            output.push(path);
        }
    }
    Ok(())
}
