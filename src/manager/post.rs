use chrono::{DateTime, Utc};
use rusqlite::params;

use crate::{
    manager::binded::Binded, manager::PostArchiverConnection, AuthorId, CollectionId, Comment,
    Content, FileMetaId, PlatformId, PostId, TagId,
};

//=============================================================
// Update / Delete
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, PostId, C> {
    /// Remove this post from the archive.
    ///
    /// This also removes all associated file metadata, author associations,
    /// tag associations, and collection associations.
    pub fn delete(&self) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM posts WHERE id = ?")?;
        stmt.execute([self.id()])?;
        Ok(())
    }

    /// Set this post's title.
    pub fn set_title(&self, title: String) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET title = ? WHERE id = ?")?;
        stmt.execute(params![title, self.id()])?;
        Ok(())
    }

    /// Set this post's source URL.
    pub fn set_source(&self, source: Option<String>) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET source = ? WHERE id = ?")?;
        stmt.execute(params![source, self.id()])?;
        Ok(())
    }

    /// Set this post's platform.
    pub fn set_platform(&self, platform: Option<PlatformId>) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET platform = ? WHERE id = ?")?;
        stmt.execute(params![platform, self.id()])?;
        Ok(())
    }

    /// Set this post's thumbnail.
    pub fn set_thumb(&self, thumb: Option<FileMetaId>) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET thumb = ? WHERE id = ?")?;
        stmt.execute(params![thumb, self.id()])?;
        Ok(())
    }

    /// Set this post's content.
    pub fn set_content(&self, content: Vec<Content>) -> Result<(), rusqlite::Error> {
        let content = serde_json::to_string(&content).unwrap();
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET content = ? WHERE id = ?")?;
        stmt.execute(params![content, self.id()])?;
        Ok(())
    }

    /// Set this post's comments.
    pub fn set_comments(&self, comments: Vec<Comment>) -> Result<(), rusqlite::Error> {
        let comments = serde_json::to_string(&comments).unwrap();
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET comments = ? WHERE id = ?")?;
        stmt.execute(params![comments, self.id()])?;
        Ok(())
    }

    /// Set this post's published timestamp.
    pub fn set_published(&self, published: DateTime<Utc>) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET published = ? WHERE id = ?")?;
        stmt.execute(params![published, self.id()])?;
        Ok(())
    }

    /// Set this post's updated timestamp.
    pub fn set_updated(&self, updated: DateTime<Utc>) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET updated = ? WHERE id = ?")?;
        stmt.execute(params![updated, self.id()])?;
        Ok(())
    }

    /// Set this post's updated timestamp only if the new value is more recent.
    pub fn set_updated_by_latest(&self, updated: DateTime<Utc>) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET updated = ? WHERE id = ? AND updated < ?")?;
        stmt.execute(params![updated, self.id(), updated])?;
        Ok(())
    }
}

//=============================================================
// Relations: Authors
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, PostId, C> {
    /// List all author IDs associated with this post.
    pub fn list_authors(&self) -> Result<Vec<AuthorId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT author FROM author_posts WHERE post = ?")?;
        let rows = stmt.query_map([self.id()], |row| row.get(0))?;
        rows.collect()
    }

    /// Associate one or more authors with this post.
    /// Duplicate associations are silently ignored.
    pub fn add_authors(&self, authors: &[AuthorId]) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT OR IGNORE INTO author_posts (author, post) VALUES (?, ?)")?;
        for author in authors {
            stmt.execute(params![author, self.id()])?;
        }
        Ok(())
    }

    /// Remove one or more authors from this post.
    pub fn remove_authors(&self, authors: &[AuthorId]) -> Result<(), rusqlite::Error> {
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
    pub fn list_tags(&self) -> Result<Vec<TagId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT tag FROM post_tags WHERE post = ?")?;
        let rows = stmt.query_map([self.id()], |row| row.get(0))?;
        rows.collect()
    }

    /// Associate one or more tags with this post.
    /// Duplicate associations are silently ignored.
    pub fn add_tags(&self, tags: &[TagId]) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT OR IGNORE INTO post_tags (post, tag) VALUES (?, ?)")?;
        for tag in tags {
            stmt.execute(params![self.id(), tag])?;
        }
        Ok(())
    }

    /// Remove one or more tags from this post.
    pub fn remove_tags(&self, tags: &[TagId]) -> Result<(), rusqlite::Error> {
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
    pub fn list_collections(&self) -> Result<Vec<CollectionId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT collection FROM collection_posts WHERE post = ?")?;
        let rows = stmt.query_map([self.id()], |row| row.get(0))?;
        rows.collect()
    }

    /// Associate one or more collections with this post.
    /// Duplicate associations are silently ignored.
    pub fn add_collections(&self, collections: &[CollectionId]) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "INSERT OR IGNORE INTO collection_posts (collection, post) VALUES (?, ?)",
        )?;
        for collection in collections {
            stmt.execute(params![collection, self.id()])?;
        }
        Ok(())
    }

    /// Remove one or more collections from this post.
    pub fn remove_collections(&self, collections: &[CollectionId]) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM collection_posts WHERE collection = ? AND post = ?")?;
        for collection in collections {
            stmt.execute(params![collection, self.id()])?;
        }
        Ok(())
    }
}
