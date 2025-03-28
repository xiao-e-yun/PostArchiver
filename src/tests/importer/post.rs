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
fn test_check_post() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create a author
    let (author, _) = UnsyncAuthor::new("octocat".to_string())
        .sync(&manager)
        .unwrap();

    // Create a post
    let (post, _) = UnsyncPost::new(author.id)
        .source(Some("https://example.com".to_string()))
        .sync(&manager)
        .unwrap();

    // Check if the post exists
    let post_id = manager.check_post(&post.source.unwrap()).unwrap();
    assert_eq!(post_id, Some(post.id));
}

#[test]
fn test_check_post_with_updated() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create a author
    let (author, _) = UnsyncAuthor::new("octocat".to_string())
        .sync(&manager)
        .unwrap();

    // Create a post
    let (post, _) = UnsyncPost::new(author.id)
        .source(Some("https://example.com".to_string()))
        .sync(&manager)
        .unwrap();

    // Check if the post exists
    let post_id = manager
        .check_post_with_updated(&post.source.unwrap(), &post.updated)
        .unwrap();
    assert_eq!(post_id, Some(post.id));
}

#[test]
fn test_import_post_meta() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create a author
    let (author, _) = UnsyncAuthor::new("octocat".to_string())
        .sync(&manager)
        .unwrap();

    // Create a post
    let post = UnsyncPost::new(author.id)
        .source(Some("https://example.com".to_string()))
        .title("Hello World".to_string())
        .content(vec![UnsyncContent::Text("Hello World".to_string())]);

    // Import the post meta
    let post = manager.import_post_meta(post).unwrap();
    let metas = HashMap::new();

    // Complete the post
    let post = manager.import_post(post, &metas).unwrap();

    // Update the author
    manager.set_author_updated_by_latest(post.author).unwrap();
    manager.set_author_thumb_by_latest(post.author).unwrap();
}

#[test]
fn test_sync_post() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let (author, _) = UnsyncAuthor::new("octocat".to_string())
        .alias(vec!["github:octocat".to_string()])
        .sync(&manager)
        .unwrap();

    // Create a post with an image file
    let post = UnsyncPost::new(author.id)
        .title("Test Post".to_string())
        .content(vec![
            UnsyncContent::Text("Hello World".to_string()),
            UnsyncContent::File(UnsyncFileMeta {
                filename: "image.png".to_string(),
                mime: "image/png".to_string(),
                extra: HashMap::new(),
                method: ImportFileMetaMethod::Data(vec![1, 2, 3]),
            }),
        ]);

    // Sync the post
    let (post, files) = post.sync(&manager).unwrap();

    assert_eq!(post.title, "Test Post");
    assert_eq!(files.len(), 1); // Should have one file to save
    assert!(post.thumb.is_some()); // Should auto-select first image as thumb
}

#[test]
fn test_post_setters() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let (author, _) = UnsyncAuthor::new("test_author".to_string())
        .sync(&manager)
        .unwrap();

    // Create a basic post
    let (post, _) = UnsyncPost::new(author.id)
        .title("Original Title".to_string())
        .sync(&manager)
        .unwrap();

    // Test set_post_title
    manager.set_post_title(post.id, "Updated Title").unwrap();

    // Test set_post_source
    let new_source = Some("https://example.com/updated".to_string());
    manager.set_post_source(post.id, &new_source).unwrap();

    // Test set_post_comments
    let comments = vec![Comment {
        user: "testuser".to_string(),
        text: "Test comment".to_string(),
        replies: vec![],
    }];
    manager.set_post_comments(post.id, &comments).unwrap();

    // Test set_post_thumb
    let thumb_id = FileMetaId(1);
    manager.set_post_thumb(post.id, &Some(thumb_id)).unwrap();
    manager.set_post_thumb(post.id, &None).unwrap(); // Test removing thumb

    // Test set_post_content
    let new_content = vec![Content::Text("Updated content".to_string())];
    manager.set_post_content(post.id, &new_content).unwrap();
}

#[test]
fn test_post_timestamps() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author and post
    let (author, _) = UnsyncAuthor::new("test_author".to_string())
        .sync(&manager)
        .unwrap();
    let (post, _) = UnsyncPost::new(author.id)
        .title("Test Post".to_string())
        .sync(&manager)
        .unwrap();

    // Test update timestamps
    let time1 = Utc.timestamp_opt(1000000000, 0).unwrap();
    let time2 = Utc.timestamp_opt(1000000001, 0).unwrap();

    // Test set_post_updated
    manager.set_post_updated(post.id, &time1).unwrap();

    // Test set_post_updated_by_latest
    manager.set_post_updated_by_latest(post.id, &time2).unwrap();
    // Try to update with older time - should not update
    manager.set_post_updated_by_latest(post.id, &time1).unwrap();

    // Test set_post_published
    manager.set_post_published(post.id, &time1).unwrap();

    // Test set_post_published_by_latest
    manager
        .set_post_published_by_latest(post.id, &time2)
        .unwrap();
    // Try to update with older time - should not update
    manager
        .set_post_published_by_latest(post.id, &time1)
        .unwrap();
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
