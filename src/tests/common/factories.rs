//! Test data factories for creating test fixtures
//!
//! Provides helper functions for creating consistent test data

use crate::{Author, Collection, Comment, Content, FileMeta, Platform, Post, Tag};
use crate::{AuthorId, CollectionId, FileMetaId, PlatformId, PostId, TagId};
use chrono::{TimeZone, Utc};

/// Creates a test author with default values
pub fn create_test_author(id: u32, name: &str) -> Author {
    // Use a fixed timestamp to avoid test flakiness
    let fixed_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
    Author {
        id: AuthorId::new(id),
        name: name.to_string(),
        thumb: None,
        updated: fixed_time,
    }
}

/// Creates a test post with default values
pub fn create_test_post(id: u32, title: &str) -> Post {
    // Use a fixed timestamp to avoid test flakiness
    let fixed_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
    Post {
        id: PostId::new(id),
        source: None,
        title: title.to_string(),
        content: vec![],
        thumb: None,
        comments: vec![],
        updated: fixed_time,
        published: fixed_time,
        platform: None,
    }
}

/// Creates a test tag with default values
pub fn create_test_tag(id: u32, name: &str) -> Tag {
    Tag {
        id: TagId::new(id),
        name: name.to_string(),
        platform: None,
    }
}

/// Creates a test platform with default values
pub fn create_test_platform(id: u32, name: &str) -> Platform {
    Platform {
        id: PlatformId::new(id),
        name: name.to_string(),
    }
}

/// Creates a test file meta with default values
pub fn create_test_file_meta(id: u32, filename: &str, post_id: u32) -> FileMeta {
    use std::collections::HashMap;
    FileMeta {
        id: FileMetaId::new(id),
        filename: filename.to_string(),
        post: PostId::new(post_id),
        mime: "text/plain".to_string(),
        extra: HashMap::new(),
    }
}

/// Creates a test collection with default values
pub fn create_test_collection(id: u32, name: &str) -> Collection {
    Collection {
        id: CollectionId::new(id),
        name: name.to_string(),
        source: None,
        thumb: None,
    }
}

/// Creates test content with text
pub fn create_test_text_content(text: &str) -> Content {
    Content::Text(text.to_string())
}

/// Creates test content with file reference
pub fn create_test_file_content(file_id: u32) -> Content {
    Content::File(FileMetaId::new(file_id))
}

/// Creates a test comment
pub fn create_test_comment(user: &str, text: &str) -> Comment {
    Comment {
        user: user.to_string(),
        text: text.to_string(),
        replies: vec![],
    }
}

/// Creates a nested test comment with replies
pub fn create_test_comment_with_replies(user: &str, text: &str, replies: Vec<Comment>) -> Comment {
    Comment {
        user: user.to_string(),
        text: text.to_string(),
        replies,
    }
}
