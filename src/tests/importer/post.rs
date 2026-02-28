//! Post importer tests
//!
//! Tests for post import functionality including creation,
//! updating existing posts, and handling content/files.

use crate::{
    importer::{
        collection::UnsyncCollection,
        post::{UnsyncContent, UnsyncPost},
        tag::UnsyncTag,
        UnsyncFileMeta,
    },
    manager::PostArchiverManager,
    tests::helpers,
    Comment, PlatformId,
};
use chrono::Utc;
use std::collections::HashMap;

#[test]
fn test_import_new_post() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".to_string());

    let content: Vec<UnsyncContent<()>> = vec![UnsyncContent::Text("Hello, world!".to_string())];
    let unsync_post = UnsyncPost::new(
        platform_id,
        "https://example.com/post/1".to_string(),
        "Test Post".to_string(),
        content,
    )
    .published(Utc::now())
    .updated(Utc::now());

    let (post_id, authors, collections, files) = manager
        .import_post(unsync_post, true)
        .expect("Failed to import post");

    assert!(post_id.raw() > 0);
    assert!(authors.is_empty());
    assert!(collections.is_empty());
    assert!(files.is_empty());

    let post = helpers::get_post(&manager, post_id);
    assert_eq!(post.title, "Test Post");
    assert_eq!(post.source, Some("https://example.com/post/1".to_string()));
    assert_eq!(post.platform, Some(platform_id));
}

#[test]
fn test_import_existing_post() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".to_string());

    let source = "https://example.com/post/1".to_string();

    let existing_post_id = helpers::add_post(
        &manager,
        "Original Title".to_string(),
        Some(source.clone()),
        Some(platform_id),
        Some(Utc::now()),
        Some(Utc::now()),
    );

    let content: Vec<UnsyncContent<()>> = vec![UnsyncContent::Text("Updated content".to_string())];
    let unsync_post = UnsyncPost::new(platform_id, source, "Updated Title".to_string(), content);

    let (post_id, _, _, _) = manager
        .import_post(unsync_post, true)
        .expect("Failed to import existing post");

    assert_eq!(post_id, existing_post_id);

    let post = helpers::get_post(&manager, post_id);
    assert_eq!(post.title, "Updated Title");
}

#[test]
fn test_unsync_post_builder() {
    let platform_id = PlatformId(1);
    let updated_time = Utc::now();
    let published_time = Utc::now();

    let content: Vec<UnsyncContent<()>> =
        vec![UnsyncContent::Text("Builder test content".to_string())];
    let tags = vec![UnsyncTag {
        name: "test_tag".to_string(),
        platform: Some(platform_id),
    }];

    let post = UnsyncPost::new(
        platform_id,
        "https://example.com/builder".to_string(),
        "Builder Test".to_string(),
        content.clone(),
    )
    .title("Updated Builder Test".to_string())
    .source("https://example.com/updated_builder".to_string())
    .updated(updated_time)
    .published(published_time)
    .tags(tags.clone())
    .comments(vec![Comment {
        user: "Commenter".to_string(),
        text: "Great post!".to_string(),
        replies: vec![],
    }]);

    assert_eq!(post.title, "Updated Builder Test");
    assert_eq!(post.source, "https://example.com/updated_builder");
    assert_eq!(post.updated, Some(updated_time));
    assert_eq!(post.published, Some(published_time));
    assert_eq!(post.tags.len(), 1);
    assert_eq!(post.comments.len(), 1);
}

#[test]
fn test_import_post_with_text_content() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".to_string());

    let content: Vec<UnsyncContent<()>> = vec![
        UnsyncContent::Text("First paragraph".to_string()),
        UnsyncContent::Text("Second paragraph".to_string()),
    ];

    let unsync_post = UnsyncPost::new(
        platform_id,
        "https://example.com/text_post".to_string(),
        "Text Post".to_string(),
        content,
    )
    .published(Utc::now())
    .updated(Utc::now());

    let (post_id, _, _, _) = manager
        .import_post(unsync_post, true)
        .expect("Failed to import post with text content");

    let post = helpers::get_post(&manager, post_id);
    assert_eq!(post.content.len(), 2);

    if let crate::Content::Text(text) = &post.content[0] {
        assert_eq!(text, "First paragraph");
    } else {
        panic!("Expected text content");
    }

    if let crate::Content::Text(text) = &post.content[1] {
        assert_eq!(text, "Second paragraph");
    } else {
        panic!("Expected text content");
    }
}

#[test]
fn test_import_post_with_file_content() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".to_string());

    let file_meta = UnsyncFileMeta {
        filename: "test_image.jpg".to_string(),
        mime: "image/jpeg".to_string(),
        extra: HashMap::new(),
        data: (),
    };

    let content = vec![
        UnsyncContent::Text("Check out this image:".to_string()),
        UnsyncContent::File(file_meta),
    ];

    let unsync_post = UnsyncPost::new(
        platform_id,
        "https://example.com/file_post".to_string(),
        "File Post".to_string(),
        content,
    )
    .published(Utc::now())
    .updated(Utc::now());

    let (post_id, _, _, _) = manager
        .import_post(unsync_post, true)
        .expect("Failed to import post with file content");

    let post = helpers::get_post(&manager, post_id);
    assert_eq!(post.content.len(), 2);

    if let crate::Content::Text(text) = &post.content[0] {
        assert_eq!(text, "Check out this image:");
    } else {
        panic!("Expected text content");
    }

    if let crate::Content::File(file_id) = &post.content[1] {
        let file = helpers::get_file_meta(&manager, *file_id);
        assert_eq!(file.filename, "test_image.jpg");
        assert_eq!(file.mime, "image/jpeg");
    } else {
        panic!("Expected file content");
    }
}

#[test]
fn test_import_post_with_authors() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".to_string());

    let author1_id = helpers::add_author(&manager, "Author 1".to_string(), Some(Utc::now()));
    let author2_id = helpers::add_author(&manager, "Author 2".to_string(), Some(Utc::now()));

    let content: Vec<UnsyncContent<()>> =
        vec![UnsyncContent::Text("Post by multiple authors".to_string())];
    let unsync_post = UnsyncPost::new(
        platform_id,
        "https://example.com/multi_author".to_string(),
        "Multi Author Post".to_string(),
        content,
    )
    .authors(vec![author1_id, author2_id])
    .published(Utc::now())
    .updated(Utc::now());

    let (post_id, authors, _, _) = manager
        .import_post(unsync_post, true)
        .expect("Failed to import post with authors");

    assert_eq!(authors.len(), 2);
    assert!(authors.contains(&author1_id));
    assert!(authors.contains(&author2_id));

    let post_authors = helpers::list_post_authors(&manager, post_id);
    assert_eq!(post_authors.len(), 2);
}

#[test]
fn test_import_post_with_tags() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".to_string());

    let tags = vec![
        UnsyncTag {
            name: "rust".to_string(),
            platform: Some(platform_id),
        },
        UnsyncTag {
            name: "programming".to_string(),
            platform: None,
        },
    ];

    let content: Vec<UnsyncContent<()>> = vec![UnsyncContent::Text(
        "A post about Rust programming".to_string(),
    )];
    let unsync_post = UnsyncPost::new(
        platform_id,
        "https://example.com/rust_post".to_string(),
        "Rust Post".to_string(),
        content,
    )
    .tags(tags)
    .published(Utc::now())
    .updated(Utc::now());

    let (post_id, _, _, _) = manager
        .import_post(unsync_post, true)
        .expect("Failed to import post with tags");

    let post_tags = helpers::list_post_tags(&manager, post_id);
    assert_eq!(post_tags.len(), 2);

    let tag_names: Vec<String> = post_tags.iter().map(|t| t.name.clone()).collect();
    assert!(tag_names.contains(&"rust".to_string()));
    assert!(tag_names.contains(&"programming".to_string()));
}

#[test]
fn test_import_post_with_collections() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".to_string());

    let collections = vec![
        UnsyncCollection {
            name: "Tech Posts".to_string(),
            source: "tech_posts".to_string(),
        },
        UnsyncCollection {
            name: "Tutorials".to_string(),
            source: "tutorials".to_string(),
        },
    ];

    let content: Vec<UnsyncContent<()>> = vec![UnsyncContent::Text("A tutorial post".to_string())];
    let unsync_post = UnsyncPost::new(
        platform_id,
        "https://example.com/tutorial".to_string(),
        "Tutorial Post".to_string(),
        content,
    )
    .collections(collections)
    .published(Utc::now())
    .updated(Utc::now());

    let (post_id, _, collection_ids, _) = manager
        .import_post(unsync_post, true)
        .expect("Failed to import post with collections");

    assert_eq!(collection_ids.len(), 2);

    let post_collections = helpers::list_post_collections(&manager, post_id);
    assert_eq!(post_collections.len(), 2);
}

#[test]
fn test_import_post_with_comments() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".to_string());

    let comments = vec![
        Comment {
            user: "User1".to_string(),
            text: "Great post!".to_string(),
            replies: vec![],
        },
        Comment {
            user: "User2".to_string(),
            text: "Thanks for sharing!".to_string(),
            replies: vec![],
        },
    ];

    let content: Vec<UnsyncContent<()>> =
        vec![UnsyncContent::Text("A post with comments".to_string())];
    let unsync_post = UnsyncPost::new(
        platform_id,
        "https://example.com/commented_post".to_string(),
        "Commented Post".to_string(),
        content,
    )
    .comments(comments.clone())
    .published(Utc::now())
    .updated(Utc::now());

    let (post_id, _, _, _) = manager
        .import_post(unsync_post, true)
        .expect("Failed to import post with comments");

    let post = helpers::get_post(&manager, post_id);
    assert_eq!(post.comments.len(), 2);
    assert_eq!(post.comments[0].user, "User1");
    assert_eq!(post.comments[1].user, "User2");
}

#[test]
fn test_import_posts_multiple() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".to_string());

    let posts: Vec<UnsyncPost<()>> = vec![
        UnsyncPost::new(
            platform_id,
            "https://example.com/post1".to_string(),
            "Post 1".to_string(),
            vec![UnsyncContent::Text("Content 1".to_string())],
        )
        .published(Utc::now())
        .updated(Utc::now()),
        UnsyncPost::new(
            platform_id,
            "https://example.com/post2".to_string(),
            "Post 2".to_string(),
            vec![UnsyncContent::Text("Content 2".to_string())],
        )
        .published(Utc::now())
        .updated(Utc::now()),
    ];

    let (post_ids, files) = manager
        .import_posts(posts, true)
        .expect("Failed to import multiple posts");

    assert_eq!(post_ids.len(), 2);
    assert!(files.is_empty());

    for post_id in post_ids {
        let post = helpers::get_post(&manager, post_id);
        assert!(post.title.starts_with("Post "));
    }
}

#[test]
fn test_import_post_with_thumbnail() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".to_string());

    let thumb = UnsyncFileMeta {
        filename: "thumbnail.jpg".to_string(),
        mime: "image/jpeg".to_string(),
        extra: HashMap::new(),
        data: (),
    };

    let content = vec![UnsyncContent::Text("Post with thumbnail".to_string())];
    let unsync_post = UnsyncPost::new(
        platform_id,
        "https://example.com/thumb_post".to_string(),
        "Thumbnail Post".to_string(),
        content,
    )
    .thumb(Some(thumb))
    .published(Utc::now())
    .updated(Utc::now());

    let (post_id, _, _, _) = manager
        .import_post(unsync_post, true)
        .expect("Failed to import post with thumbnail");

    let post = helpers::get_post(&manager, post_id);
    assert!(post.thumb.is_some());

    let thumb_file = helpers::get_file_meta(&manager, post.thumb.unwrap());
    assert_eq!(thumb_file.filename, "thumbnail.jpg");
}

#[test]
fn test_import_post_auto_thumbnail_from_content() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".to_string());

    let image_file = UnsyncFileMeta {
        filename: "content_image.png".to_string(),
        mime: "image/png".to_string(),
        extra: HashMap::new(),
        data: (),
    };

    let content = vec![
        UnsyncContent::Text("Check this out:".to_string()),
        UnsyncContent::File(image_file),
        UnsyncContent::Text("Great image, right?".to_string()),
    ];

    let unsync_post = UnsyncPost::new(
        platform_id,
        "https://example.com/auto_thumb".to_string(),
        "Auto Thumbnail Post".to_string(),
        content,
    )
    .published(Utc::now())
    .updated(Utc::now());

    let (post_id, _, _, _) = manager
        .import_post(unsync_post, true)
        .expect("Failed to import post with auto thumbnail");

    let post = helpers::get_post(&manager, post_id);
    assert!(post.thumb.is_some());

    let thumb_file = helpers::get_file_meta(&manager, post.thumb.unwrap());
    assert_eq!(thumb_file.filename, "content_image.png");
}

#[test]
fn test_unsync_post_sync_method() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".to_string());

    let content: Vec<UnsyncContent<()>> = vec![UnsyncContent::Text("Sync method test".to_string())];
    let post = UnsyncPost::new(
        platform_id,
        "https://example.com/sync_test".to_string(),
        "Sync Test".to_string(),
        content,
    )
    .published(Utc::now())
    .updated(Utc::now());

    let (post_id, files) = post.sync(&manager).expect("Failed to sync post");

    assert!(post_id.raw() > 0);
    assert!(files.is_empty());

    let stored_post = helpers::get_post(&manager, post_id);
    assert_eq!(stored_post.title, "Sync Test");
}

#[test]
fn test_unsync_content_variants() {
    let text_content: UnsyncContent<()> = UnsyncContent::Text("Text content".to_string());
    let file_content: UnsyncContent<()> = UnsyncContent::File(UnsyncFileMeta {
        filename: "test.txt".to_string(),
        mime: "text/plain".to_string(),
        extra: HashMap::new(),
        data: (),
    });

    let text_debug = format!("{:?}", text_content);
    let file_debug = format!("{:?}", file_content);

    assert!(text_debug.contains("Text"));
    assert!(file_debug.contains("File"));

    let _text_clone = text_content.clone();
    let _file_clone = file_content.clone();
}

#[test]
fn test_import_post_without_update_relation() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".to_string());

    let author_id = helpers::add_author(&manager, "Test Author".to_string(), Some(Utc::now()));

    let content: Vec<UnsyncContent<()>> =
        vec![UnsyncContent::Text("No relation update test".to_string())];
    let unsync_post = UnsyncPost::new(
        platform_id,
        "https://example.com/no_update".to_string(),
        "No Update Test".to_string(),
        content,
    )
    .authors(vec![author_id])
    .published(Utc::now())
    .updated(Utc::now());

    let (post_id, authors, _, _) = manager
        .import_post(unsync_post, false)
        .expect("Failed to import post without relation update");

    assert!(post_id.raw() > 0);
    assert_eq!(authors.len(), 1);
    assert_eq!(authors[0], author_id);
}
