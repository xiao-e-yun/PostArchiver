use chrono::{DateTime, Utc};
use rusqlite::{params, ToSql};

use crate::{
    manager::{platform::PlatformIdOrRaw, PostArchiverConnection, PostArchiverManager},
    Author, AuthorId, FileMetaId,
};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Import or update an author record in the archive.
    ///
    /// Checks if an author with any of the given aliases exists in the archive.
    /// Updates the existing record if found, otherwise creates a new entry.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn import_author(&self, author: &UnsyncAuthor) -> Result<AuthorId, rusqlite::Error> {
        let aliases = author
            .aliases
            .iter()
            .map(|(platform, source, _link)| (platform, source.as_str()))
            .collect::<Vec<_>>();

        let exist = self.check_author(aliases.as_slice())?;

        match exist {
            Some(id) => self.import_author_by_update(id, author),
            None => self.import_author_by_create(author),
        }
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
        let updated = author.updated.unwrap_or_else(|| Utc::now());
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT INTO authors (name, updated) VALUES (?, ?) RETURNING id")?;

        let id: AuthorId = stmt.query_row(params![&author.name, &updated], |row| row.get(0))?;
        self.add_author_aliases(&id, &author.aliases)?;

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
        self.add_author_aliases(&id, &author.aliases)?;

        if let Some(updated) = author.updated {
            self.set_author_updated(id, &updated)?;
        }

        Ok(id)
    }
}

/// Represents an author that is not yet synced with the archive.
#[derive(Debug, Clone)]
pub struct UnsyncAuthor {
    name: String,
    updated: Option<DateTime<Utc>>,
    aliases: Vec<(PlatformIdOrRaw, String, Option<String>)>,
}

impl UnsyncAuthor {
    pub fn new(name: String) -> Self {
        Self {
            name,
            updated: None,
            aliases: Vec::new(),
        }
    }

    pub fn name(self, name: String) -> Self {
        Self { name, ..self }
    }
    pub fn aliases(self, aliases: Vec<UnsyncAlias>) -> Self {
        let aliases = aliases
            .into_iter()
            .map(|alias| (alias.platform, alias.source, alias.link))
            .collect();
        Self { aliases, ..self }
    }
    pub fn updated(self, updated: Option<DateTime<Utc>>) -> Self {
        Self { updated, ..self }
    }

    /// Sync the author with the archive.
    ///
    /// It will check if the author already exists in the archive by their alias.  
    /// If the author exists, it will update the existing entry. If not, it will create a new entry.
    ///
    /// # Examples
    /// ```rust
    /// use post_archiver::manager::PostArchiverManager;
    /// use post_archiver::importer::{UnsyncAuthor, UnsyncAlias};
    /// use post_archiver::{AuthorId};
    ///
    /// use chrono::Utc;
    ///
    /// // Open a manager
    /// let manager = PostArchiverManager::open_in_memory().unwrap();
    ///
    /// // Create an author
    /// let author = UnsyncAuthor::new("octocat".to_string())
    ///    .aliases(vec![UnsyncAlias::new(UNKNOWN_PLATFORM, "octocat")])
    ///    .updated(Some(Utc::now()))
    ///    .sync(&manager)
    ///    .unwrap();
    ///
    /// let archived_author = manager.get_author(&author.id).unwrap();
    /// assert_eq!(author, archived_author);
    pub fn sync<T>(self, manager: &PostArchiverManager<T>) -> Result<Author, rusqlite::Error>
    where
        T: PostArchiverConnection,
    {
        let id = manager.import_author(&self)?;
        let author = manager.get_author(&id)?;
        Ok(author)
    }
}

/// Represents an alias for an author that is not yet synced with the archive.
///
/// This is used to track different platforms or identifiers for the same author.
/// It can include links to their profiles or other relevant information.
#[derive(Debug, Clone)]
pub struct UnsyncAlias {
    link: Option<String>,
    platform: PlatformIdOrRaw,
    source: String,
}

impl UnsyncAlias {
    pub fn new<S: Into<String>>(platform: &PlatformIdOrRaw, source: S) -> Self {
        Self {
            link: None,
            source: source.into(),
            platform: platform.clone(),
        }
    }

    pub fn link<S: Into<String>>(mut self, link: S) -> Self {
        self.link = Some(link.into());
        self
    }
}
