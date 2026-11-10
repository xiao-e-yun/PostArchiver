use std::ops::Deref;

use rusqlite::{params, Connection, OptionalExtension};

use crate::{
    error::Result,
    importer::{Importer, SavepointExt},
    manager::{PostArchiverBindExt, UpdateAuthor},
    AuthorId, PlatformId,
};

impl<C: SavepointExt> Importer<'_, C> {
    pub fn import_author(&mut self, author: UnsyncAuthor) -> Result<AuthorId> {
        let id = self.lookup_author(author.clone())?;
        let aliases = author.aliases.into_iter().map(|v| v.into());
        let id = match id {
            Some(id) => {
                let sp = self.savepoint()?;
                let b = sp.conn.bind(id);
                b.update(UpdateAuthor::default().name(author.name.clone()))?;
                b.add_aliases(aliases)?;
                sp.commit()?;
                id
            }
            None => {
                let sp = self.conn.savepoint()?;
                let mut stmt =
                    sp.prepare_cached("INSERT INTO authors (name) VALUES (?) RETURNING id")?;

                let id: AuthorId = stmt.query_row(params![author.name], |row| row.get(0))?;
                sp.bind(id).add_aliases(aliases)?;
                drop(stmt);
                sp.commit()?;
                id
            }
        };

        Ok(id)
    }
}

impl<C: Deref<Target = Connection>> Importer<'_, C> {
    pub fn lookup_author(&self, author: UnsyncAuthor) -> Result<Option<AuthorId>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT target FROM author_aliases WHERE platform = ? AND source = ?",
        )?;
        for alias in &author.aliases {
            let query = stmt.query_row(params![alias.platform, alias.source], |row| row.get(0));
            if let Some(author_id) = query.optional()? {
                return Ok(Some(author_id));
            }
        }
        Ok(None)
    }
}

/// Represents an author that is not yet synced with the archive.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnsyncAuthor {
    name: String,
    aliases: Vec<UnsyncAlias>,
}

impl UnsyncAuthor {
    pub fn new(name: String) -> Self {
        Self {
            name,
            aliases: Vec::new(),
        }
    }

    pub fn name(self, name: String) -> Self {
        Self { name, ..self }
    }
    pub fn aliases(self, aliases: Vec<UnsyncAlias>) -> Self {
        Self { aliases, ..self }
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

impl From<UnsyncAlias> for (String, PlatformId, Option<String>) {
    fn from(val: UnsyncAlias) -> Self {
        (val.source, val.platform, val.link)
    }
}
