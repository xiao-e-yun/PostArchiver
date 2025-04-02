use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::id::FileMetaId;

/// A content segment within a post that can be either text or a file reference
///
/// - Used within [`Post`](crate::post::Post) content arrays
/// - References [`FileMeta`](crate::file_meta::FileMeta) through FileMetaId
///
/// # Variants:
/// - Text: Contains markdown-formatted text content
/// - File: References external file content via FileMetaId
///
/// # Processing:
/// - Text content should be rendered as markdown
/// - File content requires metadata lookup and appropriate handling
///
/// # Examples
/// ```rust
/// use post_archiver::{Content, FileMetaId};
///
/// // Text content with markdown
/// let text = Content::Text("# Heading\n\nSome **bold** text".to_string());
///
/// // File reference
/// let image = Content::File(FileMetaId::new(1));
///
/// // Mixed content post
/// let contents = vec![
///     Content::Text("Introduction:".to_string()),
///     Content::File(FileMetaId::new(1)),
///     Content::Text("*Caption for the above image*".to_string())
/// ];
/// ```
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum Content {
    /// Markdown-formatted text content
    ///
    /// The text content should be processed as markdown when rendering,
    Text(String),
    /// Reference to a file via its metadata ID
    File(FileMetaId),
}
