use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::id::FileMetaId;

/// Represents the content of a post
/// 
/// # Variants
/// - `Text(String)`  
///    Represents a `markdown` text
///
/// - `File(FileMetaId)`  
///    Represents a file id that is referenced in the post
///
/// # Relationships
/// [`FileMetaId`](crate::id::FileMetaId) - Represents a file that is referenced in the post
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum Content {
    /// Represents a markdown text
    ///
    /// # Transform
    /// It should render the text using `markdown`
    Text(String),
    /// Represents a file id that is referenced in the post
    /// 
    /// # Transform
    /// It should get the file metadata from `file_metas` using the id
    File(FileMetaId),
}
