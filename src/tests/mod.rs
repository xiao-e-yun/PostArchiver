//! Comprehensive test suite for post-archiver
//!
//! This module provides unit tests for all core functionality including:
//! - Core data structures and ID types
//! - Manager database operations
//! - Importer functionality
//! - Common test utilities

pub mod common;
pub mod core;

#[cfg(feature = "utils")]
pub mod manager;

#[cfg(feature = "importer")]
pub mod importer;
