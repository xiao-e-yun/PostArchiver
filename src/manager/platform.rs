use rusqlite::params;

use crate::{
    manager::binded::Binded, manager::PostArchiverConnection, utils::macros::AsTable, Platform,
    PlatformId, PostId, TagId,
};

//=============================================================
// Update / Delete
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, PlatformId, C> {
    /// Get this platform's current data from the database.
    pub fn value(&self) -> Result<Platform, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM platforms WHERE id = ?")?;
        stmt.query_row([self.id()], Platform::from_row)
    }

    /// Remove this platform from the archive.
    ///
    /// This operation will also set the platform to UNKNOWN for all author aliases and posts.
    /// Tags associated with the platform will be deleted.
    pub fn delete(self) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM platforms WHERE id = ?")?;
        stmt.execute([self.id()])?;
        Ok(())
    }

    /// Set this platform's name.
    pub fn set_name(&self, name: String) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE platforms SET name = ? WHERE id = ?")?;
        stmt.execute(params![name, self.id()])?;
        Ok(())
    }
}

//=============================================================
// Relations: Tags / Posts
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, PlatformId, C> {
    /// List all tag IDs associated with this platform.
    pub fn list_tags(&self) -> Result<Vec<TagId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM tags WHERE platform = ?")?;
        let rows = stmt.query_map([self.id()], |row| row.get(0))?;
        rows.collect()
    }

    /// List all post IDs associated with this platform.
    pub fn list_posts(&self) -> Result<Vec<PostId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM posts WHERE platform = ?")?;
        let rows = stmt.query_map([self.id()], |row| row.get(0))?;
        rows.collect()
    }
}
