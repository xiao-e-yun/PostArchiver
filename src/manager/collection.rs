use rusqlite::params;

use crate::{
    error::Result,
    manager::{binded::Binded, PostArchiverConnection},
    query::FromQuery,
    Collection, CollectionId, FileMetaId, PostId,
};

/// Specifies how to update a collection's thumbnail.
#[derive(Debug, Clone)]
pub enum CollectionThumb {
    /// Set to an explicit value (or clear with `None`).
    Set(Option<FileMetaId>),
    /// Set to the thumb of the most recently updated post in this collection.
    ByLatest,
}

/// Builder for updating a collection's fields.
///
/// Fields left as `None` are not modified.
#[derive(Debug, Clone, Default)]
pub struct UpdateCollection {
    pub name: Option<String>,
    pub source: Option<Option<String>>,
    pub thumb: Option<CollectionThumb>,
}

impl UpdateCollection {
    /// Set the collection's name.
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
    /// Set or clear the source URL.
    pub fn source(mut self, source: Option<String>) -> Self {
        self.source = Some(source);
        self
    }
    /// Set or clear the thumbnail.
    pub fn thumb(mut self, thumb: Option<FileMetaId>) -> Self {
        self.thumb = Some(CollectionThumb::Set(thumb));
        self
    }
    /// Set the thumbnail to the latest post's thumb in this collection.
    pub fn thumb_by_latest(mut self) -> Self {
        self.thumb = Some(CollectionThumb::ByLatest);
        self
    }
}

//=============================================================
// Update / Delete
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, CollectionId, C> {
    /// Get this collection's current data from the database.
    pub fn value(&self) -> Result<Collection> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM collections WHERE id = ?")?;
        Ok(stmt.query_row([self.id()], Collection::from_row)?)
    }

    /// Remove this collection from the archive.
    ///
    /// Also removes all collection-post relationships.
    pub fn delete(self) -> Result<()> {
        self.conn()
            .execute("DELETE FROM collections WHERE id = ?", [self.id()])?;
        Ok(())
    }

    /// Apply a batch of field updates to this collection in a single SQL statement.
    ///
    /// Only fields set on `update` (i.e. `Some(...)`) are written to the database.
    pub fn update(&self, update: UpdateCollection) -> Result<()> {
        use rusqlite::types::ToSql;

        let id = self.id();
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
        push!(update.source, "source = ?");

        match &update.thumb {
            Some(CollectionThumb::Set(v)) => {
                sets.push("thumb = ?");
                params.push(v);
            }
            Some(CollectionThumb::ByLatest) => {
                sets.push("thumb = (SELECT posts.thumb FROM posts INNER JOIN collection_posts ON collection_posts.post = posts.id WHERE collection_posts.collection = ? AND posts.thumb IS NOT NULL ORDER BY posts.updated DESC LIMIT 1)");
                params.push(&id);
            }
            None => {}
        }

        if sets.is_empty() {
            return Ok(());
        }

        params.push(&id);

        let sql = format!("UPDATE collections SET {} WHERE id = ?", sets.join(", "));
        self.conn().execute(&sql, params.as_slice())?;
        Ok(())
    }
}

//=============================================================
// Relations: Posts
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, CollectionId, C> {
    /// List all post IDs in this collection.
    pub fn list_posts(&self) -> Result<Vec<PostId>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT post FROM collection_posts WHERE collection = ?")?;
        let rows = stmt.query_map([self.id()], |row| row.get(0))?;
        rows.collect::<std::result::Result<_, _>>()
            .map_err(Into::into)
    }

    /// Add posts to this collection.
    pub fn add_posts(&self, posts: &[PostId]) -> Result<()> {
        let mut stmt = self.conn().prepare_cached(
            "INSERT OR IGNORE INTO collection_posts (collection, post) VALUES (?, ?)",
        )?;
        for post in posts {
            stmt.execute(params![self.id(), post])?;
        }
        Ok(())
    }

    /// Remove posts from this collection.
    pub fn remove_posts(&self, posts: &[PostId]) -> Result<()> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM collection_posts WHERE collection = ? AND post = ?")?;
        for post in posts {
            stmt.execute(params![self.id(), post])?;
        }
        Ok(())
    }
}
