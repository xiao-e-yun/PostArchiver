use rusqlite::{params, OptionalExtension};

use crate::{Collection, CollectionId, FileMetaId, Post, PostId};

use super::{PostArchiverConnection, PostArchiverManager};

//=============================================================
// Querying
//=============================================================
impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    pub fn list_collections(&self) -> Result<Vec<crate::Collection>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM collections")?;
        let collections = stmt.query_map([], |row| crate::Collection::from_row(row))?;
        collections.collect()
    }

    pub fn find_collection(&self, source: &str) -> Result<Option<CollectionId>, rusqlite::Error> {
        if let Some(id) = self.cache.collections.get(source) {
            return Ok(Some(*id));
        }

        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM collections WHERE source = ?")?;
        let id = stmt.query_row([source], |row| row.get(0)).optional();

        if let Ok(Some(id)) = id {
            self.cache.collections.insert(source.to_string(), id);
        }

        id
    }
    /// Get a collection tag by its id or name.
    pub fn get_collection(&self, id: &CollectionId) -> Result<Option<Collection>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM collections WHERE id = ?")?;

        stmt.query_row([id], |row| crate::Collection::from_row(row))
            .optional()
    }
}

//=============================================================
// Modifying
//=============================================================
impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    pub fn add_collection(
        &self,
        name: String,
        source: Option<String>,
        thumb: Option<FileMetaId>,
    ) -> Result<CollectionId, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "INSERT INTO collections (name, source, thumb) VALUES (?, ?, ?) RETURNING id",
        )?;
        let id = stmt.query_row(params![name, source, thumb], |row| row.get(0))?;

        if let Some(source) = &source {
            self.cache.collections.insert(source.clone(), id);
        }
        Ok(id)
    }

    pub fn remove_collection(&self, id: CollectionId) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM collection_posts WHERE collection = ?")?;
        stmt.execute([id])?;
        Ok(())
    }

    pub fn set_collection_name(
        &self,
        id: CollectionId,
        name: String,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE collections SET name = ? WHERE id = ?")?;
        stmt.execute(params![name, id])?;
        Ok(())
    }

    pub fn set_collection_source(
        &self,
        id: CollectionId,
        source: Option<String>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE collections SET source = ? WHERE id = ?")?;
        stmt.execute(params![&source, id])?;
        if let Some(source) = source {
            self.cache.collections.insert(source, id);
        }
        Ok(())
    }

    pub fn set_collection_thumb(
        &self,
        id: CollectionId,
        thumb: Option<String>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE collections SET thumb = ? WHERE id = ?")?;
        stmt.execute(params![thumb, id])?;
        Ok(())
    }

    pub fn set_collection_thumb_by_latest(&self, id: CollectionId) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "UPDATE collections SET thumb = (SELECT id FROM file_metas WHERE post = (SELECT id FROM posts WHERE collection = ? ORDER BY updated DESC LIMIT 1)) WHERE id = ?",
        )?;
        stmt.execute(params![id, id])?;
        Ok(())
    }
}

//=============================================================
// Relationships
//=============================================================
impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    pub fn list_post_collections(&self, post: &PostId) -> Result<Vec<Collection>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT collections.* FROM collections INNER JOIN collection_posts ON post_collections.collection = collections.id WHERE post_collections.post = ?")?;
        let collections = stmt.query_map([post], |row| crate::Collection::from_row(row))?;
        collections.collect()
    }

    pub fn list_collection_posts(
        &self,
        collection: &CollectionId,
    ) -> Result<Vec<Post>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT posts.* FROM posts INNER JOIN collection_posts ON collection_posts.post = posts.id WHERE collection_posts.collection = ?")?;
        let posts = stmt.query_map([collection], |row| Post::from_row(row))?;
        posts.collect()
    }
}

impl Post {
    pub fn collections(
        &self,
        manager: &PostArchiverManager,
    ) -> Result<Vec<Collection>, rusqlite::Error> {
        manager.list_post_collections(&self.id)
    }
}

impl Collection {
    pub fn posts(&self, manager: &PostArchiverManager) -> Result<Vec<Post>, rusqlite::Error> {
        manager.list_collection_posts(&self.id)
    }
}
