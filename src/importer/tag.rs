use std::hash::Hash;

use rusqlite::params;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    PlatformId, TagId,
};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Import a tag into the archive.
    ///
    /// If the tag already exists, it returns the existing ID.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn import_tag(&self, tag: UnsyncTag) -> Result<TagId, rusqlite::Error> {
        // find
        if let Some(id) = self.find_tag(&tag.name, tag.platform)? {
            return Ok(id);
        }

        // insert
        let mut ins_stmt = self
            .conn()
            .prepare_cached("INSERT INTO tags (name, platform) VALUES (?, ?) RETURNING id")?;
        ins_stmt.query_row(params![tag.name, tag.platform], |row| row.get(0))
    }

    /// Import multiple tags into the archive.
    ///
    /// If a tag already exists, it returns the existing ID.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn import_tags(
        &self,
        tags: impl IntoIterator<Item = UnsyncTag>,
    ) -> Result<Vec<TagId>, rusqlite::Error> {
        tags.into_iter()
            .map(|tag| self.import_tag(tag))
            .collect::<Result<Vec<TagId>, rusqlite::Error>>()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnsyncTag {
    pub name: String,
    pub platform: Option<PlatformId>,
}
