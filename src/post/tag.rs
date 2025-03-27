use serde::{Deserialize, Serialize};

use crate::{PostId, PostTagId};

#[cfg(feature = "typescript")]
use ts_rs::TS;

/// Represents post and tag relationship
///
/// # Structure
/// `post` id of the post  
/// `tag` id of the tag  
/// 
/// # Relationships
/// [`Post`](crate::post::Post) - Represents a post that the tag belongs to  
/// [`Tag`](crate::tag::Tag) - Represents a tag that the post belongs to
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PostTag {
    pub post: PostId,
    pub tag: PostTagId,
}
