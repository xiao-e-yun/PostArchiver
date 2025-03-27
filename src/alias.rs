use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::id::AuthorId;

/// Represents an alias mapping for an author
///
/// Maps alternative names or identifiers to a unique author ID in the system,
/// useful for handling cases where the same author might have different names
/// 
/// # Structure
/// `source` is the alias name as `platform:name`,  
/// `target` is the unique author ID
/// 
/// # Relationships
/// [`Author`](crate::author::Author) - Represents the author associated with the alias
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Alias {
    /// The alias name
    /// 
    /// Should be unique across all aliases,  
    /// and follow the format  `[platform]:[name]`.  
    /// 
    /// ## Platform
    /// short name of the platform,  
    /// such as `github`, `x`, `fanbox`, etc.
    /// 
    /// ## Name
    /// The name of the author on the platform,  
    /// It should be never changed.
    /// 
    /// 1. short name (e.g. `octocat`, `jack`)
    /// 2. id (e.g. `18623`, `11g978qh2ki-1hhf98aq9533a`)
    /// 3. full name (e.g. `The Octocat`, `Jack Dorsey`)
    /// 
    /// ## Example
    /// `github:octocat`, `x:jack`.
    /// 
    pub source: String,
    pub target: AuthorId,
}