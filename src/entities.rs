use crate::{
    attributes::{AttributeRules, AttributeSet},
    content::StableId,
    effects::StatusCollection,
    numeric::GameInt,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Allegiance {
    Player,
    Enemy,
    Neutral,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourcePool {
    pub current: GameInt,
    pub maximum: GameInt,
}

impl ResourcePool {
    pub fn full(maximum: GameInt) -> Self {
        Self {
            current: maximum.max(0),
            maximum: maximum.max(0),
        }
    }
    pub fn reduce(&mut self, requested: GameInt) -> GameInt {
        let applied = requested.max(0).min(self.current);
        self.current -= applied;
        applied
    }
    pub fn restore(&mut self, requested: GameInt) -> GameInt {
        let applied = requested
            .max(0)
            .min(self.maximum.saturating_sub(self.current));
        self.current += applied;
        applied
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entity {
    pub id: StableId,
    pub name: String,
    pub allegiance: Allegiance,
    pub attributes: AttributeSet,
    pub hp: ResourcePool,
    pub tenacity: ResourcePool,
    pub mana: ResourcePool,
    pub shield: GameInt,
    pub defense: GameInt,
    pub equipment_priority: GameInt,
    pub status_priority: GameInt,
    pub statuses: StatusCollection,
    pub active: bool,
}

impl Entity {
    pub fn new(
        id: StableId,
        name: impl Into<String>,
        allegiance: Allegiance,
        attributes: AttributeSet,
        rules: &AttributeRules,
    ) -> Result<Self, crate::attributes::AttributeError> {
        let derived = rules.derive(&attributes)?;
        Ok(Self {
            id,
            name: name.into(),
            allegiance,
            attributes,
            hp: ResourcePool::full(derived.max_hp),
            tenacity: ResourcePool::full(derived.max_tenacity),
            mana: ResourcePool::full(derived.max_mana),
            shield: 0,
            defense: 0,
            equipment_priority: 0,
            status_priority: 0,
            statuses: StatusCollection::default(),
            active: true,
        })
    }
    pub fn is_alive(&self) -> bool {
        self.active && self.hp.current > 0
    }
}
