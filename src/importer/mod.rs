//! Module for importing data from various sources.
//!
//! This module contains the necessary structures and functions to handle the import of data
//!
//! It includes two methods for importing data:
//!   1. object like (recommended)
//!     It is high level and easy to use.
//!   2. function like
//!     It is low level and more flexible.
//!
//! # Examples
//! ```rust
//! use post_archiver::manager::PostArchiverManager;
//! use post_archiver::importer::{UnsyncAuthor, UnsyncPost, UnsyncFileMeta, UnsyncContent, ImportFileMetaMethod};
//! use post_archiver::Link;
//! use chrono::Utc;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Open a connection to the archive
//!     let manager = PostArchiverManager::open_in_memory()?;
//!     
//!     // Create and import an author
//!     let (author, _) = UnsyncAuthor::new("octocat".to_string())
//!         .alias(vec!["github:octocat".to_string()])
//!         .links(vec![Link::new("github", "https://github.com/octocat")])
//!         .updated(Some(Utc::now()))
//!         .sync(&manager)?;
//!     
//!     // Create and import a post with files
//!     let (post, _) = UnsyncPost::new(author.id)
//!         .title("Hello World".to_string())
//!         .content(vec![
//!             UnsyncContent::text("This is my first post!".to_string()),
//!             UnsyncContent::file(UnsyncFileMeta {
//!                 filename: "avatar.png".to_string(),
//!                 mime: "image/png".to_string(),
//!                 extra: Default::default(),
//!                 method: ImportFileMetaMethod::File("./avatar.png".into()),
//!             })
//!         ])
//!         .tags(vec!["hello".to_string(), "first-post".to_string()])
//!         .sync(&manager)?;
//!     
//!     Ok(())
//! }
//! ```
//!
//!
//!

pub mod author;
pub use author::*;

pub mod file_meta;
pub use file_meta::*;

pub mod post;
pub use post::*;

pub mod tags;
