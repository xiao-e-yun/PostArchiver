use serde::{Deserialize, Serialize};

use crate::{PostId, PostTagId};

#[cfg(feature = "typescript")]
use ts_rs::TS;

#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PostTag {
    pub post: PostId,
    pub tag: PostTagId,
}
