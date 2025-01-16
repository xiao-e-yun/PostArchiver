use std::hash::{ Hash, Hasher };

use serde::{ Deserialize, Serialize };

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::id::PostSourceId;

#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Tag {
    pub id: PostSourceId,
    pub name: String,
}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
    }
}

impl Eq for Tag {}
