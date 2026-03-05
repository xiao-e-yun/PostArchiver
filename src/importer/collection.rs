use std::hash::Hash;

use rusqlite::params;

use crate::{
    error::Result,
    manager::{PostArchiverConnection, PostArchiverManager, UpdateCollection},
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
    /// Returns `Error` if there was an error accessing the database.
    pub fn import_collection(&self, collection: UnsyncCollection) -> Result<CollectionId> {
        // find by source
        if let Some(id) = self.find_collection_by_source(&collection.source)? {
            self.bind(id)
                .update(UpdateCollection::default().name(collection.name))?;
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
    /// Returns `Error` if there was an error accessing the database.
    pub fn import_collections(
        &self,
        collections: impl IntoIterator<Item = UnsyncCollection>,
    ) -> Result<Vec<CollectionId>> {
        collections
            .into_iter()
            .map(|collection| self.import_collection(collection))
            .collect::<Result<Vec<CollectionId>>>()
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
