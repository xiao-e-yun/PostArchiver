use std::{collections::HashMap, fs, path::PathBuf};

use chrono::{DateTime, Local};
use log::{info, warn};
use post_archiver_latest::{AuthorId, Comment, Content, FileMetaId, Link, PostId, PostTagId};
use post_archiver_v0_1::{ArchiveContent, ArchiveFile, ArchiveFrom};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

use crate::{Bridge, config::Config};

#[derive(Debug, Clone, Default)]
pub struct BridgeV1 {
    fanbox_tag: Option<PostTagId>,
    fanbox_dl_tag: Option<PostTagId>,
}

impl Bridge for BridgeV1 {
    const VERSION: &'static str = "v0.1";

    fn verify(&mut self, config: &Config) -> bool {
        config.source.join("authors.json").exists()
    }

    fn upgrade(&mut self, config: &mut Config) {
        warn!("Only supports fanbox-archive");

        let db_path = config.target.join("post-archiver.db");
        let mut conn = if db_path.exists() {
            info!("Connecting to database: {}", db_path.display());
            Connection::open(&db_path).expect("Failed to open database")
        } else {
            info!("Creating database: {}", db_path.display());
            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent).expect("Failed to create database directory");
            }

            let conn = Connection::open(&db_path).expect("Failed to create database");
            conn.execute_batch(post_archiver_latest::utils::TEMPLATE_DATABASE_UP_SQL)
                .expect("Failed to create tables");
            conn
        };

        let authors = config.source.join("authors.json");
        let authors = fs::read(authors).expect("Unable to read authors.json");
        let authors: post_archiver_v0_1::ArchiveAuthorsList =
            serde_json::from_slice(&authors).expect("Unable to parse authors.json");

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime");
        for author in authors.0 {
            let tx = conn.transaction().expect("Failed to start transaction");
            let _guard = rt.enter();

            let name = author.name.clone();
            let links = vec![Link::new(
                "fanbox",
                &format!("https://{}.fanbox.cc/", author.id),
            )];

            let source = format!("fanbox:{}", author.id);
            let author_id: AuthorId = match
                tx
                    .query_row(
                        "SELECT * FROM authors JOIN author_alias ON authors.id = author_alias.target WHERE author_alias.source = ?;",
                        [&source],
                        |row| { row.get(0) }
                    )
                    .optional()
                    .expect("Failed to get author id")
            {
                Some(id) => { id }
                None => {
                    info!("Inserting author: {}", name);
                    let id: AuthorId = tx
                        .query_row(
                            "INSERT INTO authors (name, links) VALUES (?, ?) RETURNING id",
                            params![
                                &name,
                                &serde_json::to_string(&links).expect("Failed to serialize links")
                            ],
                            |row| row.get(0)
                        )
                        .expect("Failed to insert author");

                    tx.execute(
                        "INSERT OR IGNORE INTO author_alias (source, target) VALUES (?, ?)",
                        params![source, &id]
                    ).expect("Failed to insert author alias");

                    id
                }
            };

            let posts = config.source.join(author.id.clone());
            let posts: Vec<PathBuf> = fs::read_dir(posts)
                .expect("Unable to read posts directory")
                .filter_map(|entry| {
                    let entry = entry.expect("Failed to read entry");
                    if entry.file_type().expect("Failed to get file type").is_dir() {
                        Some(entry.path())
                    } else {
                        None
                    }
                })
                .collect();

            for post_path in posts {
                let post = fs::read(post_path.join("post.json")).expect("Unable to read post");
                let post: ArchivePost =
                    serde_json::from_slice(&post).expect("Unable to parse post");

                let is_fanbox_dl = post.content
                    .first()
                    .map_or(
                        false,
                        |content|
                            matches!(content, post_archiver_v0_1::ArchiveContent::Text(text) if text == "Archive from `fanbox-dl`")
                    );

                let source = if is_fanbox_dl {
                    None
                } else {
                    Some(format!("https://{}.fanbox.cc/posts/{}", author.id, post.id))
                };

                // Check if post already exists
                let count: u32 = if is_fanbox_dl {
                    tx.query_row(
                        "SELECT count() FROM posts WHERE title = ? AND author = ?",
                        params![post.title, author_id],
                        |row| row.get(0),
                    )
                    .expect("Failed to get post count")
                } else {
                    tx.query_row(
                        "SELECT count() FROM posts WHERE source = ?",
                        [source.clone().unwrap()],
                        |row| row.get(0),
                    )
                    .expect("Failed to get post count")
                };
                if 1 == count {
                    continue;
                }

                let (tag, tag_name) = if is_fanbox_dl {
                    (&mut self.fanbox_dl_tag, "fanbox-dl")
                } else {
                    (&mut self.fanbox_tag, "fanbox")
                };

                let comments = post
                    .comments
                    .clone()
                    .into_iter()
                    .map(Comment::from)
                    .collect::<Vec<_>>();

                let updated = post.updated.with_timezone(&chrono::Utc).naive_utc();
                let published = post.published.with_timezone(&chrono::Utc).naive_utc();

                let post_id: PostId = tx
                    .query_row(
                        "INSERT INTO posts (author, source, title, content, comments, published, updated) VALUES (?, ?, ?, '[\"UNSYNCED\"]', ?, ?, ?) RETURNING id",
                        params![
                            author_id,
                            source,
                            post.title,
                            serde_json::to_string(&comments).expect("Failed to serialize comments"),
                            published,
                            updated
                        ],
                        |row| row.get(0)
                    )
                    .expect("Failed to insert post");

                let tag = match tag {
                    Some(tag) => *tag,
                    None => {
                        *tag = tx
                            .query_row(
                                "SELECT id FROM tags WHERE name = ?",
                                params![tag_name],
                                |row| row.get(0),
                            )
                            .optional()
                            .expect("Failed to get tag id");
                        if tag.is_none() {
                            info!("Inserting tag: {}", tag_name);
                            *tag = tx
                                .query_row(
                                    "INSERT INTO tags (name) VALUES (?) RETURNING id",
                                    params![tag_name],
                                    |row| row.get(0),
                                )
                                .expect("Failed to insert tag");
                        }

                        tag.unwrap()
                    }
                };
                tx.execute("INSERT INTO post_tags (post, tag) VALUES (?, ?)", params![
                    post_id, tag
                ])
                .expect("Failed to insert post tag");

                if !post.files.is_empty() {
                    let target = config
                        .target
                        .join(author_id.to_string())
                        .join(post_id.to_string());
                    fs::create_dir_all(target).expect("Failed to create post directory");
                }

                let mut tasks = JoinSet::new();
                let mut file_map: HashMap<String, FileMetaId> = HashMap::new();
                let mut insert_file_stmt = tx
                    .prepare_cached(
                        "INSERT INTO file_metas (post, author, filename, mime, extra) VALUES (?, ?, ?, ?, ?) RETURNING id"
                    )
                    .expect("Failed to prepare statement");
                for file in post.files {
                    let file_path = config.source.join(file.path());
                    let filename = file.filename().to_string_lossy().to_string();
                    let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
                    let extra = match file {
                        ArchiveFile::Image { width, height, .. } => {
                            format!("{{\"height\":{},\"width\":{}}}", height, width)
                        }
                        _ => "{}".to_string(),
                    };

                    let file_id = insert_file_stmt
                        .query_row(
                            params![post_id, author_id, filename, mime.to_string(), extra],
                            |row| row.get(0),
                        )
                        .expect("Failed to insert file meta");

                    file_map.insert(file.path().to_string_lossy().to_string(), file_id);

                    let target_path = config
                        .target
                        .join(author_id.to_string())
                        .join(post_id.to_string())
                        .join(filename);

                    tasks.spawn(async move {
                        tokio::fs::copy(file_path, target_path)
                            .await
                            .expect("Failed to copy file");
                    });
                }

                let content = post
                    .content
                    .iter()
                    .map(|content| match content {
                        ArchiveContent::Text(text) => Content::Text(text.clone()),
                        ArchiveContent::Image(path) => {
                            Content::File(file_map.get(path).unwrap().clone())
                        }
                        ArchiveContent::Video(path) => {
                            Content::File(file_map.get(path).unwrap().clone())
                        }
                        ArchiveContent::File(path) => {
                            Content::File(file_map.get(path).unwrap().clone())
                        }
                    })
                    .collect::<Vec<_>>();

				tx.execute(
					"UPDATE posts SET content = ? WHERE id = ?",
					params![
						serde_json::to_string(&content).expect("Failed to serialize content"),
						post_id
					]
				).expect("Failed to update post content");

                rt.block_on(async {
                    while let Some(task) = tasks.join_next().await {
                        task.unwrap();
                    }
                });
            }

            tx.commit().expect("Failed to commit transaction");
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePost {
    pub id: String,
    pub title: String,
    pub author: String,
    pub from: ArchiveFrom,
    pub thumb: Option<PathBuf>,
    pub files: Vec<ArchiveFile>,
    pub updated: DateTime<Local>,
    pub published: DateTime<Local>,
    pub content: Vec<ArchiveContent>,
    pub comments: Vec<ArchiveComment>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveComment {
    pub user: String,
    pub text: String,
    #[serde(skip_serializing_if = "<[_]>::is_empty", default)]
    // because replies has no default value in v0.1.14
    pub replies: Vec<ArchiveComment>,
}

impl From<ArchiveComment> for Comment {
    fn from(comment: ArchiveComment) -> Self {
        Comment {
            user: comment.user,
            text: comment.text,
            replies: comment.replies.into_iter().map(Comment::from).collect(),
        }
    }
}
