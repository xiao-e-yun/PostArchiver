use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::id::AuthorId;
/// A mapping between an alternative author name and their canonical identifier
///
/// - Links to [`Author`](crate::author::Author) through the target ID
/// - Used by importer modules for author resolution
/// - Referenced in post management for author lookups
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
    /// 1. id (e.g. `18623`, `11g978qh2ki-1hhf98aq9533a`)
    /// 2. short name (e.g. `octocat`, `jack`)
    /// 3. full name (e.g. `The Octocat`, `Jack Dorsey`)
    ///
    /// ## Examples
    /// `github:octocat`, `x:jack`.
    ///
    pub source: String,
    pub target: AuthorId,
}

impl Alias {
    /// Get the platform and name from the alias
    ///
    /// # Examples
    /// ```rust
    /// # use post_archiver::{Alias, AuthorId};
    ///
    /// let alias = Alias {
    ///    source: "github:octocat".to_string(),
    ///    target: AuthorId::new(1)
    /// };
    ///
    /// let (platform, name) = alias.platform_and_name();
    /// assert_eq!(platform, "github");
    /// assert_eq!(name, "octocat");
    /// ```
    pub fn platform_and_name(&self) -> (String, String) {
        let (platform, name) = self.source.split_once(':').unwrap_or_default();
        (platform.to_string(), name.to_string())
    }

    /// Get the platform from the alias
    ///
    /// # Examples
    /// ```rust
    /// # use post_archiver::{Alias, AuthorId};
    ///
    /// let alias = Alias {
    ///    source: "github:octocat".to_string(),
    ///    target: AuthorId::new(1)
    /// };
    ///
    /// assert_eq!(alias.platform(), "github");
    /// ```
    pub fn platform(&self) -> String {
        self.platform_and_name().0
    }

    /// Get the name from the alias
    ///
    /// # Examples
    /// ```rust
    /// # use post_archiver::{Alias, AuthorId};
    ///
    /// let alias = Alias {
    ///    source: "github:octocat".to_string(),
    ///    target: AuthorId::new(1)
    /// };
    ///
    /// assert_eq!(alias.name(), "octocat");
    pub fn name(&self) -> String {
        self.platform_and_name().1
    }
}
