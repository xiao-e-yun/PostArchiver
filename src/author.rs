use std::hash::Hash;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::{AuthorId, FileMetaId, PlatformId, PostId};

/// A content creator or contributor in the system
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Author {
    pub id: AuthorId,
    pub name: String,
    pub thumb: Option<FileMetaId>,
    pub updated: DateTime<Utc>,
}

/// A mapping between an alternative author name and their canonical identifier
///
/// - Links to [`Author`] through the target ID
/// - Used by importer modules for author resolution
/// - Referenced in post management for author lookups
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Alias {
    /// The alias name
    ///
    /// Should be unique across all aliases,  
    /// The name of the author on the platform,  
    /// It should be never changed.
    ///
    /// 1. id (e.g. `18623`, `11g978qh2ki-1hhf98aq9533a`)
    /// 2. short name (e.g. `octocat`, `jack`)
    /// 3. full name (e.g. `The Octocat`, `Jack Dorsey`)
    ///
    pub source: String,
    /// The platform this alias belongs to
    pub platform: PlatformId,
    /// The target author ID this alias maps to
    pub target: AuthorId,
    /// A link to the author's profile on the platform
    pub link: Option<String>,
}

/// Association type that creates a many-to-many relationship between posts and tags
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthorPost {
    pub author: AuthorId,
    pub post: PostId,
}

#[cfg(feature = "utils")]
mod definitions {
    use crate::utils::macros::as_table;

    use super::*;

    as_table! {
        "authors" => Author(AuthorDefinitions) {
            column id: AuthorId,
            column name: String,
            column thumb: Option<FileMetaId>,
            column updated: DateTime<Utc>,
            relation author_posts(author => post) as PostId,
            relation author_aliases(target as {
                source: "source",
                platform: "platform",
                target: "target",
                link: "link"
            }) as Alias,
        }

        "author_posts" => AuthorPost(AuthorPostDefinitions) {
            column author: AuthorId,
            column post: PostId,
        }

        "author_aliases" => Alias(AliasDefinitions) {
            column source: String,
            column platform: PlatformId,
            column target: AuthorId,
            column link: Option<String>,
        }
    }
}
