#![cfg_attr(docsrs, feature(doc_cfg))]
//! Post archiving and management system
//!
//! # Overview
//! This crate provides functionality for managing and archiving posts from various platforms,
//! with support for authors, tags, files, and comments. It implements a flexible data model
//! that can handle different content types and maintain relationships between entities.
//!
//! # Features
//! - `utils`: Enables utility functions and manager functionality
//! - `importer`: Enables post importing capabilities
//! - `typescript`: Generates TypeScript type definitions
//!
//! # Core Types
//! The system is built around several core types:
//! - [`Author`]: Content creators with optional aliases
//! - [`Post`]: Content entries that can contain text and files
//! - [`Tag`]: Categorical labels for content organization
//! - [`FileMeta`]: File metadata and storage management
//! - [`Platform`]: Information about the platform from which posts are archived
//! - [`Comment`]: Nested discussion threads

pub mod alias;
pub use alias::*;

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
pub mod utils;

#[cfg(feature = "utils")]
pub mod manager;

#[cfg(feature = "importer")]
pub mod importer;

#[cfg(test)]
mod tests;
