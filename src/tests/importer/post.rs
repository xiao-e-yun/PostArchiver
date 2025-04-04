use crate::{
    importer::{
        ImportFileMetaMethod, PartialSyncPost, UnsyncAuthor, UnsyncContent, UnsyncFileMeta,
        UnsyncPost,
    },
    manager::PostArchiverManager,
    AuthorId, Comment, Content, FileMetaId, PostId,
};
use chrono::{TimeZone, Utc};
use std::collections::HashMap;

#[test]
fn test_import_post() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create a author
    let author = UnsyncAuthor::new("octocat".to_string())
        .sync(&manager)
        .unwrap();

    // Create a post
    let import_post = UnsyncPost::new(author.id)
        .source(Some("https://example.com".to_string()))
        .title("Hello World".to_string())
        .content(vec![
            UnsyncContent::Text("Hello World".to_string()),
            UnsyncContent::File(UnsyncFileMeta {
                filename: "image.png".to_string(),
                mime: "image/png".to_string(),
                extra: HashMap::new(),
                method: ImportFileMetaMethod::Data(vec![1, 2, 3]),
            }),
        ]);

    // Import the post meta
    let post = manager.import_post_meta(import_post.clone()).unwrap();
    let metas = load_files(&manager, &post);
    let post = manager.import_post(post, &metas).unwrap();
    manager.set_author_thumb_by_latest(author.id).unwrap();
    manager.set_author_updated_by_latest(author.id).unwrap();

    // Check the imported post
    let saved_post = manager.get_post(&post).unwrap();
    assert_eq!(saved_post.title, "Hello World");

    // Update the post meta by source
    let post = manager.import_post_meta(import_post).unwrap();
    let metas = load_files(&manager, &post);
    let post = manager.import_post(post, &metas).unwrap();
    manager.set_author_thumb_by_latest(author.id).unwrap();
    manager.set_author_updated_by_latest(author.id).unwrap();

    // Check the imported post
    let saved_post = manager.get_post(&post).unwrap();
    assert_eq!(saved_post.title, "Hello World");

    fn load_files(
        manager: &PostArchiverManager,
        post: &PartialSyncPost,
    ) -> HashMap<String, FileMetaId> {
        let mut files = vec![];
        let metas: HashMap<String, FileMetaId> = post
            .collect_files()
            .into_iter()
            .map(|raw| {
                let (file, method) = manager.import_file_meta(post.author, post.id, raw.clone())?;

                files.push((manager.path.join(file.path()), method));
                Ok((raw.filename, file.id))
            })
            .collect::<Result<_, rusqlite::Error>>()
            .unwrap();

        for (path, method) in files {
            match method {
                ImportFileMetaMethod::Data(data) => {
                    assert_eq!(data, vec![1, 2, 3]);
                    std::fs::write(path, data).unwrap();
                }
                _ => unreachable!(),
            }
        }

        metas
    }
}

#[test]
fn test_import_post_by_part() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create a author
    let author = UnsyncAuthor::new("octocat".to_string())
        .sync(&manager)
        .unwrap();

    // Create a post
    let import_post = UnsyncPost::new(author.id)
        .source(Some("https://example.com".to_string()))
        .title("Hello World".to_string())
        .content(vec![
            UnsyncContent::Text("Hello World".to_string()),
            UnsyncContent::File(UnsyncFileMeta {
                filename: "image.png".to_string(),
                mime: "image/png".to_string(),
                extra: HashMap::new(),
                method: ImportFileMetaMethod::Data(vec![1, 2, 3]),
            }),
        ]);

    // Import the post meta
    let post = manager
        .import_post_meta_by_create(import_post.clone())
        .unwrap();
    let metas = load_files(&manager, &post);
    let post = manager.import_post(post, &metas).unwrap();
    manager.set_author_thumb_by_latest(author.id).unwrap();
    manager.set_author_updated_by_latest(author.id).unwrap();

    // Check the imported post
    let saved_post = manager.get_post(&post).unwrap();
    assert_eq!(saved_post.title, "Hello World");

    // Update the post meta by source
    let post = manager
        .import_post_meta_by_update(post, import_post)
        .unwrap();
    let metas = load_files(&manager, &post);
    let post = manager.import_post(post, &metas).unwrap();
    manager.set_author_thumb_by_latest(author.id).unwrap();
    manager.set_author_updated_by_latest(author.id).unwrap();

    // Check the imported post
    let saved_post = manager.get_post(&post).unwrap();
    assert_eq!(saved_post.title, "Hello World");

    fn load_files(
        manager: &PostArchiverManager,
        post: &PartialSyncPost,
    ) -> HashMap<String, FileMetaId> {
        let mut files = vec![];
        let metas: HashMap<String, FileMetaId> = post
            .collect_files()
            .into_iter()
            .map(|raw| {
                let (file, method) = manager.import_file_meta(post.author, post.id, raw.clone())?;

                files.push((manager.path.join(file.path()), method));
                Ok((raw.filename, file.id))
            })
            .collect::<Result<_, rusqlite::Error>>()
            .unwrap();

        for (path, method) in files {
            match method {
                ImportFileMetaMethod::Data(data) => {
                    assert_eq!(data, vec![1, 2, 3]);
                    std::fs::write(path, data).unwrap();
                }
                _ => unreachable!(),
            }
        }

        metas
    }
}

#[test]
fn test_sync_post() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create a author
    let author = UnsyncAuthor::new("octocat".to_string())
        .sync(&manager)
        .unwrap();

    let updated = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();

    // Create a post
    let (_, files) = UnsyncPost::new(author.id)
        .source(Some("https://example.com".to_string()))
        .title("Hello World".to_string())
        .published(updated)
        .updated(updated)
        .comments(vec![Comment {
            user: "octocat".to_string(),
            text: "Hello World".to_string(),
            replies: vec![],
        }])
        .content(vec![
            UnsyncContent::Text("Hello World".to_string()),
            UnsyncContent::File(UnsyncFileMeta {
                filename: "image.png".to_string(),
                mime: "image/png".to_string(),
                extra: HashMap::new(),
                method: ImportFileMetaMethod::Data(vec![1, 2, 3]),
            }),
        ])
        .sync(&manager)
        .unwrap();

    for (path, method) in files {
        match method {
            ImportFileMetaMethod::Data(data) => {
                assert_eq!(data, vec![1, 2, 3]);
                std::fs::write(path, data).unwrap();
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn test_post_setters() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let author = UnsyncAuthor::new("test_author".to_string())
        .sync(&manager)
        .unwrap();

    // Create a basic post
    let updated = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();

    let (post, _) = UnsyncPost::new(author.id)
        .source(Some("https://example.com".to_string()))
        .title("Hello World".to_string())
        .published(updated)
        .updated(updated)
        .comments(vec![Comment {
            user: "octocat".to_string(),
            text: "Hello World".to_string(),
            replies: vec![],
        }])
        .content(vec![
            UnsyncContent::Text("Hello World".to_string()),
            UnsyncContent::File(UnsyncFileMeta {
                filename: "image.png".to_string(),
                mime: "image/png".to_string(),
                extra: HashMap::new(),
                method: ImportFileMetaMethod::Data(vec![1, 2, 3]),
            }),
        ])
        .sync(&manager)
        .unwrap();

    // Test set_post_title
    manager.set_post_title(post.id, "Updated Title").unwrap();
    assert_eq!(manager.get_post(&post.id).unwrap().title, "Updated Title");

    // Test set_post_source
    let new_source = Some("https://example.com/updated".to_string());
    manager.set_post_source(post.id, &new_source).unwrap();
    assert_eq!(manager.get_post(&post.id).unwrap().source, new_source);

    // Test set_post_comments
    let comments = vec![Comment {
        user: "testuser".to_string(),
        text: "Test comment".to_string(),
        replies: vec![],
    }];
    manager.set_post_comments(post.id, &comments).unwrap();
    assert_eq!(
        manager.get_post(&post.id).unwrap().comments[0].user,
        comments[0].user
    );

    // Test set_post_thumb
    let thumb = manager.get_post(&post.id).unwrap().thumb;
    manager.set_post_thumb(post.id, &None).unwrap();
    assert_eq!(manager.get_post(&post.id).unwrap().thumb, None);

    manager.set_post_thumb(post.id, &thumb).unwrap();
    assert_eq!(manager.get_post(&post.id).unwrap().thumb, thumb);

    // Test set_post_content
    let new_content = vec![Content::Text("Updated content".to_string())];
    manager.set_post_content(post.id, &new_content).unwrap();

    // Test update timestamps
    let time1 = Utc.with_ymd_and_hms(2016, 1, 1, 0, 0, 0).unwrap();
    let time2 = Utc.with_ymd_and_hms(2017, 1, 1, 0, 0, 0).unwrap();

    // Test set_post_updated
    manager.set_post_updated(post.id, &time1).unwrap();
    assert_eq!(manager.get_post(&post.id).unwrap().updated, time1);

    // Test set_post_updated_by_latest
    manager.set_post_updated_by_latest(post.id, &time2).unwrap();
    assert_eq!(manager.get_post(&post.id).unwrap().updated, time2);

    // Try to update with older time - should not update
    manager.set_post_updated_by_latest(post.id, &time1).unwrap();
    assert_eq!(manager.get_post(&post.id).unwrap().updated, time2);

    // Test set_post_published
    manager.set_post_published(post.id, &time1).unwrap();
    assert_eq!(manager.get_post(&post.id).unwrap().published, time1);

    // Test set_post_published_by_latest
    manager
        .set_post_published_by_latest(post.id, &time2)
        .unwrap();
    assert_eq!(manager.get_post(&post.id).unwrap().published, time2);

    // Try to update with older time - should not update
    manager
        .set_post_published_by_latest(post.id, &time1)
        .unwrap();
    assert_eq!(manager.get_post(&post.id).unwrap().published, time2);
}

#[test]
fn test_unsync_post_methods() {
    let author_id = AuthorId(1);
    let now = Utc::now();

    // Test new
    let post = UnsyncPost::new(author_id);
    assert_eq!(post.author, author_id);
    assert!(post.source.is_none());
    assert!(post.title.is_empty());

    // Test all setters
    let post = post
        .author(AuthorId(2))
        .source(Some("https://example.com".to_string()))
        .title("Test Title".to_string())
        .content(vec![UnsyncContent::Text("Test content".to_string())])
        .thumb(Some(UnsyncFileMeta {
            filename: "thumb.jpg".to_string(),
            mime: "image/jpeg".to_string(),
            extra: HashMap::new(),
            method: ImportFileMetaMethod::None,
        }))
        .comments(vec![Comment {
            user: "user".to_string(),
            text: "comment".to_string(),
            replies: vec![],
        }])
        .updated(now.clone())
        .published(now.clone())
        .tags(vec!["test".to_string()]);

    assert_eq!(post.author, AuthorId(2));
    assert_eq!(post.source, Some("https://example.com".to_string()));
    assert_eq!(post.title, "Test Title");
    assert_eq!(post.content.len(), 1);
    assert!(post.thumb.is_some());
    assert_eq!(post.comments.len(), 1);
    assert_eq!(post.updated, now);
    assert_eq!(post.published, now);
    assert_eq!(post.tags, vec!["test".to_string()]);
}

#[test]
fn test_unsync_content_methods() {
    // Test text constructor
    let text_content = UnsyncContent::text("Test text".to_string());
    match text_content {
        UnsyncContent::Text(text) => assert_eq!(text, "Test text"),
        _ => panic!("Expected Text variant"),
    }

    // Test file constructor
    let file_meta = UnsyncFileMeta {
        filename: "test.jpg".to_string(),
        mime: "image/jpeg".to_string(),
        extra: HashMap::new(),
        method: ImportFileMetaMethod::None,
    };
    let file_content = UnsyncContent::file(file_meta.clone());
    match file_content {
        UnsyncContent::File(meta) => assert_eq!(meta.filename, "test.jpg"),
        _ => panic!("Expected File variant"),
    }
}

#[test]
fn test_partial_sync_post_methods() {
    let author_id = AuthorId(1);
    let post_id = PostId(1);
    let now = Utc::now();

    // Create base UnsyncPost
    let unsync_post = UnsyncPost::new(author_id)
        .title("Test Title".to_string())
        .updated(now.clone())
        .published(now.clone());

    // Test new
    let partial = PartialSyncPost::new(author_id, post_id, unsync_post);
    assert_eq!(partial.id, post_id);
    assert_eq!(partial.author, author_id);
    assert_eq!(partial.title, "Test Title");

    // Test content setter
    let new_content = vec![UnsyncContent::Text("New content".to_string())];
    let partial = partial.content(new_content.clone());

    // Check content using pattern matching instead of direct comparison
    match (&partial.content[0], &new_content[0]) {
        (UnsyncContent::Text(a), UnsyncContent::Text(b)) => assert_eq!(a, b),
        _ => panic!("Expected Text content"),
    }

    // Test thumb setter
    let thumb = UnsyncFileMeta {
        filename: "thumb.jpg".to_string(),
        mime: "image/jpeg".to_string(),
        extra: HashMap::new(),
        method: ImportFileMetaMethod::None,
    };
    let partial = partial.thumb(Some(thumb.clone()));
    assert!(partial.thumb.is_some());

    // Test collect_files with both content and thumb files
    let partial = PartialSyncPost::new(author_id, post_id, UnsyncPost::new(author_id))
        .content(vec![
            UnsyncContent::Text("text".to_string()),
            UnsyncContent::File(UnsyncFileMeta {
                filename: "content.jpg".to_string(),
                mime: "image/jpeg".to_string(),
                extra: HashMap::new(),
                method: ImportFileMetaMethod::None,
            }),
        ])
        .thumb(Some(UnsyncFileMeta {
            filename: "thumb.jpg".to_string(),
            mime: "image/jpeg".to_string(),
            extra: HashMap::new(),
            method: ImportFileMetaMethod::None,
        }));

    let files = partial.collect_files();
    assert_eq!(files.len(), 2); // Should have both content and thumb files
}
