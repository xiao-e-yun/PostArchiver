use std::collections::HashSet;

use chrono::{DateTime, Utc};
use rusqlite::{params, params_from_iter, OptionalExtension, ToSql};

use crate::{
    alias::Alias,
    manager::{PostArchiverConnection, PostArchiverManager},
    Author, AuthorId, FileMetaId, Link,
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
    /// Import or update an author record in the archive.
    ///
    /// Checks if an author with any of the given aliases exists in the archive.
    /// Updates the existing record if found, otherwise creates a new entry.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn import_author(&self, author: &UnsyncAuthor) -> Result<AuthorId, rusqlite::Error> {
        let exist = self.check_author(&author.alias.as_slice())?;

        let id = match exist {
            Some(id) => self.import_author_by_update(id, author)?,
            None => self.import_author_by_create(author)?,
        };

        Ok(id)
    }
    /// Create a new author entry in the archive.
    ///
    /// Creates a new entry regardless of whether the author already exists. To check for
    /// existing authors first, use [`import_author`](Self::import_author).
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn import_author_by_create(
        &self,
        author: &UnsyncAuthor,
    ) -> Result<AuthorId, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT INTO authors (name, links) VALUES (?, ?) RETURNING id")?;

        let links = serde_json::to_string(&author.links).unwrap();
        let id: AuthorId = stmt.query_row(params![&author.name, &links], |row| row.get(0))?;

        if let Some(updated) = author.updated {
            self.set_author_updated(id, &updated)?;
        };

        self.set_author_alias_by_merge(&id, &author.alias)?;
        Ok(id)
    }
    /// Update an existing author entry in the archive.
    ///
    /// Updates the author's information including name, links, aliases and timestamps.
    /// To check for existing authors first, use [`import_author`](Self::import_author).
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Panics
    ///
    /// Panics if the specified author does not exist in the archive.
    pub fn import_author_by_update(
        &self,
        id: AuthorId,
        author: &UnsyncAuthor,
    ) -> Result<AuthorId, rusqlite::Error> {
        self.set_author_name(&id, &author.name)?;
        self.set_author_links_by_merge(id, &author.links)?;
        self.set_author_alias_by_merge(&id, &author.alias)?;

        if let Some(updated) = author.updated {
            self.set_author_updated(id, &updated)?;
        }

        Ok(id)
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
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let author_id = AuthorId(1);
    ///     
    ///     let author = manager.get_author(&author_id)?;
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

    /// Merge the author alias by their id.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_author_alias_by_merge<S>(
        &self,
        author: &AuthorId,
        alias: &[S],
    ) -> Result<(), rusqlite::Error>
    where
        S: ToSql,
    {
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT OR IGNORE INTO author_alias (source, target) VALUES (?, ?)")?;
        for alias in alias.iter() {
            stmt.execute(params![alias, &author])?;
        }
        Ok(())
    }
    /// Set the author name by their id.
    pub fn set_author_name<S>(&self, author: &AuthorId, name: S) -> Result<(), rusqlite::Error>
    where
        S: AsRef<str> + ToSql,
    {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET name = ? WHERE id = ?")?;
        stmt.execute(params![name, &author])?;
        Ok(())
    }
    /// Set or remove an author's thumbnail image.
    ///
    /// Associates a file metadata ID as the author's thumbnail image, or removes it
    /// by passing `None`. The specified file must already exist in the archive.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::{AuthorId, FileMetaId};
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let author_id = AuthorId(1);
    ///     let thumb_file_id = FileMetaId(1);
    ///     
    ///     // Set a thumbnail
    ///     manager.set_author_thumb(author_id, Some(thumb_file_id))?;
    ///     
    ///     // Remove the thumbnail
    ///     manager.set_author_thumb(author_id, None)?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn set_author_thumb(
        &self,
        author: AuthorId,
        thumb: Option<FileMetaId>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET thumb = ? WHERE id = ?")?;
        stmt.execute(params![thumb, &author])?;
        Ok(())
    }
    /// Set the author's thumbnail to their most recent post's thumbnail.
    ///
    /// Find the most recent post by this author that has a thumbnail and use it as
    /// the author's thumbnail image.
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
    ///     // Update author thumbnail from latest post
    ///     manager.set_author_thumb_by_latest(author_id)?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn set_author_thumb_by_latest(&self, author: AuthorId) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET thumb = (SELECT thumb FROM posts WHERE author = ? AND thumb IS NOT NULL ORDER BY updated DESC LIMIT 1) WHERE id = ?")?;
        stmt.execute(params![author, author])?;
        Ok(())
    }
    /// Set the author links by their id.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_author_links(
        &self,
        author: AuthorId,
        links: &[Link],
    ) -> Result<(), rusqlite::Error> {
        let links = serde_json::to_string(links).unwrap();
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET links = ? WHERE id = ?")?;
        stmt.execute(params![links, &author])?;
        Ok(())
    }
    /// Merge the author links by their id.
    pub fn set_author_links_by_merge(
        &self,
        author: AuthorId,
        links: &[Link],
    ) -> Result<(), rusqlite::Error> {
        if links.is_empty() {
            return Ok(());
        }

        let mut stmt = self
            .conn()
            .prepare_cached("SELECT links FROM authors WHERE id = ?")?;
        let old_links: String = stmt.query_row(&[&author], |row| row.get(0))?;
        let old_links: HashSet<Link> = serde_json::from_str(&old_links).unwrap();
        let links = HashSet::from_iter(links.iter().cloned());
        let links = links.union(&old_links).cloned().collect::<Vec<_>>();
        self.set_author_links(author, &links)
    }
    /// Set an author's last updated timestamp.
    ///
    /// Update the timestamp indicating when this author's information was last modified.
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
    /// # use chrono::Utc;
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let author_id = AuthorId(1);
    ///     let now = Utc::now();
    ///     
    ///     manager.set_author_updated(author_id, &now)?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn set_author_updated(
        &self,
        author: AuthorId,
        updated: &DateTime<Utc>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET updated = ? WHERE id = ?")?;
        stmt.execute(params![updated, &author])?;
        Ok(())
    }
    /// Set the author's last updated time to match their most recent post.
    ///
    /// Find the author's most recently updated post and use its timestamp as the
    /// author's last updated time.
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
    ///     manager.set_author_updated_by_latest(author_id)?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn set_author_updated_by_latest(&self, author: AuthorId) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET updated = (SELECT updated FROM posts WHERE author = ? ORDER BY updated DESC LIMIT 1) WHERE id = ?")?;
        stmt.execute(params![author, author])?;
        Ok(())
    }
}

/// Represents an author that is not yet synced with the archive.
#[derive(Debug, Clone)]
pub struct UnsyncAuthor {
    name: String,
    links: Vec<Link>,
    alias: Vec<String>,
    updated: Option<DateTime<Utc>>,
}

impl UnsyncAuthor {
    pub fn new(name: String) -> Self {
        Self {
            name: name.into(),
            alias: Vec::new(),
            links: Vec::new(),
            updated: None,
        }
    }

    pub fn name(self, name: String) -> Self {
        Self { name, ..self }
    }
    pub fn alias(self, alias: Vec<String>) -> Self {
        Self { alias, ..self }
    }
    pub fn links(self, links: Vec<Link>) -> Self {
        Self { links, ..self }
    }
    pub fn updated(self, updated: Option<DateTime<Utc>>) -> Self {
        Self { updated, ..self }
    }

    /// Sync the author with the archive.
    ///
    /// It will check if the author already exists in the archive by their alias.  
    /// If the author exists, it will update the existing entry. If not, it will create a new entry.
    ///
    /// # Exampless
    /// ```rust
    /// use post_archiver::manager::PostArchiverManager;
    /// use post_archiver::importer::UnsyncAuthor;
    /// use post_archiver::{AuthorId, Link};
    ///
    /// use chrono::Utc;
    ///
    /// // Open a manager
    /// let manager = PostArchiverManager::open_in_memory().unwrap();
    ///
    /// // Create an author
    /// let (author, _) = UnsyncAuthor::new("octocat".to_string())
    ///    .alias(vec!["github:octocat".to_string()])
    ///    .links(vec![Link::new("github", "https://octodex.github.com/")])
    ///    .updated(Some(Utc::now()))
    ///    .sync(&manager)
    ///    .unwrap();
    ///
    /// let archived_author = manager.get_author(&author.id).unwrap();
    /// assert_eq!(author, archived_author);
    pub fn sync<T>(
        self,
        manager: &PostArchiverManager<T>,
    ) -> Result<(Author, Vec<Alias>), rusqlite::Error>
    where
        T: PostArchiverConnection,
    {
        let id = manager.import_author(&self)?;
        let author = manager.get_author(&id)?;
        let alias = manager.get_author_alias(&id)?;
        Ok((author, alias))
    }
}
