use rusqlite::params;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    CollectionId, FileMetaId,
};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    pub fn set_collection_thumb(
        &self,
        collection: &CollectionId,
        thumb: Option<FileMetaId>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE collections SET thumb = ? WHERE id = ?")?;
        stmt.execute(params![thumb, collection])?;
        Ok(())
    }
    pub fn set_collection_thumb_by_latest(
        &self,
        collection: &CollectionId,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE collections SET thumb = (SELECT posts.thumb FROM collection_posts JOIN posts ON collection_posts.post = posts.id WHERE collection_posts.collection = ? AND posts.thumb IS NOT NULL ORDER BY updated DESC LIMIT 1) WHERE id = ?")?;
        stmt.execute(params![collection, collection])?;
        Ok(())
    }
    pub fn set_collection_description(
        &self,
        collection: &CollectionId,
        description: &str,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE collections SET description = ? WHERE id = ?")?;
        stmt.execute(params![description, collection])?;
        Ok(())
    }
}
