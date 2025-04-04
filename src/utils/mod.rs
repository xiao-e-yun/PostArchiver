//! Utility functions and constants for the post archiver system
//!
//! # Overview
//! This module provides common utilities and configuration constants used
//! throughout the post archiver system. It includes database configuration,

pub mod author;
pub mod file_meta;
pub mod id;
pub mod post;

/// Default relative path to the SQLite database file
///
/// # Usage
/// This constant defines the default location where the post archiver
/// will create and access its SQLite database. The path is relative
/// to the current working directory.
pub const DATABASE_NAME: &str = "post-archiver.db";

/// Current version of the post archiver library
///
/// # Details
/// - Automatically set from Cargo.toml at build time
/// - Follows semantic versioning (MAJOR.MINOR.PATCH)
/// - Used for version checking and compatibility
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
