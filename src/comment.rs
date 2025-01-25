use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript")]
use ts_rs::TS;

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
