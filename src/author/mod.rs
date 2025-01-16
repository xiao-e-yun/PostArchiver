pub mod alias;

use std::hash::Hash;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::{
    id::{AuthorId, FileMetaId},
    link::Link,
};

#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Author {
    pub id: AuthorId,
    pub name: String,
    pub links: Vec<Link>,
    pub thumb: Option<FileMetaId>,
    pub updated: DateTime<Utc>,
}

impl Hash for Author {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Author {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.name == other.name
            && self.thumb == other.thumb
            && self.links == other.links
    }
}

impl Eq for Author {}
