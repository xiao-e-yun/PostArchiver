use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, Utc};
use rusqlite::params;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    AuthorId, Comment, Content, FileMetaId, Post, PostId, PostTagId,
};

use super::file_meta::{ImportFileMetaMethod, UnsyncFileMeta};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Create or update a post's metadata in the archive.
    ///
    /// Takes a post's metadata and either creates a new entry or updates an existing one  
    /// if a post with the same source already exists. This only updates metadata  
    /// use [`import_post`](Self::import_post) to import the complete post with content.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::importer::UnsyncPost;
    /// # use post_archiver::AuthorId;
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let author_id = AuthorId(1);
    ///     
    ///     let post = UnsyncPost::new(author_id)
    ///         .title("My First Post".to_string())
    ///         .source(Some("https://blog.example.com/post/1".to_string()));
    ///         
    ///     let partial_post = manager.import_post_meta(post)?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn import_post_meta(&self, post: UnsyncPost) -> Result<PartialSyncPost, rusqlite::Error> {
        let exist = if let Some(source) = &post.source {
            self.check_post(source)?
        } else {
            None
        };

        let post = match exist {
            Some(id) => self.import_post_meta_by_update(id, post),
            None => self.import_post_meta_by_create(post),
        };

        post
    }
    /// Create a new post entry in the archive.
    ///
    /// Creates a new post entry with the given metadata. Unlike [`import_post_meta`],
    /// this always creates a new entry even if a post with the same source exists.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::importer::UnsyncPost;
    /// # use post_archiver::AuthorId;
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let author_id = AuthorId(1);
    ///     
    ///     let post = UnsyncPost::new(author_id)
    ///         .title("My First Post".to_string());
    ///         
    ///     let partial_post = manager.import_post_meta_by_create(post)?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn import_post_meta_by_create(
        &self,
        post: UnsyncPost,
    ) -> Result<PartialSyncPost, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "INSERT INTO posts (author, source, title, content ,thumb, comments, updated, published) VALUES (?, ?, ?, '[]', null, ?, ?, ?) RETURNING id",
        )?;
        let comments = serde_json::to_string(&post.comments).unwrap();
        let id = stmt.query_row(
            params![
                &post.author,
                &post.source,
                &post.title,
                &comments,
                &post.updated,
                &post.published
            ],
            |row| row.get(0),
        )?;

        let tags = self.import_tags(&post.tags)?;
        self.set_post_tags(id, &tags)?;

        Ok(PartialSyncPost::new(post.author, id, post))
    }
    /// Update an existing post's metadata in the archive.
    ///
    /// Updates a post's metadata including title, source, tags, and timestamps.
    /// Unlike [`import_post_meta_by_create`], this updates an existing post.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::importer::UnsyncPost;
    /// # use post_archiver::{AuthorId, PostId};
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let author_id = AuthorId(1);
    ///     let post_id = PostId(1);
    ///     
    ///     let updated_post = UnsyncPost::new(author_id)
    ///         .title("Updated Title".to_string());
    ///         
    ///     let partial_post = manager.import_post_meta_by_update(post_id, updated_post)?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn import_post_meta_by_update(
        &self,
        id: PostId,
        post: UnsyncPost,
    ) -> Result<PartialSyncPost, rusqlite::Error> {
        // sync tags
        let tags = self.import_tags(&post.tags)?;
        self.set_post_tags(id, &tags)?;

        // sync other fields
        self.set_post_source(id, &post.source)?;
        self.set_post_title(id, &post.title)?;
        self.set_post_comments(id, &post.comments)?;
        self.set_post_updated_by_latest(id, &post.updated)?;
        self.set_post_published_by_latest(id, &post.published)?;
        Ok(PartialSyncPost::new(post.author, id, post))
    }

    /// Complete a post's import by processing its content and files.
    ///
    /// Takes a partially imported post and processes its content and file references
    /// to create a fully imported post entry in the archive.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Panics
    ///
    /// * When a referenced file is not found in the provided files map
    /// * When a thumbnail image reference is not found in the files map
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::{AuthorId, PostId, FileMetaId};
    /// # use post_archiver::importer::{PartialSyncPost, UnsyncPost};
    /// # use std::collections::HashMap;
    /// # use chrono::Utc;
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     
    ///     let post = UnsyncPost::new(AuthorId(1))
    ///         .title("My Post".to_string());
    ///     let partial_post = manager.import_post_meta_by_create(post)?;
    ///     
    ///     let mut files = HashMap::new();
    ///     files.insert("image.jpg".to_string(), FileMetaId(1));
    ///     
    ///     let complete_post = manager.import_post(partial_post, &files)?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn import_post(
        &self,
        post: PartialSyncPost,
        files: &HashMap<String, FileMetaId>,
    ) -> Result<PostId, rusqlite::Error> {
        let content: Vec<Content> = post
            .content
            .into_iter()
            .map(|content| match content {
                UnsyncContent::Text(text) => Content::Text(text),
                UnsyncContent::File(file) => {
                    Content::File(*files.get(&file.filename).expect("file unynced"))
                }
            })
            .collect();

        self.set_post_content(post.id, &content)?;

        let thumb = post
            .thumb
            .map(|thumb| *files.get(&thumb.filename).expect("thumb unynced"));

        self.set_post_thumb(post.id, &thumb)?;

        Ok(post.id)
    }

    // Setters

    /// Associate one or more tags with a post.
    ///
    /// Creates tag associations between a post and the provided tag IDs.
    /// Duplicate associations are silently ignored.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_tags(&self, post: PostId, tags: &[PostTagId]) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT OR IGNORE INTO post_tags (post, tag) VALUES (?, ?)")?;
        for tag in tags {
            stmt.execute(params![post, tag])?;
        }
        Ok(())
    }
    /// Set a post's content.
    ///
    /// Replaces the entire content of a post with new text and file entries.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_content(
        &self,
        post: PostId,
        content: &Vec<Content>,
    ) -> Result<(), rusqlite::Error> {
        let content = serde_json::to_string(content).unwrap();

        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET content = ? WHERE id = ?")?;
        stmt.execute(params![content, post])?;
        Ok(())
    }
    /// Set or clear a post's source URL.
    ///
    /// Sets the source identifier for a post, or removes it by passing `None`.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_source(
        &self,
        post: PostId,
        source: &Option<String>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET source = ? WHERE id = ?")?;
        stmt.execute(params![source, post])?;
        Ok(())
    }
    /// Update a post's title.
    ///
    /// Sets a new title for the specified post.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_title(&self, post: PostId, title: &str) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET title = ? WHERE id = ?")?;
        stmt.execute(params![title, post])?;
        Ok(())
    }
    /// Set or remove a post's thumbnail image.
    ///
    /// Associates a file metadata ID as the post's thumbnail, or removes it by passing `None`.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_thumb(
        &self,
        post: PostId,
        thumb: &Option<FileMetaId>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET thumb = ? WHERE id = ?")?;
        stmt.execute(params![thumb, post])?;
        Ok(())
    }
    /// Replace all comments on a post.
    ///
    /// Updates the post with a new set of comments, replacing any existing ones.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_comments(
        &self,
        post: PostId,
        comments: &Vec<Comment>,
    ) -> Result<(), rusqlite::Error> {
        let comments = serde_json::to_string(comments).unwrap();

        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET comments = ? WHERE id = ?")?;
        stmt.execute(params![comments, post])?;
        Ok(())
    }
    /// Update when a post was last modified.
    ///
    /// Sets the timestamp indicating when this post was last changed.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_updated(
        &self,
        post: PostId,
        updated: &DateTime<Utc>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET updated = ? WHERE id = ?")?;
        stmt.execute(params![updated, post])?;
        Ok(())
    }
    /// Update a post's timestamp if the new one is more recent.
    ///
    /// Only updates the last modified time if the new timestamp is more recent than the existing one.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_updated_by_latest(
        &self,
        post: PostId,
        updated: &DateTime<Utc>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET updated = ? WHERE id = ? AND updated < ?")?;
        stmt.execute(params![updated, post, updated])?;
        Ok(())
    }
    /// Set when a post was originally published.
    ///
    /// Updates the timestamp indicating when this post was first published.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_published(
        &self,
        post: PostId,
        published: &DateTime<Utc>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET published = ? WHERE id = ?")?;
        stmt.execute(params![published, post])?;
        Ok(())
    }
    /// Update a post's publish date if the new one is more recent.
    ///
    /// Only updates the publication timestamp if the new one is more recent than the existing one.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_published_by_latest(
        &self,
        post: PostId,
        published: &DateTime<Utc>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET published = ? WHERE id = ? AND published < ?")?;
        stmt.execute(params![published, post, published])?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
/// Represents a post that is not yet synchronized with the archive.
pub struct UnsyncPost {
    /// The ID of the author who created this post
    pub author: AuthorId,
    /// The original URL of this post (e.g., "https://example.com/blog/1")
    pub source: Option<String>,
    /// The title of the post
    pub title: String,
    /// The post's content items (text and file references)
    pub content: Vec<UnsyncContent>,
    /// Optional thumbnail image for the post
    pub thumb: Option<UnsyncFileMeta>,
    /// Comments on the post
    pub comments: Vec<Comment>,
    /// When the post was last updated
    pub updated: DateTime<Utc>,
    /// When the post was first published
    pub published: DateTime<Utc>,
    /// Tags associated with the post
    pub tags: Vec<(String, String)>,
}

impl UnsyncPost {
    pub fn new(author: AuthorId) -> Self {
        Self {
            author,
            source: None,
            title: String::new(),
            content: Vec::new(),
            thumb: None,
            comments: Vec::new(),
            updated: Utc::now(),
            published: Utc::now(),
            tags: Vec::new(),
        }
    }

    pub fn author(self, author: AuthorId) -> Self {
        Self { author, ..self }
    }
    pub fn source(self, source: Option<String>) -> Self {
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
        Self { updated, ..self }
    }
    pub fn published(self, published: DateTime<Utc>) -> Self {
        Self { published, ..self }
    }
    pub fn tags(self, tags: Vec<(impl Into<String>, impl Into<String>)>) -> Self {
        let tags = tags
            .into_iter()
            .map(|(cat, name)| (cat.into(), name.into()))
            .collect();
        Self { tags, ..self }
    }

    pub fn sync(
        self,
        manager: &PostArchiverManager<impl PostArchiverConnection>,
    ) -> Result<(Post, Vec<(PathBuf, ImportFileMetaMethod)>), rusqlite::Error> {
        let mut post = manager.import_post_meta(self)?;

        // select first image as thumb if not set
        post.thumb = post.thumb.clone().or_else(|| {
            post.content.iter().find_map(|content| match content {
                UnsyncContent::File(file) => {
                    if file.mime.starts_with("image/") {
                        Some(file.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            })
        });

        let metas = post.collect_files();
        let mut files = Vec::with_capacity(metas.capacity());

        let metas: HashMap<String, FileMetaId> = metas
            .into_iter()
            .map(|raw| {
                let (file, method) = manager.import_file_meta(post.author, post.id, raw.clone())?;

                files.push((manager.path.join(file.path()), method));
                Ok((raw.filename, file.id))
            })
            .collect::<Result<_, rusqlite::Error>>()?;

        let author = post.author;
        let post = manager.import_post(post, &metas)?;
        manager.set_author_updated_by_latest(author)?;
        manager.set_author_thumb_by_latest(author)?;

        let post = manager.get_post(&post)?;

        Ok((post, files))
    }
}

#[derive(Debug, Clone)]
pub enum UnsyncContent {
    Text(String),
    File(UnsyncFileMeta),
}

impl UnsyncContent {
    pub fn text(text: String) -> Self {
        Self::Text(text)
    }
    pub fn file(file: UnsyncFileMeta) -> Self {
        Self::File(file)
    }
}

/// Represents a partially synchronized post with metadata imported but content pending.
pub struct PartialSyncPost {
    /// The post's ID in the archive
    pub id: PostId,
    /// The ID of the author who created this post
    pub author: AuthorId,
    /// The original URL of this post (e.g., "https://example.com/blog/1")
    pub source: Option<String>,
    /// The title of the post
    pub title: String,
    /// The post's content items (text and file references)
    pub content: Vec<UnsyncContent>,
    /// Optional thumbnail image for the post
    pub thumb: Option<UnsyncFileMeta>,
    /// Comments on the post
    pub comments: Vec<Comment>,
    /// When the post was last updated
    pub updated: DateTime<Utc>,
    /// When the post was first published
    pub published: DateTime<Utc>,
}

impl PartialSyncPost {
    pub fn new(author: AuthorId, id: PostId, post: UnsyncPost) -> Self {
        Self {
            id,
            author,
            thumb: post.thumb,
            source: post.source,
            title: post.title,
            content: post.content,
            comments: post.comments,
            updated: post.updated,
            published: post.published,
        }
    }
    pub fn content(self, content: Vec<UnsyncContent>) -> Self {
        Self { content, ..self }
    }
    pub fn thumb(self, thumb: Option<UnsyncFileMeta>) -> Self {
        Self { thumb, ..self }
    }
    /// Collect all file references from this post.
    ///
    /// Gathers unique file metadata entries from:
    /// - Content items that are files (not text)
    /// - Thumbnail image (if present)
    ///
    /// Files are deduplicated by their filename.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::importer::{PartialSyncPost, UnsyncContent, UnsyncFileMeta, ImportFileMetaMethod};
    /// # use post_archiver::{AuthorId, PostId};
    /// # use std::collections::HashMap;
    /// fn example() {
    ///     let post = PartialSyncPost {
    ///         id: PostId(1),
    ///         author: AuthorId(1),
    ///         source: None,
    ///         title: "Test".to_string(),
    ///         content: vec![
    ///             UnsyncContent::File(UnsyncFileMeta {
    ///                 filename: "image.jpg".to_string(),
    ///                 mime: "image/jpeg".to_string(),
    ///                 extra: HashMap::new(),
    ///                 method: ImportFileMetaMethod::None,
    ///             }),
    ///             UnsyncContent::Text("some text".to_string()),
    ///             UnsyncContent::File(UnsyncFileMeta {
    ///                 filename: "doc.pdf".to_string(),
    ///                 mime: "application/pdf".to_string(),
    ///                 extra: HashMap::new(),
    ///                 method: ImportFileMetaMethod::None,
    ///             }),
    ///         ],
    ///         thumb: Some(UnsyncFileMeta {
    ///             filename: "thumb.jpg".to_string(),
    ///             mime: "image/jpeg".to_string(),
    ///             extra: HashMap::new(),
    ///             method: ImportFileMetaMethod::None,
    ///         }),
    ///         comments: vec![],
    ///         updated: chrono::Utc::now(),
    ///         published: chrono::Utc::now(),
    ///     };
    ///     
    ///     let files = post.collect_files();
    ///     assert_eq!(files.len(), 3)
    /// }
    /// ```
    pub fn collect_files(&self) -> Vec<UnsyncFileMeta> {
        let mut files = HashMap::new();
        for content in &self.content {
            match content {
                UnsyncContent::File(file) => {
                    files.insert(file.filename.clone(), file.clone());
                }
                _ => {}
            }
        }

        if let Some(thumb) = self
            .thumb
            .clone()
            .filter(|thumb| files.get(&thumb.filename).is_none())
        {
            files.insert(thumb.filename.clone(), thumb);
        }
        files.into_values().collect::<Vec<_>>()
    }
}
