use rusqlite::{params, OptionalExtension};

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    Alias, Author, AuthorId, Post,
};

use super::platform::{PlatformIdOrRaw, PlatformLike};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Look up an author by their alias list.
    ///
    /// Searches the database for an author that matches any of the provided aliases.
    /// Returns the first matching author's ID, or `None` if no match is found.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error querying the database.
    pub fn check_author<S>(
        &self,
        aliases: &[(&impl PlatformLike, S)],
    ) -> Result<Option<AuthorId>, rusqlite::Error>
    where
        S: AsRef<str>,
    {
        if aliases.is_empty() {
            return Ok(None);
        }

        let mut stmt = self.conn().prepare_cached(
            "SELECT target FROM author_aliases WHERE platform = ? AND source = ?",
        )?;

        for (platform, alias) in aliases {
            let Some(platform) = self.get_platform(platform.clone())? else {
                continue;
            };
            if let Some(id) = stmt
                .query_row(params![platform.id, alias.as_ref()], |row| row.get(0))
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
    pub fn get_author(&self, author: &AuthorId) -> Result<Author, rusqlite::Error> {
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
    pub fn get_author_aliases(&self, author: &AuthorId) -> Result<Vec<Alias>, rusqlite::Error> {
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
    pub fn get_author_posts(&self, author: &AuthorId) -> Result<Vec<Post>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT posts.* FROM posts INNER JOIN author_posts ON author_posts.post = posts.id WHERE author_posts.author = ?")?;
        let posts = stmt.query_map([author], |row| Post::from_row(row))?;
        posts.collect()
    }
}

impl Author {
    /// Retrieve all aliases associated with this author.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn aliases(&self, manager: &PostArchiverManager) -> Result<Vec<Alias>, rusqlite::Error> {
        manager.get_author_aliases(&self.id)
    }
    /// Retrieve all posts associated with this author.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn posts(&self, manager: &PostArchiverManager) -> Result<Vec<Post>, rusqlite::Error> {
        manager.get_author_posts(&self.id)
    }
}

impl Post {
    /// Retrieve all authors associated with this post.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn authors(&self, manager: &PostArchiverManager) -> Result<Vec<Author>, rusqlite::Error> {
        manager.get_post_authors(self)
    }
}
