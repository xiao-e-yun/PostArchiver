use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    Alias, Author, AuthorId, FileMetaId, PlatformId, Post, PostId,
};

//=============================================================
// Querying
//=============================================================
impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Retrieve all authors in the archive.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn list_authors(&self) -> Result<Vec<Author>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM authors")?;
        let authors = stmt.query_map([], Author::from_row)?;
        authors.collect()
    }
    /// Find an author by their aliases.
    ///
    /// Checks if any of the provided aliases match an existing author in the archive.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error querying the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::{AuthorId, PlatformId};
    /// # fn example(manager: &PostArchiverManager) -> Result<(), Box<dyn std::error::Error>> {
    /// let platform = todo!("Define your platform ID here");
    ///
    /// let author = manager.find_author(&[("octocat", platform)])?;
    ///
    /// match author {
    ///     Some(id) => println!("Found author with ID: {}", id),
    ///     None => println!("No author found for the given aliases"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn find_author(
        &self,
        aliases: &[impl FindAlias],
    ) -> Result<Option<AuthorId>, rusqlite::Error> {
        if aliases.is_empty() {
            return Ok(None);
        }

        let mut stmt = self.conn().prepare_cached(
            "SELECT target FROM author_aliases WHERE platform = ? AND source = ?",
        )?;

        for alias in aliases {
            if let Some(id) = stmt
                .query_row(params![alias.platform(), alias.source()], |row| row.get(0))
                .optional()?
            {
                return Ok(id);
            }
        }

        Ok(None)
    }
    /// Retrieve an author by their ID.
    ///
    /// Fetches all information about an author including their name, links, and metadata.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if:
    /// * The author ID does not exist
    /// * There was an error accessing the database
    pub fn get_author(&self, author: AuthorId) -> Result<Author, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM authors WHERE id = ?")?;
        stmt.query_row([author], |row| {
            Ok(Author {
                id: row.get("id")?,
                name: row.get("name")?,
                thumb: row.get("thumb")?,
                updated: row.get("updated")?,
            })
        })
    }
}

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

//=============================================================
// Modifying
//=============================================================
impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Add a new author to the archive.
    ///
    /// Inserts a new author with the given name and optional updated timestamp.
    /// It does not check for duplicates, so ensure the author does not already exist.
    ///
    /// # Parameters
    /// - updated: Optional timestamp for when the author was last updated. Defaults to the current time if not provided.
    ///
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn add_author(
        &self,
        name: String,
        updated: Option<DateTime<Utc>>,
    ) -> Result<AuthorId, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT INTO authors (name, updated) VALUES (?, ?) RETURNING id")?;

        let id: AuthorId = stmt
            .query_row(params![name, updated.unwrap_or_else(Utc::now)], |row| {
                row.get(0)
            })?;
        Ok(id)
    }
    /// Remove an author from the archive.
    ///
    /// This operation will also remove all associated aliases.
    /// And Author-Post relationships will be removed as well.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn remove_author(&self, author: AuthorId) -> Result<(), rusqlite::Error> {
        self.conn()
            .execute("DELETE FROM authors WHERE id = ?", [author])?;
        Ok(())
    }
    /// Add or update aliases for an author.
    ///
    /// Inserts multiple aliases for the specified author.
    /// If alias.source and alias.platform already exist for the author, it will be replaced.
    ///
    /// # Parameters
    ///
    /// - aliases[..]: (Source, Platform, Link)
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::{AuthorId, PlatformId};
    /// # fn example(manager: &PostArchiverManager, author_id: AuthorId) -> Result<(), rusqlite::Error> {
    /// let aliases = vec![
    ///     ("octocat".to_string(), PlatformId(1), Some("https://example.com/octocat".to_string())),
    ///     ("octocat2".to_string(), PlatformId(2), None),
    /// ];
    ///
    /// manager.add_author_aliases(author_id, aliases)
    /// # }
    /// ```
    ///
    pub fn add_author_aliases(
        &self,
        author: AuthorId,
        aliases: Vec<(String, PlatformId, Option<String>)>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT OR REPLACE INTO author_aliases (target, source, platform, link) VALUES (?, ?, ?, ?)")?;

        for (source, platform, target) in aliases {
            stmt.execute(params![author, source, platform, target])?;
        }
        Ok(())
    }

    /// Remove aliases for an author.
    ///
    /// Deletes the specified aliases for the given author.
    /// If an alias does not exist, it will be ignored.
    ///
    /// # Parameters
    ///
    /// - `aliases[..]`: (Source, Platform)
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn remove_author_aliases(
        &self,
        author: AuthorId,
        aliases: &[(String, PlatformId)],
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "DELETE FROM author_aliases WHERE target = ? AND source = ? AND platform = ?",
        )?;

        for (source, platform) in aliases {
            stmt.execute(params![author, source, platform])?;
        }
        Ok(())
    }

    /// Set an name of author's alias.
    ///
    /// # Parameters
    ///
    /// - `alias`: (Source, Platform)
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_author_alias_name(
        &self,
        author: AuthorId,
        alias: &(String, PlatformId),
        name: String,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "UPDATE author_aliases SET source = ? WHERE target = ? AND source = ? AND platform = ?",
        )?;

        stmt.execute(params![name, author, alias.0, alias.1])?;
        Ok(())
    }

    /// Set an platform of author's alias.
    ///
    /// # Parameters
    ///
    /// - `alias`: (Source, Platform)
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    pub fn set_author_alias_platform(
        &self,
        author: AuthorId,
        alias: &(String, PlatformId),
        platform: PlatformId,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE author_aliases SET platform = ? WHERE target = ? AND source = ? AND platform = ?")?;

        stmt.execute(params![platform, author, alias.0, alias.1])?;
        Ok(())
    }

    /// Set a link of author's alias.
    ///
    /// # Parameters
    ///
    /// - `alias`: (Source, Platform)
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_author_alias_link(
        &self,
        author: AuthorId,
        alias: &(String, PlatformId),
        link: Option<String>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "UPDATE author_aliases SET link = ? WHERE target = ? AND source = ? AND platform = ?",
        )?;

        stmt.execute(params![link, author, alias.0, alias.1])?;
        Ok(())
    }

    /// Set an name of author.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_author_name(&self, author: AuthorId, name: String) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET name = ? WHERE id = ?")?;
        stmt.execute(params![name, author])?;
        Ok(())
    }

    /// Set a thumb of author.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_author_thumb(
        &self,
        author: AuthorId,
        thumb: Option<FileMetaId>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET thumb = ? WHERE id = ?")?;
        stmt.execute(params![thumb, author])?;
        Ok(())
    }

    /// Set the author's thumb to the latest post's thumb  that has a non-null thumb.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_author_thumb_by_latest(&self, author: AuthorId) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET thumb = (SELECT thumb FROM posts WHERE id IN (SELECT post FROM author_posts WHERE author = ?) AND thumb IS NOT NULL ORDER BY updated DESC LIMIT 1) WHERE id = ?")?;
        stmt.execute(params![author, author])?;
        Ok(())
    }

    /// Set the updated timestamp of an author.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_author_updated(
        &self,
        author: AuthorId,
        updated: DateTime<Utc>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET updated = ? WHERE id = ?")?;
        stmt.execute(params![updated, author])?;
        Ok(())
    }

    pub fn set_author_updated_by_latest(&self, author: AuthorId) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET updated = (SELECT updated FROM posts WHERE id IN (SELECT post FROM author_posts WHERE author = ?) ORDER BY updated DESC LIMIT 1) WHERE id = ?")?;
        stmt.execute(params![author, author])?;
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
    /// Retrieve all aliases associated with an author.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn list_author_aliases(&self, author: AuthorId) -> Result<Vec<Alias>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM author_aliases WHERE target = ?")?;
        let tags = stmt.query_map([author], Alias::from_row)?;
        tags.collect()
    }
    /// Retrieve all posts associated with an author.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn list_author_posts(&self, author: AuthorId) -> Result<Vec<Post>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT posts.* FROM posts INNER JOIN author_posts ON author_posts.post = posts.id WHERE author_posts.author = ?")?;
        let posts = stmt.query_map([author], Post::from_row)?;
        posts.collect()
    }
    /// Retrieve all authors associated with a post.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn list_post_authors(&self, post: &PostId) -> Result<Vec<Author>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT authors.* FROM authors INNER JOIN author_posts ON author_posts.author = authors.id WHERE author_posts.post = ?")?;
        let authors = stmt.query_map([post], Author::from_row)?;
        authors.collect()
    }
}

impl Author {
    /// Retrieve all aliases associated with this author.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn aliases(&self, manager: &PostArchiverManager) -> Result<Vec<Alias>, rusqlite::Error> {
        manager.list_author_aliases(self.id)
    }
    /// Retrieve all posts associated with this author.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn posts(&self, manager: &PostArchiverManager) -> Result<Vec<Post>, rusqlite::Error> {
        manager.list_author_posts(self.id)
    }
}

impl Post {
    /// Retrieve all authors associated with this post.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn authors(&self, manager: &PostArchiverManager) -> Result<Vec<Author>, rusqlite::Error> {
        manager.list_post_authors(&self.id)
    }
}
