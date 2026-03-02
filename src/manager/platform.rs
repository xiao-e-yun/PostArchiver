use crate::{
    error::Result, manager::binded::Binded, manager::PostArchiverConnection,
    utils::macros::AsTable, Platform, PlatformId, PostId, TagId,
};

/// Builder for updating a platform's fields.
///
/// Fields left as `None` are not modified.
#[derive(Debug, Clone, Default)]
pub struct UpdatePlatform {
    pub name: Option<String>,
}

impl UpdatePlatform {
    /// Set the platform's name.
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}

//=============================================================
// Update / Delete
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, PlatformId, C> {
    /// Get this platform's current data from the database.
    pub fn value(&self) -> Result<Platform> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM platforms WHERE id = ?")?;
        Ok(stmt.query_row([self.id()], Platform::from_row)?)
    }

    /// Remove this platform from the archive.
    ///
    /// This operation will also set the platform to UNKNOWN for all author aliases and posts.
    /// Tags associated with the platform will be deleted.
    pub fn delete(self) -> Result<()> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM platforms WHERE id = ?")?;
        stmt.execute([self.id()])?;
        Ok(())
    }

    /// Apply a batch of field updates to this platform in a single SQL statement.
    ///
    /// Only fields set on `update` (i.e. `Some(...)`) are written to the database.
    pub fn update(&self, update: UpdatePlatform) -> Result<()> {
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

        if sets.is_empty() {
            return Ok(());
        }

        let id = self.id();
        params.push(&id);

        let sql = format!("UPDATE platforms SET {} WHERE id = ?", sets.join(", "));
        self.conn().execute(&sql, params.as_slice())?;
        Ok(())
    }
}

//=============================================================
// Relations: Tags / Posts
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, PlatformId, C> {
    /// List all tag IDs associated with this platform.
    pub fn list_tags(&self) -> Result<Vec<TagId>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM tags WHERE platform = ?")?;
        let rows = stmt.query_map([self.id()], |row| row.get(0))?;
        rows.collect::<std::result::Result<_, _>>()
            .map_err(Into::into)
    }

    /// List all post IDs associated with this platform.
    pub fn list_posts(&self) -> Result<Vec<PostId>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM posts WHERE platform = ?")?;
        let rows = stmt.query_map([self.id()], |row| row.get(0))?;
        rows.collect::<std::result::Result<_, _>>()
            .map_err(Into::into)
    }
}
