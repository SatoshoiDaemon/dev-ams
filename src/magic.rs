use crate::{actions::ActionRegistry, glyphs::GlyphRegistry};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpellTemplateDefinition {
    pub id: String,
    pub name: String,
    pub race_id: String,
    #[serde(default)]
    pub nodes: Vec<String>,
    #[serde(default)]
    pub glyph_ids: Vec<String>,
    #[serde(default)]
    pub demonstration_action_id: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct SpellTemplateRegistry {
    definitions: BTreeMap<String, SpellTemplateDefinition>,
}

#[derive(Debug, Error)]
pub enum SpellTemplateError {
    #[error("failed to read spell templates {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("invalid spell templates {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("duplicate spell template ID {0}")]
    Duplicate(String),
    #[error("spell template {template_id} references unknown action {action_id}")]
    UnknownAction {
        template_id: String,
        action_id: String,
    },
    #[error("spell template {template_id} references unknown Glyph {glyph_id}")]
    UnknownGlyph {
        template_id: String,
        glyph_id: String,
    },
}

impl SpellTemplateRegistry {
    pub fn load(
        path: &Path,
        actions: &ActionRegistry,
        glyphs: &GlyphRegistry,
    ) -> Result<Self, SpellTemplateError> {
        let text = fs::read_to_string(path).map_err(|source| SpellTemplateError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let definitions: Vec<SpellTemplateDefinition> =
            serde_json::from_str(&text).map_err(|source| SpellTemplateError::Json {
                path: path.to_path_buf(),
                source,
            })?;
        let mut registry = Self::default();
        for definition in definitions {
            if let Some(action_id) = &definition.demonstration_action_id {
                if actions.get(action_id).is_none() {
                    return Err(SpellTemplateError::UnknownAction {
                        template_id: definition.id,
                        action_id: action_id.clone(),
                    });
                }
            }
            if let Some(glyph_id) = definition
                .glyph_ids
                .iter()
                .find(|glyph_id| glyphs.get(glyph_id).is_none())
            {
                return Err(SpellTemplateError::UnknownGlyph {
                    template_id: definition.id,
                    glyph_id: glyph_id.clone(),
                });
            }
            let id = definition.id.clone();
            if registry
                .definitions
                .insert(id.clone(), definition)
                .is_some()
            {
                return Err(SpellTemplateError::Duplicate(id));
            }
        }
        Ok(registry)
    }

    pub fn get(&self, id: &str) -> Option<&SpellTemplateDefinition> {
        self.definitions.get(id)
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}
