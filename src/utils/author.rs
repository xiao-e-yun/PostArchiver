use rusqlite::{params_from_iter, OptionalExtension};

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    Alias, Author, AuthorId, FileMeta, Link, Post,
};

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
    pub fn check_author(&self, alias: &[String]) -> Result<Option<AuthorId>, rusqlite::Error> {
        if alias.is_empty() {
            return Ok(None);
        }

        let query_array = "?,".repeat(alias.len() - 1) + "?";

        let mut stmt = self.conn().prepare(&format!(
            "SELECT target FROM author_alias WHERE source IN ({})",
            query_array
        ))?;

        stmt.query_row(params_from_iter(alias), |row| row.get(0))
            .optional()
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
            let links: String = row.get("links")?;
            let links: Vec<Link> = serde_json::from_str(&links).unwrap();
            Ok(Author {
                links,
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
    ///     let aliases = manager.get_author_alias(&author_id)?;
    ///     for alias in aliases {
    ///         println!("Author alias: {} -> {}", alias.source, alias.target);
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn get_author_alias(&self, author: &AuthorId) -> Result<Vec<Alias>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM author_alias WHERE target = ?")?;
        let tags = stmt.query_map([author], |row| {
            Ok(Alias {
                source: row.get("source")?,
                target: row.get("target")?,
            })
        })?;
        tags.collect()
    }
}

/// Trait to get author
pub trait GetAuthor {
    /// Get author id
    fn author_id(&self) -> &AuthorId;
    /// Get author
    fn author(&self, manager: &PostArchiverManager) -> Result<Author, rusqlite::Error> {
        manager.get_author(self.author_id())
    }
    /// Get author alias
    fn author_alias(&self, manager: &PostArchiverManager) -> Result<Vec<Alias>, rusqlite::Error> {
        manager.get_author_alias(self.author_id())
    }
}

impl GetAuthor for AuthorId {
    fn author_id(&self) -> &AuthorId {
        self
    }
}

impl GetAuthor for Author {
    fn author_id(&self) -> &AuthorId {
        &self.id
    }
}

impl GetAuthor for Post {
    fn author_id(&self) -> &AuthorId {
        &self.author
    }
}

impl GetAuthor for FileMeta {
    fn author_id(&self) -> &AuthorId {
        &self.author
    }
}

impl GetAuthor for Alias {
    fn author_id(&self) -> &AuthorId {
        &self.target
    }
}

#[cfg(feature = "importer")]
impl GetAuthor for crate::importer::UnsyncPost {
    fn author_id(&self) -> &AuthorId {
        &self.author
    }
}

#[cfg(feature = "importer")]
impl GetAuthor for crate::importer::PartialSyncPost {
    fn author_id(&self) -> &AuthorId {
        &self.author
    }
}
