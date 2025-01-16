use serde::{ Deserialize, Serialize };
use std::hash::Hash;

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::id::AuthorId;

#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct AuthorAlias {
    pub source: String,
    pub target: AuthorId,
}
