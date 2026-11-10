use std::{fmt::Debug, ops::Deref};

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use crate::{
    error::Result,
    importer::{Importer, SavepointExt, UnsyncAuthor},
    manager::{PostArchiverBindExt, UpdateAuthor, UpdateCollection, UpdatePost},
    Comment, Content, PostId,
};

use super::{collection::UnsyncCollection, tag::UnsyncTag, UnsyncFileMeta};

impl<C: SavepointExt> Importer<'_, '_, C> {
    /// Import a post into the archive.
    ///
    /// If the post already exists (by source), it updates its title, platform, published date,
    ///
    /// # Errors
    ///
    /// Returns `Error` if there was an error accessing the database.
    pub fn import_post(&mut self, post: UnsyncPost) -> Result<PostId> {
        let mut sp = self.savepoint()?;

        let updated = post.updated.unwrap_or_else(Utc::now);
        let id: PostId = {
            let mut stmt = sp.conn.prepare_cached(
                "INSERT INTO posts (source, title, updated) VALUES (?, ?, ?) ON CONFLICT(source) DO UPDATE SET source = excluded.source RETURNING id",
            )?;
            stmt.query_row(params![post.source, post.title, updated], |row| row.get(0))?
        };

        let mut thumb = match post.thumb {
            Some(thumb) => Some(sp.import_file_meta(id, thumb)?),
            None => None,
        };

        let content = post
            .content
            .into_iter()
            .map(|content| {
                Ok(match content {
                    UnsyncContent::Text(text) => Content::Text(text.clone()),
                    UnsyncContent::File(file) => {
                        let need_thumb = thumb.is_none() && file.mime.starts_with("image/");
                        let file_meta = sp.import_file_meta(id, file.clone())?;
                        need_thumb.then(|| thumb = Some(file_meta));
                        Content::File(file_meta)
                    }
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let platform = sp.import_platform(post.platform)?;

        macro_rules! import_many {
            ($vec:expr => $method:ident) => {
                $vec.into_iter()
                    .map(|d| sp.$method(d))
                    .collect::<std::result::Result<Vec<_>, _>>()?
            };
        }

        let tags = import_many!(post.tags => import_tag);
        let collections = import_many!(post.collections => import_collection);
        let authors = import_many!(post.authors => import_author);

        let b = sp.conn.bind(id);
        b.update(
            UpdatePost::default()
                .title(post.title)
                .platform(Some(platform))
                .published(post.published.unwrap_or_else(Utc::now))
                .updated_by_latest(updated)
                .thumb(thumb)
                .content(content)
                .comments(post.comments),
        )?;

        b.add_tags(&tags)?;
        b.add_collections(&collections)?;
        b.add_authors(&authors)?;

        authors.iter().try_for_each(|&author| {
            sp.conn.bind(author).update(
                UpdateAuthor::default()
                    .thumb_by_latest()
                    .updated_by_latest(),
            )
        })?;

        collections.iter().try_for_each(|&collection| {
            sp.conn
                .bind(collection)
                .update(UpdateCollection::default().thumb_by_latest())
        })?;

        sp.commit()?;

        Ok(id)
    }
}

impl<C: Deref<Target = Connection>> Importer<'_, '_, C> {
    pub fn lookup_post(&self, source: &str) -> Result<Option<PostId>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT id FROM posts WHERE source = ?")?;
        let query = stmt.query_row(params![source], |row| row.get(0));
        Ok(query.optional()?)
    }

    pub fn lookup_post_with_updated(
        &self,
        source: &str,
        updated: &DateTime<Utc>,
    ) -> crate::error::Result<Option<PostId>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT id FROM posts WHERE source = ? AND updated >= ?")?;
        let query = stmt.query_row(params![source, updated], |row| row.get(0));
        Ok(query.optional()?)
    }
}

#[derive(Debug, Clone)]
/// Represents a post that is not yet synchronized with the archive.
pub struct UnsyncPost {
    /// The original URL of this post (e.g., `https://example.com/blog/1`)
    pub source: String,
    /// The title of the post
    pub title: String,
    /// The post's content items (text and file references)
    pub content: Vec<UnsyncContent>,
    /// Optional thumbnail image for the post
    pub thumb: Option<UnsyncFileMeta>,
    /// Comments on the post
    pub comments: Vec<Comment>,
    /// When the post was updated
    pub updated: Option<DateTime<Utc>>,
    /// When the post was published
    pub published: Option<DateTime<Utc>>,
    /// Platform associated with the post
    pub platform: String,
    /// Tags associated with the post
    pub tags: Vec<UnsyncTag>,
    /// The IDs of the author who created this post
    pub authors: Vec<UnsyncAuthor>,
    /// The collections this post belongs to
    pub collections: Vec<UnsyncCollection>,
}

impl UnsyncPost {
    pub fn new(
        platform: String,
        source: String,
        title: String,
        content: Vec<UnsyncContent>,
    ) -> Self {
        Self {
            source,
            title,
            content,
            thumb: None,
            comments: Vec::new(),
            updated: None,
            published: None,
            platform,
            tags: Vec::new(),
            authors: Vec::new(),
            collections: Vec::new(),
        }
    }

    pub fn source(self, source: String) -> Self {
        Self { source, ..self }
    }

    pub fn title(self, title: String) -> Self {
        Self { title, ..self }
    }

    pub fn content(self, content: Vec<UnsyncContent>) -> Self {
        Self { content, ..self }
    }

    pub fn thumb(self, thumb: Option<UnsyncFileMeta>) -> Self {
        Self { thumb, ..self }
    }

    pub fn comments(self, comments: Vec<Comment>) -> Self {
        Self { comments, ..self }
    }

    pub fn updated(self, updated: DateTime<Utc>) -> Self {
        Self {
            updated: Some(updated),
            ..self
        }
    }

    pub fn published(self, published: DateTime<Utc>) -> Self {
        Self {
            published: Some(published),
            ..self
        }
    }

    pub fn platform(self, platform: String) -> Self {
        Self { platform, ..self }
    }

    pub fn tags(self, tags: Vec<UnsyncTag>) -> Self {
        Self { tags, ..self }
    }

    pub fn authors(self, authors: Vec<UnsyncAuthor>) -> Self {
        Self { authors, ..self }
    }

    pub fn collections(self, collections: Vec<UnsyncCollection>) -> Self {
        Self {
            collections,
            ..self
        }
    }
}

#[derive(Debug, Clone)]
pub enum UnsyncContent {
    Text(String),
    File(UnsyncFileMeta),
}
