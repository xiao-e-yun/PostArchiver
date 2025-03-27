use rusqlite::OptionalExtension;

use crate::{
    utils::manager::{PostArchiverConnection, PostArchiverManager},
    PostTagId,
};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    pub fn import_tag(&self, tag: &str) -> Result<PostTagId, rusqlite::Error> {
        // check cache
        if let Some(id) = self.cache.tags.lock().unwrap().get(tag) {
            return Ok(*id);
        }

        // check if tag exists
        let exist = self
            .conn()
            .query_row("SELECT id FROM tags WHERE name = ?", [&tag], |row| {
                row.get(0)
            })
            .optional()?;

        let id: PostTagId = match exist {
            // if tag exists, return the id
            Some(id) => {
                self.cache.tags.lock().unwrap().insert(tag.to_string(), id);
                return Ok(id);
            }
            // if tag does not exist, insert the tag and return the id
            None => self.conn().query_row(
                "INSERT INTO tags (name) VALUES (?) RETURNING id",
                [&tag],
                |row| row.get(0),
            ),
        }?;

        // insert into cache
        self.cache.tags.lock().unwrap().insert(tag.to_string(), id);

        Ok(id)
    }

    pub fn import_tags<S>(&self, tags: &[S]) -> Result<Vec<PostTagId>, rusqlite::Error>
    where
        S: AsRef<str>,
    {
        tags.iter()
            .map(|tag| self.import_tag(tag.as_ref()))
            .collect()
    }
}
