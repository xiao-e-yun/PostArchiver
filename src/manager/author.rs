use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    Alias, Author, AuthorId, FileMetaId, PlatformId, Post,
};

//=============================================================
// Querying
//=============================================================
impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    pub fn list_authors(&self) -> Result<Vec<Author>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM authors")?;
        let authors = stmt.query_map([], |row| Author::from_row(row))?;
        authors.collect()
    }
    /// Look up an author by their alias list.
    ///
    /// Searches the database for an author that matches any of the provided aliases.
    /// Returns the first matching author's ID, or `None` if no match is found.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error querying the database.
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

        return Ok(None);
    }
    /// Retrieve an author's complete information from the archive.
    ///
    /// Fetches all information about an author including their name, links, and metadata.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if:
    /// * The author ID does not exist
    /// * There was an error accessing the database
    /// * The stored data is malformed
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::AuthorId;
    /// fn example(manager: &PostArchiverManager, id: AuthorId) -> Result<(), Box<dyn std::error::Error>> {
    ///     let author = manager.get_author(&id)?;
    ///     println!("Author name: {}", author.name);
    ///     
    ///     Ok(())
    /// }
    /// ```
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
        &self.0
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
    pub fn add_author(
        &self,
        name: String,
        updated: Option<DateTime<Utc>>,
    ) -> Result<AuthorId, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT INTO authors (name, updated) VALUES (?, ?) RETURNING id")?;

        let id: AuthorId = stmt.query_row(params![name, updated], |row| row.get(0))?;
        Ok(id)
    }
    pub fn remove_author(&self, author: AuthorId) -> Result<(), rusqlite::Error> {
        self.conn()
            .execute("DELETE FROM authors WHERE id = ?", [author])?;
        Ok(())
    }
    /// List of aliases to add for the author.
    /// 1. `source`: The alias name.
    /// 2. `platform`: The platform ID for the alias.
    /// 3. `link`: Optional link associated with the alias.
    pub fn add_author_aliases(
        &self,
        author: AuthorId,
        aliases: Vec<(String, PlatformId, Option<String>)>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT INTO author_aliases (target, source, platform, link) VALUES (?, ?, ?, ?) ON CONFLICT REPLACE")?;

        for (source, platform, target) in aliases {
            stmt.execute(params![author, source, platform, target])?;
        }
        Ok(())
    }

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

    pub fn set_author_name(&self, author: AuthorId, name: String) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET name = ? WHERE id = ?")?;
        stmt.execute(params![name, author])?;
        Ok(())
    }

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

    pub fn set_author_thumb_by_latest(&self, author: AuthorId) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET thumb = (SELECT thumb FROM posts WHERE id IN (SELECT post FROM author_posts WHERE author = ?) AND thumb IS NOT NULL ORDER BY updated DESC LIMIT 1) WHERE id = ?")?;
        stmt.execute(params![author, author])?;
        Ok(())
    }

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
    /// Fetches all alternate identifiers that map to the given author ID.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::AuthorId;
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let author_id = AuthorId(1);
    ///     
    ///     let aliases = manager.get_author_aliases(&author_id)?;
    ///     for alias in aliases {
    ///         println!("Author aliases: {} -> {}", alias.source, alias.target);
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn list_author_aliases(&self, author: AuthorId) -> Result<Vec<Alias>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM author_aliases WHERE target = ?")?;
        let tags = stmt.query_map([author], |row| Alias::from_row(row))?;
        tags.collect()
    }
    /// Retrieve all posts associated with an author.
    ///
    /// Fetches all posts that have been authored by the given author ID.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::{AuthorId, Post};
    /// fn example(manager: &PostArchiverManager, author_id: AuthorId) -> Result<(), Box<dyn
    /// std::error::Error>> {
    ///     let posts = manager.get_author_posts(&author_id)?;
    ///     for post in posts {
    ///         println!("Author post: {}", post.title);
    ///     };
    ///     Ok(())
    /// }
    /// ```
    pub fn list_author_posts(&self, author: AuthorId) -> Result<Vec<Post>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT posts.* FROM posts INNER JOIN author_posts ON author_posts.post = posts.id WHERE author_posts.author = ?")?;
        let posts = stmt.query_map([author], |row| Post::from_row(row))?;
        posts.collect()
    }
    /// Retrieve all authors associated with a post.
    ///
    /// Fetches all authors that have contributed to the given post ID.
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
    ///     let authors = manager.get_post_authors(&post_id)?;
    ///     for author in authors {
    ///         println!("Post author: {}", author.name);
    ///     };
    ///     Ok(())
    /// }
    /// ```
    pub fn list_post_authors(&self, post: &Post) -> Result<Vec<Author>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT authors.* FROM authors INNER JOIN author_posts ON author_posts.author = authors.id WHERE author_posts.post = ?")?;
        let authors = stmt.query_map([post.id], |row| Author::from_row(row))?;
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
        manager.list_post_authors(self)
    }
}
