use std::hash::Hash;

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::PlatformId;

pub const UNKNOWN_PLATFORM: PlatformId = PlatformId(0);

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

#[cfg(feature = "utils")]
crate::utils::macros::as_table! {
    Platform {
        id: "id",
        name: "name",
    }
}
