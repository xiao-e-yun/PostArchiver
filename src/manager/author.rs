use chrono::{DateTime, Utc};
use rusqlite::params;

use crate::{
    manager::binded::Binded, manager::PostArchiverConnection, utils::macros::AsTable, Alias,
    Author, AuthorId, FileMetaId, PlatformId, PostId,
};

//=============================================================
// Update / Delete
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, AuthorId, C> {
    /// Get this author's current data from the database.
    pub fn value(&self) -> Result<Author, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM authors WHERE id = ?")?;
        stmt.query_row([self.id()], Author::from_row)
    }

    /// Remove this author from the archive.
    ///
    /// This also removes all associated aliases and author-post relationships.
    pub fn delete(self) -> Result<(), rusqlite::Error> {
        self.conn()
            .execute("DELETE FROM authors WHERE id = ?", [self.id()])?;
        Ok(())
    }

    /// Set this author's name.
    pub fn set_name(&self, name: String) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET name = ? WHERE id = ?")?;
        stmt.execute(params![name, self.id()])?;
        Ok(())
    }

    /// Set this author's thumbnail.
    pub fn set_thumb(&self, thumb: Option<FileMetaId>) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET thumb = ? WHERE id = ?")?;
        stmt.execute(params![thumb, self.id()])?;
        Ok(())
    }

    /// Set this author's thumb to the latest post's thumb that has a non-null thumb.
    pub fn set_thumb_by_latest(&self) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "UPDATE authors SET thumb = (SELECT thumb FROM posts WHERE id IN (SELECT post FROM author_posts WHERE author = ?) AND thumb IS NOT NULL ORDER BY updated DESC LIMIT 1) WHERE id = ?",
        )?;
        stmt.execute(params![self.id(), self.id()])?;
        Ok(())
    }

    /// Set this author's updated timestamp.
    pub fn set_updated(&self, updated: DateTime<Utc>) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET updated = ? WHERE id = ?")?;
        stmt.execute(params![updated, self.id()])?;
        Ok(())
    }

    /// Set this author's updated timestamp to the latest post's updated time.
    pub fn set_updated_by_latest(&self) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "UPDATE authors SET updated = (SELECT updated FROM posts WHERE id IN (SELECT post FROM author_posts WHERE author = ?) ORDER BY updated DESC LIMIT 1) WHERE id = ?",
        )?;
        stmt.execute(params![self.id(), self.id()])?;
        Ok(())
    }
}

//=============================================================
// Relations: Aliases
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, AuthorId, C> {
    /// List all aliases associated with this author.
    pub fn list_aliases(&self) -> Result<Vec<Alias>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM author_aliases WHERE target = ?")?;
        let rows = stmt.query_map([self.id()], Alias::from_row)?;
        rows.collect()
    }

    /// Add or update aliases for this author.
    ///
    /// # Parameters
    /// - aliases: Vec of (source, platform, link)
    pub fn add_aliases(
        &self,
        aliases: Vec<(String, PlatformId, Option<String>)>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "INSERT OR REPLACE INTO author_aliases (target, source, platform, link) VALUES (?, ?, ?, ?)",
        )?;
        for (source, platform, link) in aliases {
            stmt.execute(params![self.id(), source, platform, link])?;
        }
        Ok(())
    }

    /// Remove aliases from this author.
    ///
    /// # Parameters
    /// - aliases: &[(source, platform)]
    pub fn remove_aliases(&self, aliases: &[(String, PlatformId)]) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "DELETE FROM author_aliases WHERE target = ? AND source = ? AND platform = ?",
        )?;
        for (source, platform) in aliases {
            stmt.execute(params![self.id(), source, platform])?;
        }
        Ok(())
    }

    /// Set an alias's name.
    pub fn set_alias_name(
        &self,
        alias: &(String, PlatformId),
        name: String,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "UPDATE author_aliases SET source = ? WHERE target = ? AND source = ? AND platform = ?",
        )?;
        stmt.execute(params![name, self.id(), alias.0, alias.1])?;
        Ok(())
    }

    /// Set an alias's platform.
    pub fn set_alias_platform(
        &self,
        alias: &(String, PlatformId),
        platform: PlatformId,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "UPDATE author_aliases SET platform = ? WHERE target = ? AND source = ? AND platform = ?",
        )?;
        stmt.execute(params![platform, self.id(), alias.0, alias.1])?;
        Ok(())
    }

    /// Set an alias's link.
    pub fn set_alias_link(
        &self,
        alias: &(String, PlatformId),
        link: Option<String>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "UPDATE author_aliases SET link = ? WHERE target = ? AND source = ? AND platform = ?",
        )?;
        stmt.execute(params![link, self.id(), alias.0, alias.1])?;
        Ok(())
    }
}

//=============================================================
// Relations: Posts
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, AuthorId, C> {
    /// List all post IDs associated with this author.
    pub fn list_posts(&self) -> Result<Vec<PostId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT post FROM author_posts WHERE author = ?")?;
        let rows = stmt.query_map([self.id()], |row| row.get(0))?;
        rows.collect()
    }
}

//=============================================================
// FindAlias trait (kept for importer compatibility)
//=============================================================
pub trait FindAlias {
    fn source(&self) -> &str;
    fn platform(&self) -> PlatformId;
}

impl FindAlias for (&str, PlatformId) {
    fn source(&self) -> &str {
        self.0
    }
    fn platform(&self) -> PlatformId {
        self.1
    }
}

#[cfg(feature = "importer")]
impl FindAlias for crate::importer::UnsyncAlias {
    fn source(&self) -> &str {
        &self.source
    }
    fn platform(&self) -> PlatformId {
        self.platform
    }
}
