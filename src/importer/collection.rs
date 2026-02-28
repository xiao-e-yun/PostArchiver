use std::hash::Hash;

use rusqlite::{params, OptionalExtension};

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    CollectionId, FileMetaId,
};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Import a collection into the archive.
    ///
    /// If the collection already exists (by source), it updates its name and returns the existing ID.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn import_collection(
        &self,
        collection: UnsyncCollection,
    ) -> Result<CollectionId, rusqlite::Error> {
        // find by source
        let mut find_stmt = self
            .conn()
            .prepare_cached("SELECT id FROM collections WHERE source = ?")?;
        if let Some(id) = find_stmt
            .query_row([&collection.source], |row| row.get::<_, CollectionId>(0))
            .optional()?
        {
            self.bind(id).set_name(collection.name)?;
            return Ok(id);
        }

        // insert
        let mut ins_stmt = self.conn().prepare_cached(
            "INSERT INTO collections (name, source, thumb) VALUES (?, ?, ?) RETURNING id",
        )?;
        let id: CollectionId = ins_stmt.query_row(
            params![
                collection.name,
                collection.source,
                Option::<FileMetaId>::None
            ],
            |row| row.get(0),
        )?;
        Ok(id)
    }

    /// Import multiple collections into the archive.
    ///
    /// This method takes an iterator of `UnsyncCollection` and imports each one.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn import_collections(
        &self,
        collections: impl IntoIterator<Item = UnsyncCollection>,
    ) -> Result<Vec<CollectionId>, rusqlite::Error> {
        collections
            .into_iter()
            .map(|collection| self.import_collection(collection))
            .collect::<Result<Vec<CollectionId>, rusqlite::Error>>()
    }
}

/// Represents a collection that is not yet synced to the database.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnsyncCollection {
    pub name: String,
    pub source: String,
}

impl UnsyncCollection {
    pub fn new(name: String, source: String) -> Self {
        Self { name, source }
    }
    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }
    pub fn source(mut self, source: String) -> Self {
        self.source = source;
        self
    }
}
