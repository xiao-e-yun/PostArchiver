use rusqlite::OptionalExtension;

use crate::{utils::collection::CollectionLike, Collection, CollectionId, Post, PostId};

use super::{PostArchiverConnection, PostArchiverManager};

#[derive(Debug, Clone)]
pub enum CollectionIdOrRaw {
    Id(CollectionId),
    Raw(String),
}

impl From<Collection> for CollectionIdOrRaw {
    fn from(value: Collection) -> Self {
        CollectionIdOrRaw::Id(value.id)
    }
}

impl From<CollectionId> for CollectionIdOrRaw {
    fn from(value: CollectionId) -> Self {
        CollectionIdOrRaw::Id(value)
    }
}

impl From<&str> for CollectionIdOrRaw {
    fn from(value: &str) -> Self {
        CollectionIdOrRaw::Raw(value.to_string())
    }
}

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Retrieve all collections.
    pub fn list_collections(&self) -> Result<Vec<Collection>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM collections")?;
        let tags = stmt.query_map([], |row| Collection::from_row(row))?;
        tags.collect()
    }

    /// Retrieve all collections associated with a post.
    ///
    /// Fetches all collections that the given post ID belongs to.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::PostId;
    /// fn example(manager: &PostArchiverManager, post_id: PostId) -> Result<(), Box<dyn
    /// std::error::Error>> {
    ///     let collections = manager.list_post_collections(&post_id)?;
    ///     for collection in collections {
    ///         println!("Post collection: {}", collection.name);
    ///     };
    ///     Ok(())
    /// }
    /// ```
    pub fn list_post_collections(
        &self,
        post: &PostId,
    ) -> Result<Vec<crate::Collection>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT collections.* FROM collections INNER JOIN collection_posts ON post_collections.collection = collections.id WHERE post_collections.post = ?")?;
        let collections = stmt.query_map([post], |row| crate::Collection::from_row(row))?;
        collections.collect()
    }

    /// Get a collection tag by its id or name.
    pub fn get_collection(
        &self,
        collection: &impl CollectionLike,
    ) -> Result<Option<crate::Collection>, rusqlite::Error> {
        match collection.id() {
            Some(id) => {
                let mut stmt = self
                    .conn()
                    .prepare_cached("SELECT * FROM collections WHERE id = ?")?;
                stmt.query_row([id], |row| Collection::from_row(row))
                    .optional()
            }
            None => {
                let name = collection.raw().unwrap();
                let mut stmt = self
                    .conn()
                    .prepare_cached("SELECT * FROM collections WHERE name = ?")?;
                stmt.query_row([name], |row| Collection::from_row(row))
                    .optional()
            }
        }
    }
}

impl Post {
    pub fn collections(
        &self,
        manager: &PostArchiverManager,
    ) -> Result<Vec<crate::Collection>, rusqlite::Error> {
        manager.list_post_collections(&self.id)
    }
}
