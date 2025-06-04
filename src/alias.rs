use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::{AuthorId, PlatformId};
/// A mapping between an alternative author name and their canonical identifier
///
/// - Links to [`Author`](crate::author::Author) through the target ID
/// - Used by importer modules for author resolution
/// - Referenced in post management for author lookups
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
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

impl Hash for Alias {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.source.hash(state);
        self.platform.hash(state);
        self.target.hash(state);
    }
}

#[cfg(feature = "utils")]
crate::utils::macros::as_table! {
    Alias {
        source: "source",
        platform: "platform",
        target: "target",
        link: "link",
    }
}
