use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript")]
use ts_rs::TS;

/// Represents a comment on a post
///
/// # Structure
/// `user` Name of the user who made the comment  
/// `text` Content of the comment  
/// `replies` nested comments (replies)  
///
/// # Relationships
/// [`Post`](crate::post::Post) - Represents a post that the comment belongs to
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, Hash)]
pub struct Comment {
    pub user: String,
    pub text: String,
    #[cfg_attr(feature = "typescript", ts(as = "Option<Vec<Comment>>", optional))]
    #[serde(skip_serializing_if = "<[_]>::is_empty", default)]
    pub replies: Vec<Comment>,
}
