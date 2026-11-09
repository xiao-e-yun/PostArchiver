use std::ops::Deref;

use rusqlite::{Connection, OptionalExtension};

use crate::{error::Result, importer::Importer, PlatformId};

impl<C: Deref<Target = Connection>> Importer<'_, C> {
    /// Import a platform into the archive.
    ///
    /// If the platform already exists, it returns the existing ID.
    ///
    /// # Errors
    ///
    /// Returns `Error` if there was an error accessing the database.
    pub fn import_platform(&self, platform: String) -> Result<PlatformId> {
        if let Some(id) = self.lookup_platform(&platform)? {
            return Ok(id);
        }

        // insert
        let mut stmt = self
            .conn
            .prepare_cached("INSERT INTO platforms (name) VALUES (?) RETURNING id")?;
        Ok(stmt.query_row([&platform], |row| row.get(0))?)
    }

    pub fn lookup_platform(&self, platform: &str) -> Result<Option<PlatformId>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT id FROM platforms WHERE name = ?")?;
        let query = stmt.query_row([platform], |row| row.get(0));
        Ok(query.optional()?)
    }
}
