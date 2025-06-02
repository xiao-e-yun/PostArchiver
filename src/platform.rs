use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::PlatformId;

pub const UNKNOWN_PLATFORM: PlatformId = PlatformId(0);

/// A label that can be applied to posts
///
/// # Safety
/// - Name must not be empty
/// - Name should be kebab-case
/// - Name can be chained (e.g. "x:y:z")
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Platform {
    pub id: PlatformId,
    pub name: String,
}

impl Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Hash for Platform {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Platform {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
    }
}
