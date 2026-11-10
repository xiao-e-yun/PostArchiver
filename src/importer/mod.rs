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

use std::{
    collections::HashSet,
    fs, mem,
    ops::Deref,
    path::{Path, PathBuf},
};

pub use author::*;

pub mod file_meta;
pub use file_meta::*;

pub mod post;
pub use post::*;

pub mod collection;
pub use collection::*;

pub mod tag;
use rusqlite::{Connection, Result, Savepoint};
pub use tag::*;

use crate::manager::PostArchiverManager;

pub mod platform;

impl PostArchiverManager {
    /// Create an importer for importing data into the archive.
    pub fn importer(&mut self) -> Importer<'_, 'static, &mut Connection> {
        Importer {
            conn: &mut self.conn,
            path: self.path.as_path(),
            pending: ImporterPending {
                buffer: HashSet::new(),
                parent: None,
                depth: 0,
            },
        }
    }
}

#[derive(Debug)]
pub struct Importer<'root, 'parent, C> {
    pub path: &'root Path,
    pub conn: C,
    pub pending: ImporterPending<'parent>,
}

impl<C: SavepointExt> Importer<'_, '_, C> {
    pub fn savepoint(&'_ mut self) -> Result<Importer<'_, '_, Savepoint<'_>>> {
        Ok(Importer {
            conn: self.conn.savepoint()?,
            path: self.path,
            pending: ImporterPending {
                buffer: HashSet::new(),
                parent: Some(&mut self.pending.buffer),
                depth: self.pending.depth + 1,
            },
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
impl SavepointExt for Savepoint<'_> {
    fn savepoint(&mut self) -> Result<Savepoint<'_>> {
        self.savepoint()
    }
}

impl Importer<'_, '_, Savepoint<'_>> {
    pub fn commit(self) -> crate::error::Result<()> {
        self.conn.commit()?;
        self.pending.commit()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ImporterPending<'parent> {
    pub buffer: HashSet<PathBuf>,
    pub parent: Option<&'parent mut HashSet<PathBuf>>,
    pub depth: usize,
}

impl ImporterPending<'_> {
    const IMPORTING_EXT: &'static str = "importing";
    pub fn stage(&mut self, src: &Path, dst: PathBuf) -> crate::error::Result<()> {
        let temp = dst.with_added_extension(Self::IMPORTING_EXT);
        std::fs::hard_link(src, &temp).or_else(|_| std::fs::copy(src, &temp).map(|_| ()))?;
        self.buffer.insert(dst);
        Ok(())
    }
    pub fn commit(mut self) -> crate::error::Result<()> {
        if self.depth <= 1 {
            for dst in &self.buffer {
                let src = dst.with_added_extension(Self::IMPORTING_EXT);
                fs::rename(&src, dst)?;
            }
            self.buffer.clear();
        } else {
            let parent = self.parent.as_mut().unwrap();
            parent.extend(mem::take(&mut self.buffer));
        }
        Ok(())
    }
}

impl Drop for ImporterPending<'_> {
    fn drop(&mut self) {
        for dst in &self.buffer {
            let src = dst.with_added_extension(Self::IMPORTING_EXT);
            let _ = fs::remove_file(&src);
        }
    }
}
