use std::hash::Hash;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::{
    id::{AuthorId, FileMetaId},
    link::Link,
};

/// Represents a author
/// 
/// # Structure
/// `id` Unique identifier for the author  
/// `name` Name of the author  
/// `links` Collection of relevant links associated with the author  
/// `thumb` Optional thumbnail/avatar image reference  
/// `updated` Timestamp of when the author information was last updated  
/// 
/// # Relationships
/// [`Alias`](crate::alias::Alias) - Represents an alias mapping for an author  
/// [`Post`](crate::post::Post) - Represents a post created by the author
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
