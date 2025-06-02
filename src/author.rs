use std::hash::Hash;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::{AuthorId, FileMetaId};

/// A content creator or contributor in the system
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Author {
    pub id: AuthorId,
    pub name: String,
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
            && self.updated == other.updated
    }
}

impl Eq for Author {}
