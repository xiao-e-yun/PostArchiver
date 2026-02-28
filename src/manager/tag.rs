use rusqlite::params;

use crate::{manager::binded::Binded, manager::PostArchiverConnection, PlatformId, PostId, TagId};

//=============================================================
// FindTag trait (kept for importer compatibility)
//=============================================================
pub trait FindTag {
    fn name(&self) -> &str;
    fn platform(&self) -> Option<PlatformId> {
        None
    }
}

impl FindTag for &str {
    fn name(&self) -> &str {
        self
    }
}

impl FindTag for (&str, PlatformId) {
    fn name(&self) -> &str {
        self.0
    }
    fn platform(&self) -> Option<PlatformId> {
        Some(self.1)
    }
}

impl FindTag for (&str, Option<PlatformId>) {
    fn name(&self) -> &str {
        self.0
    }
    fn platform(&self) -> Option<PlatformId> {
        self.1
    }
}

//=============================================================
// Update / Delete
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, TagId, C> {
    /// Remove this tag from the archive.
    ///
    /// This will also remove all post-tag relationships, but will not delete the posts themselves.
    pub fn delete(&self) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM tags WHERE id = ?")?;
        stmt.execute([self.id()])?;
        Ok(())
    }

    /// Set this tag's name.
    pub fn set_name(&self, name: String) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE tags SET name = ? WHERE id = ?")?;
        stmt.execute(params![name, self.id()])?;
        Ok(())
    }

    /// Set this tag's platform.
    pub fn set_platform(&self, platform: Option<PlatformId>) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE tags SET platform = ? WHERE id = ?")?;
        stmt.execute(params![platform, self.id()])?;
        Ok(())
    }
}

//=============================================================
// Relations: Posts
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, TagId, C> {
    /// List all post IDs associated with this tag.
    pub fn list_posts(&self) -> Result<Vec<PostId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT post FROM post_tags WHERE tag = ?")?;
        let rows = stmt.query_map([self.id()], |row| row.get(0))?;
        rows.collect()
    }
}
