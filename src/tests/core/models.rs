//! Tests for core data models
//!
//! Comprehensive tests for data structures including creation,
//! serialization, hash operations, and field access.

use crate::tests::common::*;
use crate::{Author, Collection, Comment, Content, FileMeta, Platform, Post, Tag};
use crate::{AuthorId, CollectionId, FileMetaId, PlatformId, PostId, TagId};
use chrono::Utc;
use std::collections::HashMap;

#[test]
fn test_author_creation() {
    let author = create_test_author(1, "Test Author");

    assert_eq!(author.id, AuthorId::new(1));
    assert_eq!(author.name, "Test Author");
    assert_eq!(author.thumb, None);
    assert!(author.updated <= Utc::now());
}

#[test]
fn test_author_equality() {
    let author1 = create_test_author(1, "Test Author");
    let author2 = create_test_author(1, "Test Author");
    let author3 = create_test_author(2, "Different Author");

    assert_eq!(author1, author2);
    assert_ne!(author1, author3);
}

#[test]
fn test_author_serialization() {
    let author = create_test_author(42, "Serialization Test");

    let json = serde_json::to_string(&author).expect("Failed to serialize author");
    let deserialized: Author = serde_json::from_str(&json).expect("Failed to deserialize author");

    assert_eq!(author, deserialized);
}

#[test]
fn test_post_creation() {
    let post = create_test_post(1, "Test Post");

    assert_eq!(post.id, PostId::new(1));
    assert_eq!(post.title, "Test Post");
    assert_eq!(post.source, None);
    assert!(post.content.is_empty());
    assert!(post.comments.is_empty());
    assert_eq!(post.thumb, None);
    assert_eq!(post.platform, None);
}

#[test]
fn test_post_with_content() {
    let mut post = create_test_post(1, "Post with Content");
    post.content = vec![
        create_test_text_content("Hello, world!"),
        create_test_file_content(123),
    ];

    assert_eq!(post.content.len(), 2);
    match &post.content[0] {
        Content::Text(text) => assert_eq!(text, "Hello, world!"),
        _ => panic!("Expected text content"),
    }
    match &post.content[1] {
        Content::File(file_id) => assert_eq!(*file_id, FileMetaId::new(123)),
        _ => panic!("Expected file content"),
    }
}

#[test]
fn test_post_with_comments() {
    let mut post = create_test_post(1, "Post with Comments");
    post.comments = vec![
        create_test_comment("user1", "First comment"),
        create_test_comment_with_replies(
            "user2",
            "Second comment",
            vec![create_test_comment("user3", "Reply to second comment")],
        ),
    ];

    assert_eq!(post.comments.len(), 2);
    assert_eq!(post.comments[0].user, "user1");
    assert_eq!(post.comments[0].text, "First comment");
    assert!(post.comments[0].replies.is_empty());

    assert_eq!(post.comments[1].user, "user2");
    assert_eq!(post.comments[1].replies.len(), 1);
    assert_eq!(post.comments[1].replies[0].user, "user3");
}

#[test]
fn test_tag_creation() {
    let tag = create_test_tag(1, "test-tag");

    assert_eq!(tag.id, TagId::new(1));
    assert_eq!(tag.name, "test-tag");
    assert_eq!(tag.platform, None);
}

#[test]
fn test_tag_with_platform() {
    let mut tag = create_test_tag(1, "platform-tag");
    tag.platform = Some(PlatformId::new(42));

    assert_eq!(tag.platform, Some(PlatformId::new(42)));
}

#[test]
fn test_platform_creation() {
    let platform = create_test_platform(1, "Test Platform");

    assert_eq!(platform.id, PlatformId::new(1));
    assert_eq!(platform.name, "Test Platform");
}

#[test]
fn test_file_meta_creation() {
    let file_meta = create_test_file_meta(1, "test.txt", 100);

    assert_eq!(file_meta.id, FileMetaId::new(1));
    assert_eq!(file_meta.filename, "test.txt");
    assert_eq!(file_meta.post, PostId::new(100));
    assert_eq!(file_meta.mime, "text/plain");
    assert!(file_meta.extra.is_empty());
}

#[test]
fn test_file_meta_path() {
    let file_meta = create_test_file_meta(1, "example.txt", 2049);
    let path = file_meta.path();

    // 2049 / 2048 = 1, 2049 % 2048 = 1
    assert_eq!(path.to_str(), Some("1/1/example.txt"));
}

#[test]
fn test_file_meta_with_extra() {
    let mut file_meta = create_test_file_meta(1, "test.json", 100);
    file_meta.extra.insert(
        "size".to_string(),
        serde_json::Value::Number(serde_json::Number::from(1024)),
    );
    file_meta
        .extra
        .insert("compressed".to_string(), serde_json::Value::Bool(true));

    assert_eq!(file_meta.extra.len(), 2);
    assert_eq!(file_meta.extra["size"], 1024);
    assert_eq!(file_meta.extra["compressed"], true);
}

#[test]
fn test_collection_creation() {
    let collection = create_test_collection(1, "Test Collection");

    assert_eq!(collection.id, CollectionId::new(1));
    assert_eq!(collection.name, "Test Collection");
    assert_eq!(collection.source, None);
    assert_eq!(collection.thumb, None);
}

#[test]
fn test_comment_creation() {
    let comment = create_test_comment("testuser", "This is a test comment");

    assert_eq!(comment.user, "testuser");
    assert_eq!(comment.text, "This is a test comment");
    assert!(comment.replies.is_empty());
}

#[test]
fn test_nested_comments() {
    let reply1 = create_test_comment("user2", "First reply");
    let reply2 = create_test_comment("user3", "Second reply");
    let main_comment =
        create_test_comment_with_replies("user1", "Main comment", vec![reply1, reply2]);

    assert_eq!(main_comment.user, "user1");
    assert_eq!(main_comment.text, "Main comment");
    assert_eq!(main_comment.replies.len(), 2);
    assert_eq!(main_comment.replies[0].user, "user2");
    assert_eq!(main_comment.replies[1].user, "user3");
}

#[test]
fn test_content_variants() {
    let text_content = create_test_text_content("Hello, world!");
    let file_content = create_test_file_content(42);

    match text_content {
        Content::Text(text) => assert_eq!(text, "Hello, world!"),
        _ => panic!("Expected text content"),
    }

    match file_content {
        Content::File(file_id) => assert_eq!(file_id, FileMetaId::new(42)),
        _ => panic!("Expected file content"),
    }
}

#[test]
fn test_model_serialization() {
    // Test all models can be serialized and deserialized
    let author = create_test_author(1, "Test");
    let post = create_test_post(2, "Test Post");
    let tag = create_test_tag(3, "test-tag");
    let platform = create_test_platform(4, "Test Platform");
    let file_meta = create_test_file_meta(5, "test.txt", 2);
    let collection = create_test_collection(6, "Test Collection");
    let comment = create_test_comment("user", "test comment");

    // Test each model's serialization
    let author_json = serde_json::to_string(&author).expect("Author serialization failed");
    let _: Author = serde_json::from_str(&author_json).expect("Author deserialization failed");

    let post_json = serde_json::to_string(&post).expect("Post serialization failed");
    let _: Post = serde_json::from_str(&post_json).expect("Post deserialization failed");

    let tag_json = serde_json::to_string(&tag).expect("Tag serialization failed");
    let _: Tag = serde_json::from_str(&tag_json).expect("Tag deserialization failed");

    let platform_json = serde_json::to_string(&platform).expect("Platform serialization failed");
    let _: Platform =
        serde_json::from_str(&platform_json).expect("Platform deserialization failed");

    let file_meta_json = serde_json::to_string(&file_meta).expect("FileMeta serialization failed");
    let _: FileMeta =
        serde_json::from_str(&file_meta_json).expect("FileMeta deserialization failed");

    let collection_json =
        serde_json::to_string(&collection).expect("Collection serialization failed");
    let _: Collection =
        serde_json::from_str(&collection_json).expect("Collection deserialization failed");

    let comment_json = serde_json::to_string(&comment).expect("Comment serialization failed");
    let _: Comment = serde_json::from_str(&comment_json).expect("Comment deserialization failed");
}
