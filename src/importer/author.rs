use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    AuthorId, PlatformId,
};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Import an author into the archive.
    ///
    /// If the author already exists (by aliases), it updates their name, aliases, and updated date.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn import_author(&self, author: UnsyncAuthor) -> Result<AuthorId, rusqlite::Error> {
        // find by aliases
        let id = {
            let mut stmt = self.conn().prepare_cached(
                "SELECT target FROM author_aliases WHERE platform = ? AND source = ?",
            )?;
            let mut found: Option<AuthorId> = None;
            for alias in &author.aliases {
                if let Some(id) = stmt
                    .query_row(params![alias.platform, alias.source], |row| row.get(0))
                    .optional()?
                {
                    found = Some(id);
                    break;
                }
            }
            found
        };

        let aliases = author
            .aliases
            .into_iter()
            .map(|alias| (alias.source, alias.platform, alias.link))
            .collect::<Vec<_>>();

        match id {
            Some(id) => {
                let b = self.bind(id);
                b.set_name(author.name)?;

                if let Some(updated) = author.updated {
                    b.set_updated(updated)?;
                }

                b.add_aliases(aliases)?;

                Ok(id)
            }
            None => {
                // insert
                let mut stmt = self.conn().prepare_cached(
                    "INSERT INTO authors (name, updated) VALUES (?, ?) RETURNING id",
                )?;
                let id: AuthorId = stmt.query_row(
                    params![author.name, author.updated.unwrap_or_else(Utc::now)],
                    |row| row.get(0),
                )?;

                self.bind(id).add_aliases(aliases)?;

                Ok(id)
            }
        }
    }
}

/// Represents an author that is not yet synced with the archive.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnsyncAuthor {
    name: String,
    updated: Option<DateTime<Utc>>,
    aliases: Vec<UnsyncAlias>,
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
        Self { aliases, ..self }
    }
    pub fn updated(self, updated: Option<DateTime<Utc>>) -> Self {
        Self { updated, ..self }
    }
    /// Import it into the archive.
    ///
    /// If the author already exists (by aliases), it updates their name, aliases, and updated date.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn sync<T>(self, manager: &PostArchiverManager<T>) -> Result<AuthorId, rusqlite::Error>
    where
        T: PostArchiverConnection,
    {
        manager.import_author(self)
    }
}

/// Represents an alias for an author that is not yet synced with the archive.
///
/// This is used to track different platforms or identifiers for the same author.
/// It can include links to their profiles or other relevant information.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnsyncAlias {
    pub source: String,
    pub platform: PlatformId,
    pub link: Option<String>,
}

impl UnsyncAlias {
    pub fn new(platform: PlatformId, source: String) -> Self {
        Self {
            link: None,
            source,
            platform,
        }
    }

    pub fn source<S: Into<String>>(mut self, source: S) -> Self {
        self.source = source.into();
        self
    }

    pub fn platform(mut self, platform: PlatformId) -> Self {
        self.platform = platform;
        self
    }

    pub fn link<S: Into<String>>(mut self, link: S) -> Self {
        self.link = Some(link.into());
        self
    }
}
