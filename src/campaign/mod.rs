//! Persistent player and campaign state, independent from terminal presentation.

use crate::glyphs::{Element, SpellBlueprint};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerCharacter {
    pub race_id: String,
    #[serde(default)]
    pub elements: BTreeSet<Element>,
    #[serde(default)]
    pub spells: Vec<SpellBlueprint>,
    #[serde(default)]
    pub inventory: Vec<String>,
    #[serde(default)]
    pub equipment: BTreeMap<String, Option<String>>,
}

impl PlayerCharacter {
    pub fn validate(&self) -> Result<(), String> {
        crate::content::StableId::new(self.race_id.clone()).map_err(|reason| reason.to_string())?;
        if self.elements.len() > 2 {
            return Err("a character may select at most two elements".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RaceDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug, Default)]
pub struct RaceRegistry {
    entries: BTreeMap<String, RaceDefinition>,
}

impl RaceRegistry {
    pub fn new(entries: Vec<RaceDefinition>) -> Result<Self, String> {
        let mut registry = Self::default();
        for entry in entries {
            crate::content::StableId::new(entry.id.clone()).map_err(|reason| reason.to_string())?;
            if entry.name.trim().is_empty() || entry.description.trim().is_empty() {
                return Err(format!(
                    "race {} must have a name and description",
                    entry.id
                ));
            }
            if registry
                .entries
                .insert(entry.id.clone(), entry.clone())
                .is_some()
            {
                return Err(format!("duplicate race ID {}", entry.id));
            }
        }
        Ok(registry)
    }

    pub fn get(&self, id: &str) -> Option<&RaceDefinition> {
        self.entries.get(id)
    }
    pub fn iter(&self) -> impl Iterator<Item = &RaceDefinition> {
        self.entries.values()
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CampaignProgress {
    pub chapter_id: String,
    pub current_scene_id: Option<String>,
    pub prologue_completed: bool,
    pub day: u32,
    pub location_id: Option<String>,
    #[serde(default)]
    pub flags: BTreeSet<String>,
}

impl CampaignProgress {
    pub fn prologue(scene_id: impl Into<String>) -> Self {
        Self {
            chapter_id: "core:prologue".into(),
            current_scene_id: Some(scene_id.into()),
            prologue_completed: false,
            day: 0,
            location_id: None,
            flags: BTreeSet::new(),
        }
    }

    pub fn apply_scene_events(&mut self, events: &[String]) -> Result<(), String> {
        for event in events {
            match event.as_str() {
                "core:prologue_completed" => {
                    self.chapter_id = "core:chapter_1".into();
                    self.current_scene_id = None;
                    self.prologue_completed = true;
                }
                "core:campaign_day_7" => self.day = 7,
                "core:move_to_tidal_town" => self.location_id = Some("core:tidal_town".into()),
                _ => return Err(format!("unknown campaign completion event {event}")),
            }
            self.flags.insert(event.clone());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocationDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub destinations: BTreeSet<String>,
}

#[derive(Clone, Debug, Default)]
pub struct LocationRegistry {
    entries: BTreeMap<String, LocationDefinition>,
}

impl LocationRegistry {
    pub fn new(entries: Vec<LocationDefinition>) -> Result<Self, String> {
        let mut registry = Self::default();
        for entry in entries {
            crate::content::StableId::new(entry.id.clone())
                .map_err(|reason| format!("invalid location ID {:?}: {reason}", entry.id))?;
            if registry
                .entries
                .insert(entry.id.clone(), entry.clone())
                .is_some()
            {
                return Err(format!("duplicate location ID {:?}", entry.id));
            }
        }
        for location in registry.entries.values() {
            for destination in &location.destinations {
                if !registry.entries.contains_key(destination) {
                    return Err(format!(
                        "location {} references missing destination {destination}",
                        location.id
                    ));
                }
            }
        }
        Ok(registry)
    }

    pub fn get(&self, id: &str) -> Option<&LocationDefinition> {
        self.entries.get(id)
    }
    pub fn destinations(&self, id: &str) -> Option<Vec<&LocationDefinition>> {
        let location = self.get(id)?;
        Some(
            location
                .destinations
                .iter()
                .filter_map(|id| self.get(id))
                .collect(),
        )
    }
    pub fn can_move(&self, from: &str, to: &str) -> bool {
        self.get(from)
            .is_some_and(|location| location.destinations.contains(to))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NarrativeScene {
    pub id: String,
    pub text: String,
    #[serde(default = "default_scene_style")]
    pub style: String,
    #[serde(default)]
    pub auto_advance: bool,
    #[serde(default)]
    pub next: Option<String>,
    #[serde(default)]
    pub completion_events: Vec<String>,
}

fn default_scene_style() -> String {
    "narrative".into()
}

#[derive(Clone, Debug, Default)]
pub struct SceneRegistry {
    entries: BTreeMap<String, NarrativeScene>,
}

impl SceneRegistry {
    pub fn new(entries: Vec<NarrativeScene>) -> Result<Self, String> {
        let mut registry = Self::default();
        for entry in entries {
            crate::content::StableId::new(entry.id.clone())
                .map_err(|reason| format!("invalid scene ID {:?}: {reason}", entry.id))?;
            if registry
                .entries
                .insert(entry.id.clone(), entry.clone())
                .is_some()
            {
                return Err(format!("duplicate scene ID {:?}", entry.id));
            }
        }
        for scene in registry.entries.values() {
            if let Some(next) = &scene.next {
                if !registry.entries.contains_key(next) {
                    return Err(format!(
                        "scene {} references missing next scene {next}",
                        scene.id
                    ));
                }
            }
        }
        Ok(registry)
    }
    pub fn get(&self, id: &str) -> Option<&NarrativeScene> {
        self.entries.get(id)
    }
}

pub fn move_campaign(
    progress: &mut CampaignProgress,
    locations: &LocationRegistry,
    to: &str,
) -> Result<(), String> {
    let Some(from) = progress.location_id.as_deref() else {
        return Err("campaign has no current location".into());
    };
    if !locations.can_move(from, to) {
        return Err(format!("cannot travel from {from} to {to}"));
    }
    progress.location_id = Some(to.to_owned());
    Ok(())
}

pub fn elements() -> [Element; 13] {
    Element::ALL
}

pub fn prologue_completion_events(scenes: &SceneRegistry) -> Result<Vec<String>, String> {
    scenes
        .get("core:prologue_seven_days")
        .map(|scene| scene.completion_events.clone())
        .ok_or_else(|| "required scene core:prologue_seven_days is missing".into())
}
