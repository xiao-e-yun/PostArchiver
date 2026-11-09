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

use std::{ops::Deref, path::Path};

pub use author::*;

pub mod file_meta;
pub use file_meta::*;

pub mod post;
pub use post::*;

pub mod collection;
pub use collection::*;

pub mod tag;
use rusqlite::{Connection, Result, Savepoint, Transaction};
pub use tag::*;

use crate::manager::PostArchiverManager;

pub mod platform;

impl PostArchiverManager {
    /// Create an importer for importing data into the archive.
    pub fn importer(&mut self) -> Importer<'_, &mut Connection> {
        Importer {
            conn: &mut self.conn,
            path: self.path.as_path(),
        }
    }
}

#[derive(Debug)]
pub struct Importer<'a, C> {
    pub path: &'a Path,
    pub conn: C,
}

impl<'a, C: SavepointExt> Importer<'a, C> {
    pub fn savepoint(&mut self) -> Result<Importer<'a, Savepoint<'_>>> {
        Ok(Importer {
            conn: self.conn.savepoint()?,
            path: self.path,
        })
    }
}

pub trait SavepointExt: Deref<Target = Connection> {
    fn savepoint(&mut self) -> Result<Savepoint<'_>>;
}
impl SavepointExt for &mut Connection {
    fn savepoint(&mut self) -> Result<Savepoint<'_>> {
        Savepoint::new(self)
    }
}
impl SavepointExt for Transaction<'_> {
    fn savepoint(&mut self) -> Result<Savepoint<'_>> {
        self.savepoint()
    }
}
impl SavepointExt for Savepoint<'_> {
    fn savepoint(&mut self) -> Result<Savepoint<'_>> {
        self.savepoint()
    }
}

impl<'a> Importer<'a, Savepoint<'_>> {
    pub fn commit(self) -> Result<()> {
        self.conn.commit()?;
        Ok(())
    }
}
