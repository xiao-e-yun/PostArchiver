use crate::{
    manager::binded::Binded, manager::PostArchiverConnection, utils::macros::AsTable, PlatformId,
    PostId, Tag, TagId,
};

/// Builder for updating a tag's fields.
///
/// Fields left as `None` are not modified.
/// For nullable columns (platform), use `Some(None)` to clear the value.
#[derive(Debug, Clone, Default)]
pub struct UpdateTag {
    pub name: Option<String>,
    pub platform: Option<Option<PlatformId>>,
}

impl UpdateTag {
    /// Set the tag's name.
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
    /// Set or clear the tag's platform.
    pub fn platform(mut self, platform: Option<PlatformId>) -> Self {
        self.platform = Some(platform);
        self
    }
}

//=============================================================
// FindTag trait (kept for importer compatibility)
//=============================================================
pub trait FindTag {
    fn name(&self) -> &str;
    fn platform(&self) -> Option<PlatformId> {
        None
    }
}

impl FindTag for &str {
    fn name(&self) -> &str {
        self
    }
}

impl FindTag for (&str, PlatformId) {
    fn name(&self) -> &str {
        self.0
    }
    fn platform(&self) -> Option<PlatformId> {
        Some(self.1)
    }
}

impl FindTag for (&str, Option<PlatformId>) {
    fn name(&self) -> &str {
        self.0
    }
    fn platform(&self) -> Option<PlatformId> {
        self.1
    }
}

//=============================================================
// Update / Delete
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, TagId, C> {
    /// Get this tag's current data from the database.
    pub fn value(&self) -> Result<Tag, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM tags WHERE id = ?")?;
        stmt.query_row([self.id()], Tag::from_row)
    }

    /// Remove this tag from the archive.
    ///
    /// This will also remove all post-tag relationships, but will not delete the posts themselves.
    pub fn delete(self) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM tags WHERE id = ?")?;
        stmt.execute([self.id()])?;
        Ok(())
    }

    /// Apply a batch of field updates to this tag in a single SQL statement.
    ///
    /// Only fields set on `update` (i.e. `Some(...)`) are written to the database.
    pub fn update(&self, update: UpdateTag) -> Result<(), rusqlite::Error> {
        use rusqlite::types::ToSql;

        let mut sets: Vec<&str> = Vec::new();
        let mut params: Vec<&dyn ToSql> = Vec::new();

        macro_rules! push {
            ($field:expr, $col:expr) => {
                if let Some(ref v) = $field {
                    sets.push($col);
                    params.push(v);
                }
            };
        }

        push!(update.name, "name = ?");
        push!(update.platform, "platform = ?");

        if sets.is_empty() {
            return Ok(());
        }

        let id = self.id();
        params.push(&id);

        let sql = format!("UPDATE tags SET {} WHERE id = ?", sets.join(", "));
        self.conn().execute(&sql, params.as_slice())?;
        Ok(())
    }
}

//=============================================================
// Relations: Posts
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, TagId, C> {
    /// List all post IDs associated with this tag.
    pub fn list_posts(&self) -> Result<Vec<PostId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT post FROM post_tags WHERE tag = ?")?;
        let rows = stmt.query_map([self.id()], |row| row.get(0))?;
        rows.collect()
    }
}
