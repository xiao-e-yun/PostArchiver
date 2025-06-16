//! Module for importing data from various sources.
//!
//! This module contains the necessary structures and functions to handle the import of data
//!
//! It includes two methods for importing data:
//!   1. object like (recommended)
//!      It is high level and easy to use.
//!   2. function like
//!      It is low level and more flexible.

pub mod author;
pub use author::*;

pub mod file_meta;
pub use file_meta::*;

pub mod post;
pub use post::*;

pub mod collection;
pub use collection::*;

pub mod tag;
pub use tag::*;

pub mod platform;
