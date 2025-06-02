use rusqlite::{params, OptionalExtension};

use crate::{
    utils::tag::{PlatformTagIdOrRaw, PlatformTagLike, TagIdOrRaw, TagLike},
    PlatformTag, Post, PostId, Tag,
};

use super::{PostArchiverConnection, PostArchiverManager};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Retrieve all tags.
    pub fn list_tags(&self) -> Result<Vec<Tag>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM tags")?;
        let tags = stmt.query_map([], |row| Tag::from_row(row))?;
        tags.collect()
    }

    /// Retrieve all platform tags.
    pub fn list_platform_tags(&self) -> Result<Vec<PlatformTag>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM platform_tags")?;
        let tags = stmt.query_map([], |row| PlatformTag::from_row(row))?;
        tags.collect()
    }

    /// Retrieve all tags associated with a post.
    pub fn list_post_tags(&self, id: &PostId) -> Result<Vec<Tag>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT tags.* FROM tags INNER JOIN post_tags ON post_tags.tag = tags.id WHERE post_tags.post = ?")?;
        let tags = stmt.query_map([id], |row| Tag::from_row(row))?;
        tags.collect()
    }

    /// Retrieve all platform tags associated with a post.
    pub fn list_post_platform_tags(
        &self,
        id: &PostId,
    ) -> Result<Vec<PlatformTag>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT tags.* FROM tags INNER JOIN post_platform_tags ON post_platform_tags.tag = tags.id WHERE post_platform_tags.post = ?")?;
        let tags = stmt.query_map([id], |row| PlatformTag::from_row(row))?;
        tags.collect()
    }

    /// Get a tag by its id or name.
    pub fn get_tag(&self, tag: &impl TagLike) -> Result<Option<Tag>, rusqlite::Error> {
        match tag.id() {
            Some(tag_id) => {
                let mut stmt = self
                    .conn()
                    .prepare_cached("SELECT * FROM tags WHERE id = ?")?;
                stmt.query_row([tag_id], |row| Tag::from_row(row))
            }
            None => {
                let name = tag.raw().unwrap();
                let mut stmt = self
                    .conn()
                    .prepare_cached("SELECT * FROM tags WHERE name = ?")?;
                stmt.query_row([name], |row| Tag::from_row(row))
            }
        }
        .optional()
    }

    /// Get a platform tag by its id or name.
    pub fn get_platform_tag(
        &self,
        tag: &impl PlatformTagLike,
    ) -> Result<Option<PlatformTag>, rusqlite::Error> {
        match tag.id() {
            Some(tag_id) => {
                let mut stmt = self
                    .conn()
                    .prepare_cached("SELECT * FROM platform_tags WHERE id = ?")?;
                stmt.query_row([tag_id], |row| PlatformTag::from_row(row))
            }
            None => {
                let (platform, name) = tag.raw().unwrap();
                let Some(platform) = self.get_platform(platform)? else {
                    return Ok(None);
                };
                let mut stmt = self.conn().prepare_cached(
                    "SELECT * FROM platform_tags WHERE platform = ? AND name = ?",
                )?;
                stmt.query_row(params![platform.id, name], |row| PlatformTag::from_row(row))
            }
        }
        .optional()
    }
}

impl Post {
    pub fn tags(&self, manager: &PostArchiverManager) -> Result<Vec<Tag>, rusqlite::Error> {
        manager.list_post_tags(&self.id)
    }
    pub fn platform_tags(
        &self,
        manager: &PostArchiverManager,
    ) -> Result<Vec<PlatformTag>, rusqlite::Error> {
        manager.list_post_platform_tags(&self.id)
    }
}
