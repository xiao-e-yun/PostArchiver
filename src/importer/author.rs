use std::collections::HashSet;

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension, ToSql};

use crate::{alias::AuthorAlias, Author, AuthorId, FileMetaId, Link};

use super::{ImportConnection, PostArchiverImporter};

impl<T> PostArchiverImporter<T>
where
    T: ImportConnection,
{
    pub fn check_author(&self, alias: &[String]) -> Result<Option<AuthorId>, rusqlite::Error> {
        if alias.is_empty() {
            return Ok(None);
        }

        let query_array = "?,".repeat(alias.len() - 1) + "?";

        let mut stmt = self
        .conn()
        .prepare(&format!("SELECT target FROM author_alias WHERE source IN ({})",query_array))?;
    
        stmt.query_row(params_from_iter(alias), |row| row.get(0))
            .optional()
    }
    pub fn import_author(&self, author: &UnsyncAuthor) -> Result<AuthorId, rusqlite::Error> {
        let exist = self.check_author(&author.alias.as_slice())?;

        let id = match exist {
            Some(id) => self.import_author_by_update(id, author)?,
            None => self.import_author_by_create(author)?,
        };

        Ok(id)
    }
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

        self.set_author_alias(&id, &author.alias)?;
        Ok(id)
    }
    pub fn import_author_by_update(
        &self,
        id: AuthorId,
        author: &UnsyncAuthor,
    ) -> Result<AuthorId, rusqlite::Error> {
        self.set_author_links_by_merge(id, &author.links)?;
        self.set_author_alias(&id, &author.alias)?;

        if let Some(updated) = author.updated {
            self.set_author_updated(id, &updated)?;
        }

        Ok(id)
    }

    // Getters
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

    pub fn get_author_alias(&self, author: &AuthorId) -> Result<Vec<AuthorAlias>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM author_alias WHERE target = ?")?;
        let tags = stmt.query_map([author], |row| {
            Ok(AuthorAlias {
                source: row.get("source")?,
                target: row.get("target")?,
            })
        })?;
        tags.collect()
    }

    // Setters
    pub fn set_author_alias<S>(&self, author: &AuthorId, alias: &[S]) -> Result<(), rusqlite::Error>
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
    pub fn set_author_thumb<S>(
        &self,
        author: AuthorId,
        thumb: FileMetaId,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET thumb = ? WHERE id = ?")?;
        stmt.execute(params![thumb, &author])?;
        Ok(())
    }
    pub fn set_author_thumb_by_latest(&self, author: AuthorId) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET thumb = (SELECT thumb FROM posts WHERE author = ? AND thumb IS NOT NULL ORDER BY updated DESC LIMIT 1) WHERE id = ?")?;
        stmt.execute(params![author, author])?;
        Ok(())
    }
    pub fn set_author_links(
        &self,
        author: &AuthorId,
        links: &[Link],
    ) -> Result<(), rusqlite::Error> {
        let links = serde_json::to_string(links).unwrap();
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET links = ? WHERE id = ?")?;
        stmt.execute(params![links, &author])?;
        Ok(())
    }
    pub fn set_author_links_by_merge(
        &self,
        author: AuthorId,
        links: &[Link],
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT links FROM authors WHERE id = ?")?;
        let old_links: String = stmt.query_row(&[&author], |row| row.get(0))?;
        let old_links: HashSet<Link> = serde_json::from_str(&old_links).unwrap();
        let links = HashSet::from_iter(links.iter().cloned());
        let links = links.union(&old_links).cloned().collect::<Vec<_>>();
        self.set_author_links(&author, &links)
    }
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
    pub fn set_author_updated_by_latest(&self, author: AuthorId) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET updated = (SELECT updated FROM posts WHERE author = ? ORDER BY updated DESC LIMIT 1) WHERE id = ?")?;
        stmt.execute(params![author, author])?;
        Ok(())
    }
}

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

    pub fn sync<T>(
        self,
        importer: &PostArchiverImporter<T>,
    ) -> Result<(Author, Vec<AuthorAlias>), rusqlite::Error>
    where
        T: ImportConnection,
    {
        let id = importer.import_author(&self)?;
        let author = importer.get_author(&id)?;
        let alias = importer.get_author_alias(&id)?;
        Ok((author, alias))
    }
}
