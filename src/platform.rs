use std::hash::Hash;

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::PlatformId;

/// A platform that can be used to categorize posts
///
/// # Safety
/// - Name must not be empty
/// - Name should be kebab-case
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Platform {
    pub id: PlatformId,
    pub name: String,
}

impl Platform {
    pub const UNKNOWN: PlatformId = PlatformId(0);
}

#[cfg(feature = "utils")]
crate::utils::macros::as_table! {
    Platform {
        id: "id",
        name: "name",
    }
}
