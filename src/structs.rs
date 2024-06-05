use std::{collections::HashSet, hash::Hash, path::PathBuf};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};


#[cfg(feature = "typescript")]
use ts_rs::TS;

#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ArchiveAuthorsList(pub Vec<ArchiveAuthorsItem>);

#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ArchiveAuthorsItem {
    pub id: String,
    pub name: String,
    #[cfg_attr(feature = "typescript", ts(optional))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb: Option<PathBuf>,
    pub from: HashSet<ArchiveFrom>,
}

#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveAuthor {
    pub id: String,
    pub name: String,
    pub from: HashSet<ArchiveFrom>,
    pub posts: Vec<ArchivePostShort>,
    #[cfg_attr(feature = "typescript", ts(optional))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb: Option<PathBuf>,
}

#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePost {
    pub id: String,
    pub title: String,
    pub author: String,
    pub from: ArchiveFrom,
    pub thumb: Option<PathBuf>,
    pub files: Vec<ArchiveFile>,
    pub updated: DateTime<Local>,
    pub published: DateTime<Local>,
    pub content: Vec<ArchiveContent>,
    pub comments: Vec<ArchiveComment>,
}

#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePostShort {
    pub id: String,
    pub url: PathBuf,
    pub title: String,
    pub author: String,
    pub from: ArchiveFrom,
    pub thumb: Option<PathBuf>,
    pub updated: DateTime<Local>,
}

#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ArchiveFile {
    Image {
        width: u32,
        height: u32,
        filename: PathBuf,
        path: PathBuf,
    },
    Video {
        filename: PathBuf,
        path: PathBuf,
    },
    File {
        filename: PathBuf,
        path: PathBuf,
    },
}

#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ArchiveFrom {
    Fanbox,
}

//MarkDown
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))] 
#[derive(Deserialize, Serialize, Debug, Clone, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ArchiveContent {
    Text(String),
    Image(String),
    Video(String),
    File(String),
}

#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveComment {
    pub user: String,
    pub text: String,
    #[cfg_attr(feature = "typescript", ts(as = "Option<Vec<ArchiveComment>>", optional))]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub replies: Vec<ArchiveComment>,
}
