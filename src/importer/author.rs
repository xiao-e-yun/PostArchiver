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
    /// Check if the author exists in the archive by their alias.
    ///
    /// # Example
    /// ```rust
    /// use post_archiver::manager::PostArchiverManager;
    /// use post_archiver::importer::UnsyncAuthor;
    /// use post_archiver::AuthorId;
    ///
    /// // Open a manager
    /// let manager = PostArchiverManager::open_in_memory().unwrap();
    ///
    /// // Check if the author not exists
    /// let id: Option<AuthorId> = manager.check_author(&["github:octocat".to_string()]).unwrap();
    ///
    /// assert_eq!(id, None);
    ///
    /// // Create an author
    /// let author = UnsyncAuthor::new("octocat".to_string())
    ///    .alias(vec!["github:octocat".to_string()]);
    ///
    /// // Import the author
    /// let (author, _) = author.sync(&manager).unwrap();
    ///
    /// // Check if the author exists
    /// let id: Option<AuthorId> = manager.check_author(&["github:octocat".to_string()]).unwrap();
    ///
    /// assert_eq!(id, Some(author.id));
    /// ```
    ///
    /// # Reference
    /// [Alias](crate::alias::Alias) - Represents an alias mapping for an author
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
    /// Import an author into the archive.
    ///
    /// It will check if the author already exists in the archive by their alias.  
    /// If the author exists, it will update the existing entry. If not, it will create a new entry.  
    ///
    /// # Example
    /// ```rust
    /// use post_archiver::manager::PostArchiverManager;
    /// use post_archiver::importer::UnsyncAuthor;
    /// use post_archiver::AuthorId;
    ///
    /// // Open a manager
    /// let manager = PostArchiverManager::open_in_memory().unwrap();
    ///
    /// // Create an author
    /// let author = UnsyncAuthor::new("octocat".to_string())
    ///    .alias(vec!["github:octocat".to_string()]);
    ///
    /// // Import the author
    /// let id: AuthorId = manager.import_author(&author).unwrap();
    /// ```
    pub fn import_author(&self, author: &UnsyncAuthor) -> Result<AuthorId, rusqlite::Error> {
        let exist = self.check_author(&author.alias.as_slice())?;

        let id = match exist {
            Some(id) => self.import_author_by_update(id, author)?,
            None => self.import_author_by_create(author)?,
        };

        Ok(id)
    }
    /// Import an author into the archive by creating a new entry.
    ///
    /// # Note
    /// This function will create a new entry for the author, even if they already exist in the archive.  
    /// [`import_author`](Self::import_author) will check if the author already exists and update the entry if they do.  
    ///
    /// # Example
    /// ```rust
    /// use post_archiver::manager::PostArchiverManager;
    /// use post_archiver::importer::UnsyncAuthor;
    /// use post_archiver::AuthorId;
    ///
    /// // Open a manager
    /// let manager = PostArchiverManager::open_in_memory().unwrap();
    ///
    /// // Create an author
    /// let author = UnsyncAuthor::new("octocat".to_string())
    ///    .alias(vec!["github:octocat".to_string()]);
    ///
    /// // Import the author
    /// let id: AuthorId = manager.import_author_by_create(&author).unwrap();
    /// ```
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
    /// Import an author into the archive by updating an existing entry.
    ///
    /// # Note
    /// This function will update entry for the author.  
    /// [`import_author`](Self::import_author) will check if the author already exists and update the entry if they do.  
    ///
    /// # Panic
    /// This function will panic if the author does not exist in the archive.
    ///
    /// # Example
    /// ```rust
    /// use post_archiver::manager::PostArchiverManager;
    /// use post_archiver::importer::UnsyncAuthor;
    /// use post_archiver::AuthorId;
    ///
    /// // Open a manager
    /// let manager = PostArchiverManager::open_in_memory().unwrap();
    ///
    /// // Create an author
    /// let author = UnsyncAuthor::new("octocat".to_string())
    ///    .alias(vec!["github:octocat".to_string()]);
    ///
    /// // Import the author
    /// let id: AuthorId = manager.import_author(&author).unwrap();
    ///
    /// // Next time, we can just update the author
    /// let author = UnsyncAuthor::new("octocatdog".to_string())
    ///    .alias(vec!["github:octocatdog".to_string()]);
    ///
    /// // Update the author
    /// let id: AuthorId = manager.import_author_by_update(id, &author).unwrap();
    ///
    /// // get author
    /// let author = manager.get_author(&id).unwrap();
    /// assert_eq!(author.name, "octocatdog");
    /// ```
    ///
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

    /// Get the author by their id.
    ///
    /// # Example
    /// ```rust
    /// use post_archiver::manager::PostArchiverManager;
    /// use post_archiver::importer::UnsyncAuthor;
    /// use post_archiver::AuthorId;
    ///
    /// // Open a manager
    /// let manager = PostArchiverManager::open_in_memory().unwrap();
    ///
    /// // Create an author and import it
    /// let (author, _) = UnsyncAuthor::new("octocat".to_string())
    ///    .alias(vec!["github:octocat".to_string()])
    ///    .sync(&manager)
    ///    .unwrap();
    ///
    /// // Get the author by their id
    /// let author = manager.get_author(&author.id).unwrap();
    /// assert_eq!(author.name, "octocat");
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

    /// Get alias of the author by their id.
    ///
    /// # Example
    /// ```rust
    /// use post_archiver::manager::PostArchiverManager;
    /// use post_archiver::importer::UnsyncAuthor;
    /// use post_archiver::AuthorId;
    ///
    /// // Open a manager
    /// let manager = PostArchiverManager::open_in_memory().unwrap();
    ///
    /// // Create an author and import it
    /// let (author, _) = UnsyncAuthor::new("octocat".to_string())
    ///    .alias(vec!["github:octocat".to_string(), "x:octocat".to_string()])
    ///    .sync(&manager)
    ///    .unwrap();
    ///
    /// // Get the author by their id
    /// let alias = manager.get_author_alias(&author.id).unwrap();
    ///
    /// assert_eq!(alias.len(), 2);
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
    /// # Example
    /// ```rust
    /// use post_archiver::manager::PostArchiverManager;
    /// use post_archiver::importer::UnsyncAuthor;
    /// use post_archiver::AuthorId;
    ///
    /// // Open a manager
    /// let manager = PostArchiverManager::open_in_memory().unwrap();
    ///
    /// // Create an author and import it
    /// let (author, _) = UnsyncAuthor::new("octocat".to_string())
    ///    .alias(vec!["github:octocat".to_string(), "x:octocat".to_string()])
    ///    .sync(&manager)
    ///    .unwrap();
    ///
    /// // new alias
    /// let alias = vec!["x:octocat".to_string(), "stackoverflow:octocat".to_string()];
    ///
    /// // Merge the author alias
    /// manager.set_author_alias_by_merge(&author.id, &alias).unwrap();
    ///
    /// // Check the author alias
    /// let alias = manager.get_author_alias(&author.id).unwrap();
    /// assert_eq!(alias.len(), 3);
    /// ```
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
    ///
    /// # Example
    /// ```rust
    /// use post_archiver::manager::PostArchiverManager;
    /// use post_archiver::importer::UnsyncAuthor;
    /// use post_archiver::AuthorId;
    ///
    /// // Open a manager
    /// let manager = PostArchiverManager::open_in_memory().unwrap();
    ///
    /// // Create an author and import it
    /// let (author, _) = UnsyncAuthor::new("octocat".to_string())
    ///    .sync(&manager)
    ///    .unwrap();
    ///
    /// // Set the author name
    /// manager.set_author_name(&author.id, "octocatdog").unwrap();
    ///
    /// // Get the author by their id
    /// let author = manager.get_author(&author.id).unwrap();
    /// assert_eq!(author.name, "octocatdog");
    /// ```
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
    /// Set the author thumb by their id.
    ///
    /// # Example
    /// ```rust
    /// use post_archiver::manager::PostArchiverManager;
    /// use post_archiver::importer::{UnsyncAuthor, UnsyncPost, UnsyncFileMeta, ImportFileMetaMethod};
    /// use post_archiver::AuthorId;
    /// use post_archiver::FileMetaId;
    ///
    /// use std::collections::HashMap;
    ///
    /// // Open a manager
    /// let manager = PostArchiverManager::open_in_memory().unwrap();
    ///
    /// // Create an author and import it
    /// let (author, _) = UnsyncAuthor::new("octocat".to_string())
    ///    .sync(&manager)
    ///    .unwrap();
    ///
    /// // Create a post and thumb
    /// let (post, _) = UnsyncPost::new(author.id)
    ///    .thumb(Some(UnsyncFileMeta {
    ///        filename: "thumb.png".to_string(),
    ///        mime: "image/png".to_string(),
    ///        extra: HashMap::new(),
    ///        method: ImportFileMetaMethod::Custom,
    ///    }))
    ///    .sync(&manager)
    ///    .unwrap();
    ///
    /// // Get the thumb id
    /// let thumb = manager.get_author(&author.id).unwrap().thumb;
    /// assert!(thumb.is_some());
    ///
    /// // Set the author thumb
    /// manager.set_author_thumb(author.id, None).unwrap();
    ///
    /// // Get the thumb again
    /// let thumb = manager.get_author(&author.id).unwrap().thumb;
    /// assert_eq!(thumb, None);
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
    /// Set the author thumb by the latest post.
    ///
    /// # Example
    /// ```rust
    /// use post_archiver::manager::PostArchiverManager;
    /// use post_archiver::importer::{UnsyncAuthor, UnsyncPost, UnsyncFileMeta, ImportFileMetaMethod};
    /// use post_archiver::AuthorId;
    /// use post_archiver::FileMetaId;
    ///
    /// use std::collections::HashMap;
    /// use chrono::{Utc, Duration};
    ///
    /// // Open a manager
    /// let manager = PostArchiverManager::open_in_memory().unwrap();
    ///
    /// // Create an author and import it
    /// let (author, _) = UnsyncAuthor::new("octocat".to_string())
    ///    .sync(&manager)
    ///    .unwrap();
    ///
    /// // Define updated time
    /// let updated = Utc::now();
    ///
    /// // Create a post and thumb
    /// let (first_post, _) = UnsyncPost::new(author.id)
    ///    .thumb(Some(UnsyncFileMeta {
    ///        filename: "thumb.png".to_string(),
    ///        mime: "image/png".to_string(),
    ///        extra: HashMap::new(),
    ///        method: ImportFileMetaMethod::Custom,
    ///    }))
    ///    .updated(updated)
    ///    .sync(&manager)
    ///    .unwrap();
    ///
    /// // Create another post and thumb, to update the author thumb
    /// let (second_post, _) = UnsyncPost::new(author.id)
    ///    .thumb(Some(UnsyncFileMeta {
    ///         filename: "thumb2.png".to_string(),
    ///         mime: "image/png".to_string(),
    ///         extra: HashMap::new(),
    ///         method: ImportFileMetaMethod::Custom,
    ///    }))
    ///    .updated(updated + Duration::seconds(1))
    ///    .sync(&manager)
    ///    .unwrap();
    ///
    /// // Get the author thumb
    /// let thumb = manager.get_author(&author.id).unwrap().thumb;
    /// assert_eq!(thumb, second_post.thumb);
    ///
    /// // Update the first post updated time
    /// manager.set_post_updated(first_post.id, &(updated + Duration::seconds(2))).unwrap();
    /// manager.set_author_thumb_by_latest(author.id).unwrap();
    ///
    /// // Get the author thumb
    /// let thumb = manager.get_author(&author.id).unwrap().thumb;
    /// assert_eq!(thumb, first_post.thumb);
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
    /// # Example
    /// ```rust
    /// use post_archiver::manager::PostArchiverManager;
    /// use post_archiver::importer::UnsyncAuthor;
    /// use post_archiver::AuthorId;
    /// use post_archiver::Link;
    ///
    /// // Open a manager
    /// let manager = PostArchiverManager::open_in_memory().unwrap();
    ///
    /// // Create an author and import it
    /// let links = vec![Link::new("github", "https://octodex.github.com/")];
    /// let (author, _) = UnsyncAuthor::new("octocat".to_string())
    ///    .links(links.clone())
    ///    .sync(&manager)
    ///    .unwrap();
    ///
    /// assert_eq!(author.links, links);
    ///
    /// // Set the author links
    /// let links = vec![
    ///    Link::new("example", "https://example.com/"),
    ///    Link::new("example2", "https://example2.com/"),
    /// ];
    /// manager.set_author_links(author.id, &links).unwrap();
    ///
    /// // Get the author by their id
    /// let author = manager.get_author(&author.id).unwrap();
    /// assert_eq!(author.links, links);
    /// ```
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
    ///
    /// # Example
    /// ```rust
    /// use post_archiver::manager::PostArchiverManager;
    /// use post_archiver::importer::UnsyncAuthor;
    /// use post_archiver::AuthorId;
    /// use post_archiver::Link;
    ///
    /// // Open a manager
    /// let manager = PostArchiverManager::open_in_memory().unwrap();
    ///
    /// // Create an author and import it
    /// let (author, _) = UnsyncAuthor::new("octocat".to_string())
    ///    .links(vec![Link::new("github", "https://octodex.github.com/")])
    ///    .sync(&manager)
    ///    .unwrap();
    ///
    /// assert_eq!(author.links.len(), 1);
    ///
    /// // Set the author links
    /// manager.set_author_links_by_merge(author.id, &vec![
    ///    Link::new("example", "https://example.com/"),
    ///    Link::new("example2", "https://example2.com/"),
    /// ]).unwrap();
    ///
    /// // Get the author by their id
    /// let author = manager.get_author(&author.id).unwrap();
    ///
    /// assert_eq!(author.links.len(), 3);
    /// ```
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
    /// Set the author updated time by their id.
    ///
    /// # Example
    /// ```rust
    /// use post_archiver::manager::PostArchiverManager;
    /// use post_archiver::importer::UnsyncAuthor;
    /// use post_archiver::AuthorId;
    /// use chrono::Utc;
    ///
    /// // Open a manager
    /// let manager = PostArchiverManager::open_in_memory().unwrap();
    ///
    /// let updated = Utc::now();
    ///
    /// // Create an author and import it
    /// let (author, _) = UnsyncAuthor::new("octocat".to_string())
    ///    .updated(Some(updated))
    ///    .sync(&manager)
    ///    .unwrap();
    ///
    /// assert_eq!(author.updated, updated);
    ///
    /// // Set the author updated time to next second
    /// let updated = updated + chrono::Duration::seconds(1);
    /// manager.set_author_updated(author.id, &updated).unwrap();
    ///
    /// // Get the author by their id
    /// let author = manager.get_author(&author.id).unwrap();
    /// assert_eq!(author.updated, updated);
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
    /// Set the author updated by the latest post.
    ///
    /// # Example
    /// ```rust
    /// use post_archiver::manager::PostArchiverManager;
    /// use post_archiver::importer::{UnsyncAuthor, UnsyncPost, UnsyncFileMeta, ImportFileMetaMethod};
    /// use post_archiver::AuthorId;
    /// use post_archiver::FileMetaId;
    /// use chrono::{Utc, Duration};
    ///
    /// use std::collections::HashMap;
    ///
    /// // Open a manager
    /// let manager = PostArchiverManager::open_in_memory().unwrap();
    ///
    /// // Define updated time
    /// let updated = Utc::now();
    ///
    /// // Create an author and import it
    /// let (author, _) = UnsyncAuthor::new("octocat".to_string())
    ///    .sync(&manager)
    ///    .unwrap();
    ///
    /// // Create a post
    /// let (first_post, _) = UnsyncPost::new(author.id)
    ///    .updated(updated)
    ///    .sync(&manager)
    ///    .unwrap();
    ///
    /// // Create another post, to update the author updated time
    /// let (second_post, _) = UnsyncPost::new(author.id)
    ///    .updated(updated + Duration::seconds(1))
    ///    .sync(&manager)
    ///    .unwrap();
    ///
    /// // Get the author updated
    /// let author_updated = manager.get_author(&author.id).unwrap().updated;
    /// assert_eq!(author_updated, second_post.updated);
    ///
    /// // Update the first post updated time
    /// let new_updated = updated + Duration::seconds(2);
    /// manager.set_post_updated(first_post.id, &new_updated).unwrap();
    /// manager.set_author_updated_by_latest(author.id).unwrap();
    ///
    /// // Get the author updated
    /// let author_updated = manager.get_author(&author.id).unwrap().updated;
    /// assert_eq!(author_updated, new_updated);
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
    /// # Example
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
