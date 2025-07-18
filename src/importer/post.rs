use std::{collections::HashSet, fmt::Debug, path::PathBuf};

use chrono::{DateTime, Utc};

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    AuthorId, CollectionId, Comment, Content, PlatformId, PostId, POSTS_PRE_CHUNK,
};

use super::{collection::UnsyncCollection, tag::UnsyncTag, UnsyncFileMeta};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Import a post into the archive.
    ///
    /// If the post already exists (by source), it updates its title, platform, published date,
    ///
    /// # Parameters
    ///
    /// - `update_relation`: update the relations of authors and collections after importing.
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
    /// # use post_archiver::PlatformId;
    /// # use std::collections::HashMap;
    /// # fn example(manager: &PostArchiverManager) -> Result<(), rusqlite::Error> {
    /// let post: UnsyncPost<()> = UnsyncPost::new(PlatformId(1), "https://blog.example.com/post/1".to_string(), "My First Post".to_string(), vec![]);
    ///
    /// let files_data = HashMap::<String,()>::new(); // You can provide file data if needed
    ///    
    /// let post = manager.import_post(post, true)?;
    ///
    /// Ok(())
    /// # }
    /// ```
    pub fn import_post<U>(
        &self,
        post: UnsyncPost<U>,
        update_relation: bool,
    ) -> Result<(PostId, Vec<AuthorId>, Vec<CollectionId>, Vec<(PathBuf, U)>), rusqlite::Error>
    {
        macro_rules! import_many {
            ($vec:expr => $method:ident) => {
                $vec.into_iter()
                    .map(|d| self.$method(d))
                    .collect::<Result<Vec<_>, _>>()?
            };
        }

        let id = match self.find_post(&post.source)? {
            Some(id) => {
                self.set_post_title(id, post.title)?;
                self.set_post_platform(id, Some(post.platform))?;

                self.set_post_published(id, post.published.unwrap_or_else(Utc::now))?;
                self.set_post_updated_by_latest(id, post.updated.unwrap_or_else(Utc::now))?;
                id
            }
            None => self.add_post(
                post.title,
                Some(post.source),
                Some(post.platform),
                post.published,
                post.updated,
            )?,
        };

        let mut thumb = post
            .thumb
            .as_ref()
            .map(|thumb| self.import_file_meta(id, thumb))
            .transpose()?;

        let content = post
            .content
            .iter()
            .map(|content| {
                Ok(match content {
                    UnsyncContent::Text(text) => Content::Text(text.clone()),
                    UnsyncContent::File(file) => {
                        let need_thumb = thumb.is_none() && file.mime.starts_with("image/");
                        let file_meta = self.import_file_meta(id, file)?;
                        need_thumb.then(|| thumb = Some(file_meta));
                        Content::File(file_meta)
                    }
                })
            })
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        self.set_post_content(id, content)?;
        self.set_post_thumb(id, thumb)?;

        self.set_post_comments(id, post.comments)?;

        let tags = import_many!(post.tags => import_tag);
        self.add_post_tags(id, &tags)?;

        let collections = import_many!(post.collections => import_collection);
        self.add_post_collections(id, &collections)?;

        self.add_post_authors(id, &post.authors)?;

        //
        let path = self
            .path
            .join((id.raw() / POSTS_PRE_CHUNK).to_string())
            .join((id.raw() % POSTS_PRE_CHUNK).to_string());

        let files = post
            .content
            .into_iter()
            .flat_map(|c| match c {
                UnsyncContent::Text(_) => None,
                UnsyncContent::File(file) => Some(file),
            })
            .chain(post.thumb)
            .map(|f| (path.join(f.filename), f.data))
            .collect::<Vec<_>>();

        if update_relation {
            post.authors.iter().try_for_each(|&author| {
                self.set_author_thumb_by_latest(author)?;
                self.set_author_updated_by_latest(author)
            })?;

            collections
                .iter()
                .try_for_each(|&collection| self.set_collection_thumb_by_latest(collection))?;
        }

        Ok((
            id,
            post.authors.into_iter().collect(),
            collections.into_iter().collect(),
            files,
        ))
    }

    /// Import multiple posts into the archive.
    ///
    /// This function processes each post, importing its authors, collections, and files.
    ///
    /// # Parameters
    ///
    /// - `update_relation`: update the relations of authors and collections after importing.
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
    /// # use post_archiver::PlatformId;
    /// # use std::collections::HashMap;
    /// # fn example(manager: &PostArchiverManager) -> Result<(), rusqlite::Error> {
    /// let posts: Vec<UnsyncPost<()>> = vec![
    ///     UnsyncPost::new(PlatformId(1), "https://blog.example.com/post/1".to_string(), "My First Post".to_string(), vec![]),
    ///     UnsyncPost::new(PlatformId(1), "https://blog.example.com/post/2".to_string(), "My Second Post".to_string(), vec![]),
    /// ];
    ///
    /// let post = manager.import_posts(posts, true)?;
    ///
    /// Ok(())
    /// # }
    /// ```
    pub fn import_posts<U>(
        &self,
        posts: impl IntoIterator<Item = UnsyncPost<U>>,
        update_relation: bool,
    ) -> Result<(Vec<PostId>, Vec<(PathBuf, U)>), rusqlite::Error> {
        let mut total_author = HashSet::new();
        let mut total_collections = HashSet::new();
        let mut total_files = Vec::new();
        let mut results = Vec::new();

        for post in posts {
            let (id, authors, collections, files_data) = self.import_post(post, false)?;

            results.push(id);
            total_files.extend(files_data);
            total_author.extend(authors);
            total_collections.extend(collections);
        }

        if update_relation {
            total_author.into_iter().try_for_each(|author| {
                self.set_author_thumb_by_latest(author)?;
                self.set_author_updated_by_latest(author)
            })?;

            total_collections
                .into_iter()
                .try_for_each(|collection| self.set_collection_thumb_by_latest(collection))?;
        }

        Ok((results, total_files))
    }
}

#[derive(Debug, Clone)]
/// Represents a post that is not yet synchronized with the archive.
pub struct UnsyncPost<T> {
    /// The original URL of this post (e.g., "https://example.com/blog/1")
    pub source: String,
    /// The title of the post
    pub title: String,
    /// The post's content items (text and file references)
    pub content: Vec<UnsyncContent<T>>,
    /// Optional thumbnail image for the post
    pub thumb: Option<UnsyncFileMeta<T>>,
    /// Comments on the post
    pub comments: Vec<Comment>,
    /// When the post was updated
    pub updated: Option<DateTime<Utc>>,
    /// When the post was published
    pub published: Option<DateTime<Utc>>,
    /// Platform associated with the post
    pub platform: PlatformId,
    /// Tags associated with the post
    pub tags: Vec<UnsyncTag>,
    /// The IDs of the author who created this post
    pub authors: Vec<AuthorId>,
    /// The collections this post belongs to
    pub collections: Vec<UnsyncCollection>,
}

impl<T> UnsyncPost<T> {
    pub fn new(
        platform: PlatformId,
        source: String,
        title: String,
        content: Vec<UnsyncContent<T>>,
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

    pub fn content(self, content: Vec<UnsyncContent<T>>) -> Self {
        Self { content, ..self }
    }

    pub fn thumb(self, thumb: Option<UnsyncFileMeta<T>>) -> Self {
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

    pub fn platform(self, platform: PlatformId) -> Self {
        Self { platform, ..self }
    }

    pub fn tags(self, tags: Vec<UnsyncTag>) -> Self {
        Self { tags, ..self }
    }

    pub fn authors(self, authors: Vec<AuthorId>) -> Self {
        Self { authors, ..self }
    }

    pub fn collections(self, collections: Vec<UnsyncCollection>) -> Self {
        Self {
            collections,
            ..self
        }
    }

    /// import this post into the archive, synchronizing it with the database.
    ///
    /// This is abbreviation for [import_post](crate::PostArchiverManager::import_post)
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn sync<U>(
        self,
        manager: &PostArchiverManager<U>,
    ) -> Result<(PostId, Vec<(PathBuf, T)>), rusqlite::Error>
    where
        U: PostArchiverConnection,
    {
        let (id, _, _, files_data) = manager.import_post(self, true)?;

        Ok((id, files_data))
    }
}

#[derive(Debug, Clone)]
pub enum UnsyncContent<T> {
    Text(String),
    File(UnsyncFileMeta<T>),
}
