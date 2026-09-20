use serde::{Deserialize, Serialize};
use crate::content::StableId;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)] pub struct Entity { pub id: StableId, pub name: String, pub hp: i32, pub tenacity: i32 }
