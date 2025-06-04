use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::{id::TagId, PlatformId, PostId};

/// A label that can be applied to posts
///
/// # Safety
/// - Name must not be empty
/// - Name should be kebab-case
/// - Name can be chained (e.g. "x:y:z")
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub id: TagId,
    pub name: String,
    pub platform: Option<PlatformId>,
}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[cfg(feature = "utils")]
crate::utils::macros::as_table! {
    Tag {
        id: "id",
        name: "name",
        platform: "platform",
    }
}
