use std::hash::Hash;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    CollectionId,
};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsyncCollection {
    pub name: String,
    pub source: String,
}

impl Hash for UnsyncCollection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.source.hash(state);
    }
}
