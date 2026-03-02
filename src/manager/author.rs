use chrono::{DateTime, Utc};
use rusqlite::params;

use crate::{
    error::Result, manager::binded::Binded, manager::PostArchiverConnection,
    utils::macros::AsTable, Alias, Author, AuthorId, FileMetaId, PlatformId, PostId,
};

/// Specifies how to update an author's thumbnail.
#[derive(Debug, Clone)]
pub enum AuthorThumb {
    /// Set to an explicit value (or clear with `None`).
    Set(Option<FileMetaId>),
    /// Set to the thumb of the most recently updated post associated with this author.
    ByLatest,
}

/// Specifies how to update an author's `updated` timestamp.
#[derive(Debug, Clone)]
pub enum AuthorUpdated {
    /// Unconditionally set to this value.
    Set(DateTime<Utc>),
    /// Set to the `updated` time of the most recently updated post associated with this author.
    ByLatest,
}

/// Builder for updating an author's fields.
///
/// Fields left as `None` are not modified.
#[derive(Debug, Clone, Default)]
pub struct UpdateAuthor {
    pub name: Option<String>,
    pub thumb: Option<AuthorThumb>,
    pub updated: Option<AuthorUpdated>,
}

impl UpdateAuthor {
    /// Set the author's name.
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
    /// Set or clear the author's thumbnail.
    pub fn thumb(mut self, thumb: Option<FileMetaId>) -> Self {
        self.thumb = Some(AuthorThumb::Set(thumb));
        self
    }
    /// Set the author's thumbnail to the latest post's thumb.
    pub fn thumb_by_latest(mut self) -> Self {
        self.thumb = Some(AuthorThumb::ByLatest);
        self
    }
    /// Unconditionally set the updated timestamp.
    pub fn updated(mut self, updated: DateTime<Utc>) -> Self {
        self.updated = Some(AuthorUpdated::Set(updated));
        self
    }
    /// Set the updated timestamp to the latest associated post's `updated` time.
    pub fn updated_by_latest(mut self) -> Self {
        self.updated = Some(AuthorUpdated::ByLatest);
        self
    }
}

//=============================================================
// Update / Delete
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, AuthorId, C> {
    /// Get this author's current data from the database.
    pub fn value(&self) -> Result<Author> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM authors WHERE id = ?")?;
        Ok(stmt.query_row([self.id()], Author::from_row)?)
    }

    /// Remove this author from the archive.
    ///
    /// This also removes all associated aliases and author-post relationships.
    pub fn delete(self) -> Result<()> {
        self.conn()
            .execute("DELETE FROM authors WHERE id = ?", [self.id()])?;
        Ok(())
    }

    /// Apply a batch of field updates to this author in a single SQL statement.
    ///
    /// Only fields set on `update` (i.e. `Some(...)`) are written to the database.
    pub fn update(&self, update: UpdateAuthor) -> Result<()> {
        use rusqlite::types::ToSql;

        let id = self.id();
        let mut sets: Vec<&str> = Vec::new();
        let mut params: Vec<&dyn ToSql> = Vec::new();

        macro_rules! push {
            ($field:expr, $col:expr) => {
                if let Some(ref v) = $field {
                    sets.push($col);
                    params.push(v);
                }
            };
        }

        push!(update.name, "name = ?");

        match &update.thumb {
            Some(AuthorThumb::Set(v)) => {
                sets.push("thumb = ?");
                params.push(v);
            }
            Some(AuthorThumb::ByLatest) => {
                sets.push("thumb = (SELECT thumb FROM posts WHERE id IN (SELECT post FROM author_posts WHERE author = ?) AND thumb IS NOT NULL ORDER BY updated DESC LIMIT 1)");
                params.push(&id);
            }
            None => {}
        }

        match &update.updated {
            Some(AuthorUpdated::Set(v)) => {
                sets.push("updated = ?");
                params.push(v);
            }
            Some(AuthorUpdated::ByLatest) => {
                sets.push("updated = (SELECT updated FROM posts WHERE id IN (SELECT post FROM author_posts WHERE author = ?) ORDER BY updated DESC LIMIT 1)");
                params.push(&id);
            }
            None => {}
        }

        if sets.is_empty() {
            return Ok(());
        }

        params.push(&id);

        let sql = format!("UPDATE authors SET {} WHERE id = ?", sets.join(", "));
        self.conn().execute(&sql, params.as_slice())?;
        Ok(())
    }
}

//=============================================================
// Relations: Aliases
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, AuthorId, C> {
    /// List all aliases associated with this author.
    pub fn list_aliases(&self) -> Result<Vec<Alias>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM author_aliases WHERE target = ?")?;
        let rows = stmt.query_map([self.id()], Alias::from_row)?;
        rows.collect::<std::result::Result<_, _>>()
            .map_err(Into::into)
    }

    /// Add or update aliases for this author.
    ///
    /// # Parameters
    /// - aliases: Vec of (source, platform, link)
    pub fn add_aliases(&self, aliases: Vec<(String, PlatformId, Option<String>)>) -> Result<()> {
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
    pub fn remove_aliases(&self, aliases: &[(String, PlatformId)]) -> Result<()> {
        let mut stmt = self.conn().prepare_cached(
            "DELETE FROM author_aliases WHERE target = ? AND source = ? AND platform = ?",
        )?;
        for (source, platform) in aliases {
            stmt.execute(params![self.id(), source, platform])?;
        }
        Ok(())
    }

    /// Set an alias's name.
    pub fn set_alias_name(&self, alias: &(String, PlatformId), name: String) -> Result<()> {
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
    ) -> Result<()> {
        let mut stmt = self.conn().prepare_cached(
            "UPDATE author_aliases SET platform = ? WHERE target = ? AND source = ? AND platform = ?",
        )?;
        stmt.execute(params![platform, self.id(), alias.0, alias.1])?;
        Ok(())
    }

    /// Set an alias's link.
    pub fn set_alias_link(&self, alias: &(String, PlatformId), link: Option<String>) -> Result<()> {
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
    pub fn list_posts(&self) -> Result<Vec<PostId>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT post FROM author_posts WHERE author = ?")?;
        let rows = stmt.query_map([self.id()], |row| row.get(0))?;
        rows.collect::<std::result::Result<_, _>>()
            .map_err(Into::into)
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
