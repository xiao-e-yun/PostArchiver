use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    path::PathBuf,
};

use chrono::{DateTime, Utc};
use rusqlite::params;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    AuthorId, CollectionId, Comment, Content, FileMetaId, PlatformId, Post, PostId,
    POSTS_PRE_CHUNK,
};

use super::{collection::UnsyncCollection, tag::UnsyncTag, UnsyncFileMeta};

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
    ///     
    ///     let author_id = AuthorId(1);
    ///     let post = UnsyncPost::new()
    ///         .authors(vec![author_id])
    ///         .title("My First Post".to_string())
    ///         .source(Some("https://blog.example.com/post/1".to_string()));
    ///         
    ///     let partial_post = manager.import_post_meta(post)?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn import_post<U>(
        &self,
        post: UnsyncPost,
        files_data: HashMap<String, U>,
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

        let thumb = post
            .thumb
            .clone()
            .or_else(|| {
                post.content.iter().find_map(|c| {
                    let UnsyncContent::File(file) = c else {
                        return None;
                    };
                    file.mime.starts_with("image/").then_some(file.clone())
                })
            })
            .map(|thumb| self.import_file_meta(id, thumb))
            .transpose()?;
        self.set_post_thumb(id, thumb)?;

        let content = post
            .content
            .into_iter()
            .map(|content| {
                Ok(match content {
                    UnsyncContent::Text(text) => Content::Text(text),
                    UnsyncContent::File(file) => Content::File(self.import_file_meta(id, file)?),
                })
            })
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        self.set_post_content(id, content)?;

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

        let files = files_data
            .into_iter()
            .map(|(filename, data)| (path.join(&filename), data))
            .collect::<Vec<_>>();

        Ok((
            id,
            post.authors.into_iter().collect(),
            collections.into_iter().collect(),
            files,
        ))
    }

    pub fn import_posts<U>(
        &self,
        posts: impl IntoIterator<Item = UnsyncPost>,
    ) -> Result<(Vec<PostId>, Vec<(PathBuf, U)>), rusqlite::Error> {
        let mut total_author = HashSet::new();
        let mut total_collections = HashSet::new();
        let mut total_files = Vec::new();
        let mut results = Vec::new();

        for post in posts {
            let (id, authors, collections, files_data) = self.import_post(post, HashMap::new())?;

            results.push(id);
            total_files.extend(files_data);
            total_author.extend(authors);
            total_collections.extend(collections);
        }

        total_author.into_iter().try_for_each(|author| {
            self.set_author_thumb_by_latest(author)?;
            self.set_author_updated_by_latest(author)
        })?;

        total_collections
            .into_iter()
            .try_for_each(|collection| self.set_collection_thumb_by_latest(collection))?;

        Ok((results, total_files))
    }
}

#[derive(Debug, Clone)]
/// Represents a post that is not yet synchronized with the archive.
pub struct UnsyncPost {
    /// The original URL of this post (e.g., "https://example.com/blog/1")
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
    pub platform: PlatformId,
    /// Tags associated with the post
    pub tags: Vec<UnsyncTag>,
    /// The IDs of the author who created this post
    pub authors: Vec<AuthorId>,
    /// The collections this post belongs to
    pub collections: Vec<UnsyncCollection>,
}

impl UnsyncPost {
    pub fn new(
        platform: PlatformId,
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

    pub fn sync<T, U>(
        self,
        manager: &PostArchiverManager<T>,
    ) -> Result<(PostId, Vec<(PathBuf, U)>), rusqlite::Error>
    where
        T: PostArchiverConnection,
    {
        let (id, authors, collections, files_data) = manager.import_post(self, HashMap::new())?;

        authors.into_iter().try_for_each(|author| {
            manager.set_author_thumb_by_latest(author)?;
            manager.set_author_updated_by_latest(author)
        })?;

        collections
            .into_iter()
            .try_for_each(|collection| manager.set_collection_thumb_by_latest(collection))?;

        Ok((id, files_data))
    }
}

#[derive(Debug, Clone)]
pub enum UnsyncContent {
    Text(String),
    File(UnsyncFileMeta),
}
