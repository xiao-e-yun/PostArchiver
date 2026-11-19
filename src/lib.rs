#![cfg_attr(docsrs, feature(doc_cfg))]
//! Post archiving and management system
//!
//! # Overview
//! This crate provides functionality for managing and archiving posts from various platforms,
//! with support for authors, tags, files, and comments. It implements a flexible data model
//! that can handle different content types and maintain relationships between entities.
//!
//! # Features
//! - `utils` (default): the SQLite-backed [`manager`] and [`query`] APIs. Without it the crate
//!   only provides the data types.
//! - `importer`: the [`importer`] module for importing scraped data (implies `utils`).
//! - `typescript`: derives TypeScript type definitions with `ts-rs`.
//!
//! # Storage
//! An archive is a directory holding `post-archiver.db` and one directory per post for its
//! files (see [`Post::directory`]). The database is opened with foreign keys enforced and in
//! WAL mode, so `post-archiver.db-wal` / `post-archiver.db-shm` may sit next to it while it is
//! open. Copy all three (or close every connection first) when backing up an archive.
//!
//! Archives created by older versions are upgraded with `post-archiver migrate` from `post-archiver-tools`.
//!
//! # Core Types
//! The system is built around several core types:
//! - [`Author`]: Content creators with optional aliases
//! - [`Alias`]: Alternative names for authors
//! - [`Post`]: Content entries that can contain text and files
//! - [`Tag`]: Categorical labels for content organization
//! - [`Collection`]: Grouping of posts, authors, and tags
//! - [`FileMeta`]: File metadata and storage management
//! - [`Platform`]: Information about the platform from which posts are archived
//! - [`Comment`]: Nested discussion threads

pub mod author;
pub use author::*;

pub mod collection;
pub use collection::*;

pub mod comment;
pub use comment::*;

pub mod file_meta;
pub use file_meta::*;

pub mod id;
pub use id::*;

pub mod post;
pub use post::*;

pub mod platform;
pub use platform::*;

pub mod tag;
pub use tag::*;

pub mod content;
pub use content::*;

#[cfg(feature = "utils")]
pub mod error;

#[cfg(feature = "utils")]
pub mod utils;

#[cfg(feature = "utils")]
pub mod manager;

#[cfg(feature = "utils")]
pub mod query;

#[cfg(feature = "importer")]
pub mod importer;
