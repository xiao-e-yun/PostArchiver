use std::{hash::Hash, ops::Deref};

use rusqlite::{params, Connection, OptionalExtension};

use crate::{error::Result, importer::Importer, PlatformId, TagId};

impl<C: Deref<Target = Connection>> Importer<'_, '_, C> {
    /// Import a tag into the archive.
    ///
    /// If the tag already exists, it returns the existing ID.
    ///
    /// # Errors
    ///
    /// Returns `Error` if there was an error accessing the database.
    pub fn import_tag(&self, tag: UnsyncTag) -> Result<TagId> {
        if let Some(id) = self.lookup_tag(&tag.name, tag.platform)? {
            return Ok(id);
        }

        let mut ins_stmt = self
            .conn
            .prepare_cached("INSERT INTO tags (name, platform) VALUES (?, ?) RETURNING id")?;
        Ok(ins_stmt.query_row(params![tag.name, tag.platform], |row| row.get(0))?)
    }

    pub fn lookup_tag(&self, name: &str, platform: Option<PlatformId>) -> Result<Option<TagId>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT id FROM tags WHERE name = ? AND platform IS ?")?;
        let query = stmt.query_row(params![name, platform], |row| row.get(0));
        Ok(query.optional()?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnsyncTag {
    pub name: String,
    pub platform: Option<PlatformId>,
}
