use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};

use crate::{
    utils::manager::{PostArchiverConnection, PostArchiverManager},
    AuthorId, Comment, Content, FileMetaId, Post, PostId, PostTagId,
};

use super::file_meta::{ImportFileMetaMethod, UnsyncFileMeta};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    pub fn check_post(&self, source: &String) -> Result<Option<PostId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM posts WHERE source = ?")?;

        stmt.query_row(params![source], |row| row.get(0)).optional()
    }
    pub fn check_post_with_updated(
        &self,
        source: &String,
        updated: &DateTime<Utc>,
    ) -> Result<Option<PostId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id, updated FROM posts WHERE source = ?")?;

        stmt.query_row::<(PostId, DateTime<Utc>), _, _>(params![source], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .optional()
        .map(|query| {
            query.and_then(|(id, last_update)| {
                if &last_update >= updated {
                    Some(id)
                } else {
                    None
                }
            })
        })
    }
    pub fn import_post_meta(&self, post: UnsyncPost) -> Result<PartialSyncPost, rusqlite::Error> {
        let exist = if let Some(source) = &post.source {
            self.check_post(source)?
        } else {
            None
        };

        let post = match exist {
            Some(id) => self.import_post_meta_by_sync(id, post)?,
            None => self.import_post_meta_by_create(post)?,
        };

        Ok(post)
    }
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
    pub fn import_post_meta_by_sync(
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

    pub fn import_post(
        &self,
        post: PartialSyncPost,
        files: &HashMap<String, FileMetaId>,
    ) -> Result<Post, rusqlite::Error> {
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

        Ok(Post {
            id: post.id,
            author: post.author,
            source: post.source,
            title: post.title,
            content,
            thumb,
            comments: post.comments,
            updated: post.updated,
            published: post.published,
        })
    }

    // Setters
    pub fn set_post_tags(&self, post: PostId, tags: &[PostTagId]) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT OR IGNORE INTO post_tags (post, tag) VALUES (?, ?)")?;
        for tag in tags {
            stmt.execute(params![post, tag])?;
        }
        Ok(())
    }
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
    pub fn set_post_title(&self, post: PostId, title: &str) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET title = ? WHERE id = ?")?;
        stmt.execute(params![title, post])?;
        Ok(())
    }
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
pub struct UnsyncPost {
    pub author: AuthorId,
    pub source: Option<String>,
    pub title: String,
    pub content: Vec<UnsyncContent>,
    pub thumb: Option<UnsyncFileMeta>,
    pub comments: Vec<Comment>,
    pub updated: DateTime<Utc>,
    pub published: DateTime<Utc>,
    pub tags: Vec<String>,
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
    pub fn tags(self, tags: Vec<String>) -> Self {
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
                let (file, method) =
                    manager.import_file_meta(post.author, post.id, raw.clone())?;

                // push to files
                files.push((manager.path.join(file.path()), method));
                Ok((raw.filename, file.id))
            })
            .collect::<Result<_, rusqlite::Error>>()?;

        let post = manager.import_post(post, &metas)?;
        manager.set_author_thumb_by_latest(post.author)?;

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

pub struct PartialSyncPost {
    pub id: PostId,
    pub author: AuthorId,
    pub source: Option<String>,
    pub title: String,
    pub content: Vec<UnsyncContent>,
    pub thumb: Option<UnsyncFileMeta>,
    pub comments: Vec<Comment>,
    pub updated: DateTime<Utc>,
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
