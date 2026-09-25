use crate::{
    actions::ActionRegistry,
    attributes::{Attribute, AttributeRules, AttributeSet},
    content::StableId,
    entities::{Allegiance, Entity},
    numeric::GameInt,
    session::CombatSession,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityDefinition {
    pub id: String,
    pub name: String,
    pub allegiance: Allegiance,
    #[serde(default)]
    pub race_id: Option<String>,
    #[serde(default)]
    pub tags: BTreeSet<String>,
    #[serde(default)]
    pub attributes: BTreeMap<String, GameInt>,
    #[serde(default)]
    pub defense: GameInt,
    #[serde(default)]
    pub action_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncounterActor {
    pub instance_id: String,
    pub entity_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncounterDefinition {
    pub id: String,
    pub actors: Vec<EncounterActor>,
}

#[derive(Clone, Debug, Default)]
pub struct EncounterRegistry {
    entities: BTreeMap<String, EntityDefinition>,
    encounters: BTreeMap<String, EncounterDefinition>,
}

#[derive(Debug, thiserror::Error)]
pub enum EncounterError {
    #[error("failed to read encounter content {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("invalid encounter content {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("duplicate entity or encounter ID {0}")]
    Duplicate(String),
    #[error("unknown encounter {0}")]
    UnknownEncounter(String),
    #[error("encounter {encounter_id} references unknown entity {entity_id}")]
    UnknownEntity {
        encounter_id: String,
        entity_id: String,
    },
    #[error("invalid stable ID {id}: {reason}")]
    StableId { id: String, reason: String },
    #[error("invalid attribute {attribute} in entity {entity_id}")]
    Attribute {
        entity_id: String,
        attribute: String,
    },
    #[error("invalid attributes in entity {entity_id}: {reason}")]
    AttributeValue { entity_id: String, reason: String },
    #[error("entity template {entity_id} references unknown action {action_id}")]
    UnknownAction {
        entity_id: String,
        action_id: String,
    },
}

impl EncounterRegistry {
    pub fn load(content_root: &Path) -> Result<Self, EncounterError> {
        let entities_path = content_root.join("entities.json");
        let encounters_path = content_root.join("encounters.json");
        let entities: Vec<EntityDefinition> = read_json(&entities_path)?;
        let encounters: Vec<EncounterDefinition> = read_json(&encounters_path)?;
        let mut registry = Self::default();
        for definition in entities {
            if registry
                .entities
                .insert(definition.id.clone(), definition)
                .is_some()
            {
                return Err(EncounterError::Duplicate("entity".into()));
            }
        }
        for definition in encounters {
            if registry
                .encounters
                .insert(definition.id.clone(), definition)
                .is_some()
            {
                return Err(EncounterError::Duplicate("encounter".into()));
            }
        }
        for encounter in registry.encounters.values() {
            for actor in &encounter.actors {
                if !registry.entities.contains_key(&actor.entity_id) {
                    return Err(EncounterError::UnknownEntity {
                        encounter_id: encounter.id.clone(),
                        entity_id: actor.entity_id.clone(),
                    });
                }
            }
        }
        Ok(registry)
    }

    pub fn build_session(
        &self,
        encounter_id: &str,
        rules: &AttributeRules,
    ) -> Result<CombatSession, EncounterError> {
        let encounter = self
            .encounters
            .get(encounter_id)
            .ok_or_else(|| EncounterError::UnknownEncounter(encounter_id.into()))?;
        let mut entities = Vec::new();
        let mut actions = BTreeMap::new();
        for actor in &encounter.actors {
            let definition = self.entities.get(&actor.entity_id).ok_or_else(|| {
                EncounterError::UnknownEntity {
                    encounter_id: encounter_id.into(),
                    entity_id: actor.entity_id.clone(),
                }
            })?;
            let mut attributes = AttributeSet::default();
            for (name, value) in &definition.attributes {
                let attribute =
                    Attribute::from_str(name).map_err(|_| EncounterError::Attribute {
                        entity_id: definition.id.clone(),
                        attribute: name.clone(),
                    })?;
                attributes.set(attribute, *value).map_err(|error| {
                    EncounterError::AttributeValue {
                        entity_id: definition.id.clone(),
                        reason: error.to_string(),
                    }
                })?;
            }
            let id =
                StableId::new(&actor.instance_id).map_err(|reason| EncounterError::StableId {
                    id: actor.instance_id.clone(),
                    reason,
                })?;
            let mut entity = Entity::new(
                id,
                &definition.name,
                definition.allegiance,
                attributes,
                rules,
            )
            .map_err(|error| EncounterError::AttributeValue {
                entity_id: definition.id.clone(),
                reason: error.to_string(),
            })?;
            entity.defense = definition.defense;
            entity.tags = definition.tags.clone();
            entity.race_id = definition
                .race_id
                .as_ref()
                .map(|race| {
                    StableId::new(race).map_err(|reason| EncounterError::StableId {
                        id: race.clone(),
                        reason,
                    })
                })
                .transpose()?;
            actions.insert(actor.instance_id.clone(), definition.action_ids.clone());
            entities.push(entity);
        }
        let mut session = CombatSession::new(encounter_id, entities);
        for (entity_id, action_ids) in actions {
            session.set_available_actions(&entity_id, action_ids);
        }
        Ok(session)
    }

    pub fn validate(&self, actions: &ActionRegistry) -> Result<(), EncounterError> {
        for entity in self.entities.values() {
            for action_id in &entity.action_ids {
                if actions.get(action_id).is_none() {
                    return Err(EncounterError::UnknownAction {
                        entity_id: entity.id.clone(),
                        action_id: action_id.clone(),
                    });
                }
            }
        }
        Ok(())
    }
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, EncounterError> {
    let text = fs::read_to_string(path).map_err(|source| EncounterError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| EncounterError::Json {
        path: path.to_path_buf(),
        source,
    })
}
