use chrono::{DateTime, Utc};
use rusqlite::params;

use crate::{
    error::Result, manager::binded::Binded, manager::PostArchiverConnection,
    utils::macros::AsTable, AuthorId, CollectionId, Comment, Content, FileMetaId, PlatformId, Post,
    PostId, TagId,
};

/// Specifies how to update a post's `updated` timestamp.
#[derive(Debug, Clone)]
pub enum PostUpdated {
    /// Unconditionally set to this value.
    Set(DateTime<Utc>),
    /// Set to this value only if it is more recent than the current value.
    ByLatest(DateTime<Utc>),
}

/// Builder for updating a post's fields.
///
/// Fields left as `None` are not modified.
/// For nullable columns (source, platform, thumb), use `Some(None)` to clear the value.
#[derive(Debug, Clone, Default)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub source: Option<Option<String>>,
    pub platform: Option<Option<PlatformId>>,
    pub thumb: Option<Option<FileMetaId>>,
    pub content: Option<Vec<Content>>,
    pub comments: Option<Vec<Comment>>,
    pub published: Option<DateTime<Utc>>,
    pub updated: Option<PostUpdated>,
}

impl UpdatePost {
    /// Set the title.
    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }
    /// Set or clear the source URL.
    pub fn source(mut self, source: Option<String>) -> Self {
        self.source = Some(source);
        self
    }
    /// Set or clear the platform.
    pub fn platform(mut self, platform: Option<PlatformId>) -> Self {
        self.platform = Some(platform);
        self
    }
    /// Set or clear the thumbnail.
    pub fn thumb(mut self, thumb: Option<FileMetaId>) -> Self {
        self.thumb = Some(thumb);
        self
    }
    /// Replace the content list.
    pub fn content(mut self, content: Vec<Content>) -> Self {
        self.content = Some(content);
        self
    }
    /// Replace the comments list.
    pub fn comments(mut self, comments: Vec<Comment>) -> Self {
        self.comments = Some(comments);
        self
    }
    /// Set the published timestamp.
    pub fn published(mut self, published: DateTime<Utc>) -> Self {
        self.published = Some(published);
        self
    }
    /// Unconditionally set the updated timestamp.
    pub fn updated(mut self, updated: DateTime<Utc>) -> Self {
        self.updated = Some(PostUpdated::Set(updated));
        self
    }
    /// Set the updated timestamp only if `updated` is more recent than the stored value.
    pub fn updated_by_latest(mut self, updated: DateTime<Utc>) -> Self {
        self.updated = Some(PostUpdated::ByLatest(updated));
        self
    }
}

//=============================================================
// Update / Delete
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, PostId, C> {
    /// Get this post's current data from the database.
    pub fn value(&self) -> Result<Post> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM posts WHERE id = ?")?;
        Ok(stmt.query_row([self.id()], Post::from_row)?)
    }

    /// Remove this post from the archive.
    ///
    /// This also removes all associated file metadata, author associations,
    /// tag associations, and collection associations.
    pub fn delete(self) -> Result<()> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM posts WHERE id = ?")?;
        stmt.execute([self.id()])?;
        Ok(())
    }

    /// Apply a batch of field updates to this post in a single SQL statement.
    ///
    /// Only fields set on `update` (i.e. `Some(...)`) are written to the database.
    pub fn update(&self, update: UpdatePost) -> Result<()> {
        use rusqlite::types::ToSql;

        // Pre-serialize JSON fields so they outlive the params slice.
        let content_json = update.content.map(|c| serde_json::to_string(&c).unwrap());
        let comments_json = update.comments.map(|c| serde_json::to_string(&c).unwrap());

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

        push!(update.title, "title = ?");
        push!(update.source, "source = ?");
        push!(update.platform, "platform = ?");
        push!(update.thumb, "thumb = ?");
        push!(content_json, "content = ?");
        push!(comments_json, "comments = ?");
        push!(update.published, "published = ?");

        match &update.updated {
            Some(PostUpdated::Set(t)) => {
                sets.push("updated = ?");
                params.push(t);
            }
            Some(PostUpdated::ByLatest(t)) => {
                sets.push("updated = MAX(updated, ?)");
                params.push(t);
            }
            None => {}
        }

        if sets.is_empty() {
            return Ok(());
        }

        let id = self.id();
        params.push(&id);

        let sql = format!("UPDATE posts SET {} WHERE id = ?", sets.join(", "));
        self.conn().execute(&sql, params.as_slice())?;
        Ok(())
    }

    /// List all file metadata IDs associated with this post.
    pub fn list_file_metas(&self) -> Result<Vec<FileMetaId>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM file_metas WHERE post = ?")?;
        let rows = stmt.query_map([self.id()], |row| row.get(0))?;
        rows.collect::<std::result::Result<_, _>>()
            .map_err(Into::into)
    }
}

//=============================================================
// Relations: Authors
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, PostId, C> {
    /// List all author IDs associated with this post.
    pub fn list_authors(&self) -> Result<Vec<AuthorId>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT author FROM author_posts WHERE post = ?")?;
        let rows = stmt.query_map([self.id()], |row| row.get(0))?;
        rows.collect::<std::result::Result<_, _>>()
            .map_err(Into::into)
    }

    /// Associate one or more authors with this post.
    /// Duplicate associations are silently ignored.
    pub fn add_authors(&self, authors: &[AuthorId]) -> Result<()> {
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT OR IGNORE INTO author_posts (author, post) VALUES (?, ?)")?;
        for author in authors {
            stmt.execute(params![author, self.id()])?;
        }
        Ok(())
    }

    /// Remove one or more authors from this post.
    pub fn remove_authors(&self, authors: &[AuthorId]) -> Result<()> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM author_posts WHERE post = ? AND author = ?")?;
        for author in authors {
            stmt.execute(params![self.id(), author])?;
        }
        Ok(())
    }
}

//=============================================================
// Relations: Tags
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, PostId, C> {
    /// List all tag IDs associated with this post.
    pub fn list_tags(&self) -> Result<Vec<TagId>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT tag FROM post_tags WHERE post = ?")?;
        let rows = stmt.query_map([self.id()], |row| row.get(0))?;
        rows.collect::<std::result::Result<_, _>>()
            .map_err(Into::into)
    }

    /// Associate one or more tags with this post.
    /// Duplicate associations are silently ignored.
    pub fn add_tags(&self, tags: &[TagId]) -> Result<()> {
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT OR IGNORE INTO post_tags (post, tag) VALUES (?, ?)")?;
        for tag in tags {
            stmt.execute(params![self.id(), tag])?;
        }
        Ok(())
    }

    /// Remove one or more tags from this post.
    pub fn remove_tags(&self, tags: &[TagId]) -> Result<()> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM post_tags WHERE post = ? AND tag = ?")?;
        for tag in tags {
            stmt.execute(params![self.id(), tag])?;
        }
        Ok(())
    }
}

//=============================================================
// Relations: Collections
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, PostId, C> {
    /// List all collection IDs associated with this post.
    pub fn list_collections(&self) -> Result<Vec<CollectionId>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT collection FROM collection_posts WHERE post = ?")?;
        let rows = stmt.query_map([self.id()], |row| row.get(0))?;
        rows.collect::<std::result::Result<_, _>>()
            .map_err(Into::into)
    }

    /// Associate one or more collections with this post.
    /// Duplicate associations are silently ignored.
    pub fn add_collections(&self, collections: &[CollectionId]) -> Result<()> {
        let mut stmt = self.conn().prepare_cached(
            "INSERT OR IGNORE INTO collection_posts (collection, post) VALUES (?, ?)",
        )?;
        for collection in collections {
            stmt.execute(params![collection, self.id()])?;
        }
        Ok(())
    }

    /// Remove one or more collections from this post.
    pub fn remove_collections(&self, collections: &[CollectionId]) -> Result<()> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM collection_posts WHERE collection = ? AND post = ?")?;
        for collection in collections {
            stmt.execute(params![collection, self.id()])?;
        }
        Ok(())
    }
}
