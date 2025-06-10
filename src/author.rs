use std::hash::Hash;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::{AuthorId, FileMetaId, PostId};

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

/// Association type that creates a many-to-many relationship between posts and tags
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthorPost {
    pub author: AuthorId,
    pub post: PostId,
}

#[cfg(feature = "utils")]
crate::utils::macros::as_table! {
    Author {
        id: "id",
        name: "name",
        thumb: "thumb",
        updated: "updated",
    }

    AuthorPost {
        author: "author",
        post: "post",
    }
}
