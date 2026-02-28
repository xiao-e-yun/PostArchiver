use rusqlite::params;

use crate::{
    manager::binded::Binded, manager::PostArchiverConnection, CollectionId, FileMetaId, PostId,
};

//=============================================================
// Update / Delete
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, CollectionId, C> {
    /// Remove this collection from the archive.
    ///
    /// Also removes all collection-post relationships.
    pub fn delete(&self) -> Result<(), rusqlite::Error> {
        self.conn()
            .execute("DELETE FROM collections WHERE id = ?", [self.id()])?;
        Ok(())
    }

    /// Set this collection's name.
    pub fn set_name(&self, name: String) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE collections SET name = ? WHERE id = ?")?;
        stmt.execute(params![name, self.id()])?;
        Ok(())
    }

    /// Set this collection's source.
    pub fn set_source(&self, source: Option<String>) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE collections SET source = ? WHERE id = ?")?;
        stmt.execute(params![source, self.id()])?;
        Ok(())
    }

    /// Set this collection's thumb.
    pub fn set_thumb(&self, thumb: Option<FileMetaId>) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE collections SET thumb = ? WHERE id = ?")?;
        stmt.execute(params![thumb, self.id()])?;
        Ok(())
    }

    /// Set the collection's thumb to the latest post's thumb that has a non-null thumb.
    pub fn set_thumb_by_latest(&self) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "UPDATE collections SET thumb = (
                SELECT posts.thumb FROM posts
                INNER JOIN collection_posts ON collection_posts.post = posts.id
                WHERE collection_posts.collection = ? AND posts.thumb IS NOT NULL
                ORDER BY posts.updated DESC LIMIT 1
            ) WHERE id = ?",
        )?;
        stmt.execute(params![self.id(), self.id()])?;
        Ok(())
    }
}

//=============================================================
// Relations: Posts
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, CollectionId, C> {
    /// List all post IDs in this collection.
    pub fn list_posts(&self) -> Result<Vec<PostId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT post FROM collection_posts WHERE collection = ?")?;
        let rows = stmt.query_map([self.id()], |row| row.get(0))?;
        rows.collect()
    }

    /// Add posts to this collection.
    pub fn add_posts(&self, posts: &[PostId]) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "INSERT OR IGNORE INTO collection_posts (collection, post) VALUES (?, ?)",
        )?;
        for post in posts {
            stmt.execute(params![self.id(), post])?;
        }
        Ok(())
    }

    /// Remove posts from this collection.
    pub fn remove_posts(&self, posts: &[PostId]) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM collection_posts WHERE collection = ? AND post = ?")?;
        for post in posts {
            stmt.execute(params![self.id(), post])?;
        }
        Ok(())
    }
}
