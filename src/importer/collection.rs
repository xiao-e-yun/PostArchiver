use std::hash::Hash;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    CollectionId,
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
        match self.find_collection(&collection.source)? {
            Some(id) => {
                self.set_collection_name(id, collection.name)?;
                Ok(id)
            }
            None => self.add_collection(
                collection.name,
                Some(collection.source),
                None, // No thumbnail for unsynced collections
            ),
        }
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

/// Represents a file metadata that is not yet synced to the database.
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
