use crate::{
    importer::{UnsyncAlias, UnsyncAuthor, UnsyncContent, UnsyncFileMeta, UnsyncPost},
    manager::platform::PlatformIdOrRaw,
    manager::PostArchiverManager,
    utils::tag::TagIdOrRaw,
    Comment,
};
use chrono::{TimeZone, Utc};
use std::collections::HashMap;

#[test]
fn test_import_post() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let author = UnsyncAuthor::new("octocat".to_string())
        .sync(&manager)
        .unwrap();

    // Create a post with object-oriented API
    let (post, _) = UnsyncPost::new()
        .authors(vec![author.id])
        .source(Some("https://example.com".to_string()))
        .title("Hello World".to_string())
        .content(vec![
            UnsyncContent::Text("Hello World".to_string()),
            UnsyncContent::File(UnsyncFileMeta {
                filename: "image.png".to_string(),
                mime: "image/png".to_string(),
                extra: HashMap::new(),
            }),
        ])
        .sync(&manager, {
            let mut files = HashMap::new();
            files.insert("image.png".to_string(), vec![1, 2, 3]);
            files
        })
        .unwrap();

    // Check the imported post
    assert_eq!(post.title, "Hello World");
}

#[test]
fn test_functional_import_post_meta() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create a post
    let import_post = UnsyncPost::new()
        .source(Some("https://example.com".to_string()))
        .title("Hello World".to_string())
        .content(vec![
            UnsyncContent::Text("Hello World".to_string()),
            UnsyncContent::File(UnsyncFileMeta {
                filename: "image.png".to_string(),
                mime: "image/png".to_string(),
                extra: HashMap::new(),
            }),
        ]);

    // Import the post meta using functional API
    let partial_post = manager.import_post_meta(import_post).unwrap();
    assert_eq!(partial_post.title, "Hello World");
}

#[test]
fn test_functional_import_post_meta_by_create() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create a post
    let import_post = UnsyncPost::new()
        .source(Some("https://example.com".to_string()))
        .title("Hello World".to_string());

    // Import the post meta by create
    let partial_post = manager.import_post_meta_by_create(import_post).unwrap();
    assert_eq!(partial_post.title, "Hello World");
}

#[test]
fn test_functional_import_post_meta_by_update() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create a post
    let import_post = UnsyncPost::new()
        .source(Some("https://example.com".to_string()))
        .title("Hello World".to_string());

    // Import the post meta by create first
    let partial_post = manager
        .import_post_meta_by_create(import_post.clone())
        .unwrap();

    // Update the post meta by update
    let updated_post = import_post.title("Updated Title".to_string());
    let updated_partial = manager
        .import_post_meta_by_update(partial_post.id, updated_post)
        .unwrap();

    assert_eq!(updated_partial.title, "Updated Title");
}

#[test]
fn test_functional_import_post() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create a partial post
    let partial_post = manager
        .import_post_meta_by_create(UnsyncPost::new().title("Test".to_string()))
        .unwrap();

    // Create files map
    let files = HashMap::new();

    // Import the complete post using functional API
    let post_id = manager.import_post(partial_post, &files).unwrap();

    let saved_post = manager.get_post(&post_id).unwrap();
    assert_eq!(saved_post.title, "Test");
}

#[test]
fn test_functional_check_post() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Test check_post when post doesn't exist
    let result = manager.check_post("https://nonexistent.com").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_functional_check_post_with_updated() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let updated = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();

    // Test check_post_with_updated when post doesn't exist
    let result = manager
        .check_post_with_updated("https://nonexistent.com", &updated)
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn test_functional_get_post() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create a post
    let partial_post = manager
        .import_post_meta_by_create(UnsyncPost::new().title("Test Post".to_string()))
        .unwrap();
    let post_id = manager.import_post(partial_post, &HashMap::new()).unwrap();

    // Test get_post functional API
    let retrieved_post = manager.get_post(&post_id).unwrap();
    assert_eq!(retrieved_post.title, "Test Post");
}

#[test]
fn test_unsync_post_methods() {
    let now = Utc::now();

    // Test new
    let post = UnsyncPost::new();
    assert!(post.authors.is_empty());
    assert!(post.source.is_none());
    assert!(post.title.is_empty());

    // Test all setters
    let post = post
        .authors(vec![crate::AuthorId(2)])
        .source(Some("https://example.com".to_string()))
        .title("Test Title".to_string())
        .content(vec![UnsyncContent::Text("Test content".to_string())])
        .thumb(Some(UnsyncFileMeta {
            filename: "thumb.jpg".to_string(),
            mime: "image/jpeg".to_string(),
            extra: HashMap::new(),
        }))
        .comments(vec![Comment {
            user: "user".to_string(),
            text: "comment".to_string(),
            replies: vec![],
        }])
        .updated(now.clone())
        .published(now.clone())
        .tags(vec![TagIdOrRaw::Raw("test".to_string())])
        .platform(PlatformIdOrRaw::Raw("test_platform".to_string()));

    assert_eq!(post.authors, vec![crate::AuthorId(2)]);
    assert_eq!(post.source, Some("https://example.com".to_string()));
    assert_eq!(post.title, "Test Title");
    assert_eq!(post.content.len(), 1);
    assert!(post.thumb.is_some());
    assert_eq!(post.comments.len(), 1);
    assert_eq!(post.updated, now);
    assert_eq!(post.published, now);
    assert_eq!(post.tags.len(), 1);
    assert!(post.platform.is_some());
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
    };
    let file_content = UnsyncContent::file(file_meta.clone());
    match file_content {
        UnsyncContent::File(meta) => assert_eq!(meta.filename, "test.jpg"),
        _ => panic!("Expected File variant"),
    }
}

#[test]
fn test_partial_sync_post_methods() {
    let post_id = crate::PostId(1);
    let now = Utc::now();

    // Create base UnsyncPost
    let unsync_post = UnsyncPost::new()
        .title("Test Title".to_string())
        .updated(now.clone())
        .published(now.clone());

    // Test new
    let partial = crate::importer::PartialSyncPost::new(post_id, unsync_post);
    assert_eq!(partial.id, post_id);
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
    };
    let partial = partial.thumb(Some(thumb.clone()));
    assert!(partial.thumb.is_some());

    // Test collect_files
    let partial = crate::importer::PartialSyncPost::new(post_id, UnsyncPost::new())
        .content(vec![
            UnsyncContent::Text("text".to_string()),
            UnsyncContent::File(UnsyncFileMeta {
                filename: "content.jpg".to_string(),
                mime: "image/jpeg".to_string(),
                extra: HashMap::new(),
            }),
        ])
        .thumb(Some(UnsyncFileMeta {
            filename: "thumb.jpg".to_string(),
            mime: "image/jpeg".to_string(),
            extra: HashMap::new(),
        }));

    let files = partial.collect_files();
    assert_eq!(files.len(), 2); // Should have both content and thumb files
}
