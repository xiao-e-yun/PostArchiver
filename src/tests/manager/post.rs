//! Post manager tests
//!
//! Tests for post CRUD operations, relationship management,
//! and content handling.

use crate::{
    manager::PostArchiverManager, AuthorId, CollectionId, Comment, Content, PostId, TagId,
};
use chrono::Utc;
use std::collections::HashMap;

#[test]
fn test_add_post() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let title = "Test Post".to_string();
    let now = Utc::now();

    let post_id = manager
        .add_post(title.clone(), None, None, Some(now), Some(now))
        .expect("Failed to add post");

    assert!(post_id.raw() > 0);

    // Verify the post was added
    let post = manager.get_post(&post_id).expect("Failed to get post");

    assert_eq!(post.title, title);
    assert_eq!(post.id, post_id);
    assert_eq!(post.source, None);
    assert_eq!(post.platform, None);
}

#[test]
fn test_add_post_with_source_and_platform() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Add platform first
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    let title = "Test Post".to_string();
    let source = Some("https://example.com/post/123".to_string());
    let now = Utc::now();

    let post_id = manager
        .add_post(
            title.clone(),
            source.clone(),
            Some(platform_id),
            Some(now),
            Some(now),
        )
        .expect("Failed to add post");

    let post = manager.get_post(&post_id).expect("Failed to get post");

    assert_eq!(post.title, title);
    assert_eq!(post.source, source);
    assert_eq!(post.platform, Some(platform_id));
}

#[test]
fn test_list_posts() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    // Add multiple posts
    let id1 = manager
        .add_post("Post 1".to_string(), None, None, Some(now), Some(now))
        .expect("Failed to add post 1");
    let id2 = manager
        .add_post("Post 2".to_string(), None, None, Some(now), Some(now))
        .expect("Failed to add post 2");

    let posts = manager.list_posts().expect("Failed to list posts");

    assert_eq!(posts.len(), 2);

    let ids: Vec<PostId> = posts.iter().map(|p| p.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

#[test]
fn test_get_post() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let title = "Get Test Post".to_string();
    let now = Utc::now();

    let post_id = manager
        .add_post(title.clone(), None, None, Some(now), Some(now))
        .expect("Failed to add post");

    let post = manager.get_post(&post_id).expect("Failed to get post");

    assert_eq!(post.id, post_id);
    assert_eq!(post.title, title);
}

#[test]
fn test_find_post() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let source = "https://example.com/unique-post".to_string();
    let now = Utc::now();

    let post_id = manager
        .add_post(
            "Findable Post".to_string(),
            Some(source.clone()),
            None,
            Some(now),
            Some(now),
        )
        .expect("Failed to add post");

    // Find post by source
    let found_id = manager.find_post(&source).expect("Failed to find post");

    assert_eq!(found_id, Some(post_id));

    // Test not found
    let not_found = manager
        .find_post("https://example.com/nonexistent")
        .expect("Failed to search for nonexistent post");

    assert_eq!(not_found, None);
}

#[test]
fn test_find_post_with_updated() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let source = "https://example.com/updated-post".to_string();
    let early_time = Utc::now();
    let later_time = early_time + chrono::Duration::hours(1);

    let post_id = manager
        .add_post(
            "Updated Post".to_string(),
            Some(source.clone()),
            None,
            Some(early_time),
            Some(later_time),
        )
        .expect("Failed to add post");

    // Find with earlier time - should find the post
    let found_id = manager
        .find_post_with_updated(&source, &early_time)
        .expect("Failed to find post with updated");

    assert_eq!(found_id, Some(post_id));

    // Find with much later time - should not find the post
    let much_later = later_time + chrono::Duration::hours(2);
    let not_found = manager
        .find_post_with_updated(&source, &much_later)
        .expect("Failed to search for post with later updated");

    assert_eq!(not_found, None);
}

#[test]
fn test_remove_post() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    let post_id = manager
        .add_post("To Delete".to_string(), None, None, Some(now), Some(now))
        .expect("Failed to add post");

    // Verify post exists
    manager
        .get_post(&post_id)
        .expect("Post should exist before deletion");

    // Remove post
    manager.remove_post(post_id).expect("Failed to remove post");

    // Verify post is gone
    let result = manager.get_post(&post_id);
    assert!(result.is_err(), "Post should not exist after deletion");
}

#[test]
fn test_set_post_title() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    let post_id = manager
        .add_post(
            "Original Title".to_string(),
            None,
            None,
            Some(now),
            Some(now),
        )
        .expect("Failed to add post");

    let new_title = "Updated Title".to_string();
    manager
        .set_post_title(post_id, new_title.clone())
        .expect("Failed to update post title");

    let post = manager.get_post(&post_id).expect("Failed to get post");

    assert_eq!(post.title, new_title);
}

#[test]
fn test_set_post_source() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    let post_id = manager
        .add_post("Test Post".to_string(), None, None, Some(now), Some(now))
        .expect("Failed to add post");

    let new_source = Some("https://example.com/new-source".to_string());
    manager
        .set_post_source(post_id, new_source.clone())
        .expect("Failed to set post source");

    let post = manager.get_post(&post_id).expect("Failed to get post");

    assert_eq!(post.source, new_source);

    // Test setting source to None
    manager
        .set_post_source(post_id, None)
        .expect("Failed to set post source to None");

    let post = manager.get_post(&post_id).expect("Failed to get post");
    assert_eq!(post.source, None);
}

#[test]
fn test_set_post_platform() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    // Add platform
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    let post_id = manager
        .add_post("Test Post".to_string(), None, None, Some(now), Some(now))
        .expect("Failed to add post");

    manager
        .set_post_platform(post_id, Some(platform_id))
        .expect("Failed to set post platform");

    let post = manager.get_post(&post_id).expect("Failed to get post");

    assert_eq!(post.platform, Some(platform_id));

    // Test setting platform to None
    manager
        .set_post_platform(post_id, None)
        .expect("Failed to set post platform to None");

    let post = manager.get_post(&post_id).expect("Failed to get post");
    assert_eq!(post.platform, None);
}

#[test]
fn test_set_post_published() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    let post_id = manager
        .add_post("Test Post".to_string(), None, None, Some(now), Some(now))
        .expect("Failed to add post");

    let new_published = Utc::now();
    manager
        .set_post_published(post_id, new_published)
        .expect("Failed to set post published");

    let post = manager.get_post(&post_id).expect("Failed to get post");

    // Allow small time difference due to precision
    let diff = (post.published - new_published).num_milliseconds().abs();
    assert!(
        diff < 1000,
        "Published timestamp should be close to expected"
    );
}

#[test]
fn test_set_post_updated() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    let post_id = manager
        .add_post("Test Post".to_string(), None, None, Some(now), Some(now))
        .expect("Failed to add post");

    let new_updated = Utc::now();
    manager
        .set_post_updated(post_id, new_updated)
        .expect("Failed to set post updated");

    let post = manager.get_post(&post_id).expect("Failed to get post");

    // Allow small time difference due to precision
    let diff = (post.updated - new_updated).num_milliseconds().abs();
    assert!(diff < 1000, "Updated timestamp should be close to expected");
}

#[test]
fn test_set_post_updated_by_latest() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let old_time = Utc::now() - chrono::Duration::hours(1);
    let new_time = Utc::now();

    let post_id = manager
        .add_post(
            "Test Post".to_string(),
            None,
            None,
            Some(old_time),
            Some(old_time),
        )
        .expect("Failed to add post");

    // Update with newer timestamp - should update
    manager
        .set_post_updated_by_latest(post_id, new_time)
        .expect("Failed to set post updated by latest");

    let post = manager.get_post(&post_id).expect("Failed to get post");
    let diff = (post.updated - new_time).num_milliseconds().abs();
    assert!(
        diff < 1000,
        "Updated timestamp should be updated to newer time"
    );

    // Try to update with older timestamp - should not update
    let even_older = old_time - chrono::Duration::hours(1);
    manager
        .set_post_updated_by_latest(post_id, even_older)
        .expect("Failed to call set_post_updated_by_latest");

    let post = manager.get_post(&post_id).expect("Failed to get post");
    let diff = (post.updated - new_time).num_milliseconds().abs();
    assert!(
        diff < 1000,
        "Updated timestamp should not change to older time"
    );
}

#[test]
fn test_set_post_content() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    let post_id = manager
        .add_post("Test Post".to_string(), None, None, Some(now), Some(now))
        .expect("Failed to add post");

    let content = vec![
        Content::Text("Hello world!".to_string()),
        Content::Text("This is test content.".to_string()),
    ];

    manager
        .set_post_content(post_id, content.clone())
        .expect("Failed to set post content");

    let post = manager.get_post(&post_id).expect("Failed to get post");

    assert_eq!(post.content, content);
}

#[test]
fn test_set_post_comments() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    let post_id = manager
        .add_post("Test Post".to_string(), None, None, Some(now), Some(now))
        .expect("Failed to add post");

    let comments = vec![
        Comment {
            user: "user1".to_string(),
            text: "Great post!".to_string(),
            replies: vec![],
        },
        Comment {
            user: "user2".to_string(),
            text: "Thanks for sharing.".to_string(),
            replies: vec![],
        },
    ];

    manager
        .set_post_comments(post_id, comments.clone())
        .expect("Failed to set post comments");

    let post = manager.get_post(&post_id).expect("Failed to get post");

    assert_eq!(post.comments, comments);
}

#[test]
fn test_set_post_thumb() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    let post_id = manager
        .add_post("Test Post".to_string(), None, None, Some(now), Some(now))
        .expect("Failed to add post");

    // Add a file meta for thumbnail
    let file_meta_id = manager
        .add_file_meta(
            post_id,
            "thumbnail.jpg".to_string(),
            "image/jpeg".to_string(),
            HashMap::new(),
        )
        .expect("Failed to add file meta");

    // Set post thumbnail
    manager
        .set_post_thumb(post_id, Some(file_meta_id))
        .expect("Failed to set post thumb");

    // Verify thumbnail was set
    let post = manager.get_post(&post_id).expect("Failed to get post");
    assert_eq!(post.thumb, Some(file_meta_id));

    // Test setting thumbnail to None
    manager
        .set_post_thumb(post_id, None)
        .expect("Failed to set post thumb to None");

    let post = manager.get_post(&post_id).expect("Failed to get post");
    assert_eq!(post.thumb, None);
}

#[test]
fn test_add_post_authors() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    // Add authors and post
    let author1_id = manager
        .add_author("Author 1".to_string(), Some(now))
        .expect("Failed to add author 1");
    let author2_id = manager
        .add_author("Author 2".to_string(), Some(now))
        .expect("Failed to add author 2");

    let post_id = manager
        .add_post("Test Post".to_string(), None, None, Some(now), Some(now))
        .expect("Failed to add post");

    // Add authors to post
    manager
        .add_post_authors(post_id, &[author1_id, author2_id])
        .expect("Failed to add post authors");

    // Verify authors were added
    let post = manager.get_post(&post_id).expect("Failed to get post");
    let post_authors = manager
        .list_post_authors(&post)
        .expect("Failed to list post authors");

    assert_eq!(post_authors.len(), 2);

    let author_ids: Vec<AuthorId> = post_authors.iter().map(|a| a.id).collect();
    assert!(author_ids.contains(&author1_id));
    assert!(author_ids.contains(&author2_id));
}

#[test]
fn test_remove_post_authors() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    // Setup
    let author1_id = manager
        .add_author("Author 1".to_string(), Some(now))
        .expect("Failed to add author 1");
    let author2_id = manager
        .add_author("Author 2".to_string(), Some(now))
        .expect("Failed to add author 2");

    let post_id = manager
        .add_post("Test Post".to_string(), None, None, Some(now), Some(now))
        .expect("Failed to add post");

    // Add authors
    manager
        .add_post_authors(post_id, &[author1_id, author2_id])
        .expect("Failed to add post authors");

    // Remove one author
    manager
        .remove_post_authors(post_id, &[author1_id])
        .expect("Failed to remove post authors");

    // Verify only one author remains
    let post = manager.get_post(&post_id).expect("Failed to get post");
    let remaining_authors = manager
        .list_post_authors(&post)
        .expect("Failed to list post authors");

    assert_eq!(remaining_authors.len(), 1);
    assert_eq!(remaining_authors[0].id, author2_id);
}

#[test]
fn test_add_post_tags() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    // Add platform and tags
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    let tag1_id = manager
        .add_tag("tag1".to_string(), Some(platform_id))
        .expect("Failed to add tag 1");
    let tag2_id = manager
        .add_tag("tag2".to_string(), Some(platform_id))
        .expect("Failed to add tag 2");

    let post_id = manager
        .add_post("Test Post".to_string(), None, None, Some(now), Some(now))
        .expect("Failed to add post");

    // Add tags to post
    manager
        .add_post_tags(post_id, &[tag1_id, tag2_id])
        .expect("Failed to add post tags");

    // Verify tags were added
    let _post = manager.get_post(&post_id).expect("Failed to get post");
    let post_tags = manager
        .list_post_tags(&post_id)
        .expect("Failed to list post tags");

    assert_eq!(post_tags.len(), 2);

    let tag_ids: Vec<TagId> = post_tags.iter().map(|t| t.id).collect();
    assert!(tag_ids.contains(&tag1_id));
    assert!(tag_ids.contains(&tag2_id));
}

#[test]
fn test_remove_post_tags() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    // Setup
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    let tag1_id = manager
        .add_tag("tag1".to_string(), Some(platform_id))
        .expect("Failed to add tag 1");
    let tag2_id = manager
        .add_tag("tag2".to_string(), Some(platform_id))
        .expect("Failed to add tag 2");

    let post_id = manager
        .add_post("Test Post".to_string(), None, None, Some(now), Some(now))
        .expect("Failed to add post");

    // Add tags
    manager
        .add_post_tags(post_id, &[tag1_id, tag2_id])
        .expect("Failed to add post tags");

    // Remove one tag
    manager
        .remove_post_tags(post_id, &[tag1_id])
        .expect("Failed to remove post tags");

    // Verify only one tag remains
    let _post = manager.get_post(&post_id).expect("Failed to get post");
    let remaining_tags = manager
        .list_post_tags(&post_id)
        .expect("Failed to list post tags");

    assert_eq!(remaining_tags.len(), 1);
    assert_eq!(remaining_tags[0].id, tag2_id);
}

#[test]
fn test_add_post_collections() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    // Add collections
    let collection1_id = manager
        .add_collection("Collection 1".to_string(), None, None)
        .expect("Failed to add collection 1");
    let collection2_id = manager
        .add_collection("Collection 2".to_string(), None, None)
        .expect("Failed to add collection 2");

    let post_id = manager
        .add_post("Test Post".to_string(), None, None, Some(now), Some(now))
        .expect("Failed to add post");

    // Add collections to post
    manager
        .add_post_collections(post_id, &[collection1_id, collection2_id])
        .expect("Failed to add post collections");

    // Verify collections were added
    let _post = manager.get_post(&post_id).expect("Failed to get post");
    let post_collections = manager
        .list_post_collections(&post_id)
        .expect("Failed to list post collections");

    assert_eq!(post_collections.len(), 2);

    let collection_ids: Vec<CollectionId> = post_collections.iter().map(|c| c.id).collect();
    assert!(collection_ids.contains(&collection1_id));
    assert!(collection_ids.contains(&collection2_id));
}

#[test]
fn test_remove_post_collections() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    // Setup
    let collection1_id = manager
        .add_collection("Collection 1".to_string(), None, None)
        .expect("Failed to add collection 1");
    let collection2_id = manager
        .add_collection("Collection 2".to_string(), None, None)
        .expect("Failed to add collection 2");

    let post_id = manager
        .add_post("Test Post".to_string(), None, None, Some(now), Some(now))
        .expect("Failed to add post");

    // Add collections
    manager
        .add_post_collections(post_id, &[collection1_id, collection2_id])
        .expect("Failed to add post collections");

    // Remove one collection
    manager
        .remove_post_collections(post_id, &[collection1_id])
        .expect("Failed to remove post collections");

    // Verify only one collection remains
    let _post = manager.get_post(&post_id).expect("Failed to get post");
    let remaining_collections = manager
        .list_post_collections(&post_id)
        .expect("Failed to list post collections");

    assert_eq!(remaining_collections.len(), 1);
    assert_eq!(remaining_collections[0].id, collection2_id);
}

#[test]
fn test_post_relationships_bidirectional() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    // Setup entities
    let author_id = manager
        .add_author("Test Author".to_string(), Some(now))
        .expect("Failed to add author");

    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    let tag_id = manager
        .add_tag("test-tag".to_string(), Some(platform_id))
        .expect("Failed to add tag");

    let collection_id = manager
        .add_collection("Test Collection".to_string(), None, None)
        .expect("Failed to add collection");

    let post_id = manager
        .add_post("Test Post".to_string(), None, None, Some(now), Some(now))
        .expect("Failed to add post");

    // Add relationships
    manager
        .add_post_authors(post_id, &[author_id])
        .expect("Failed to add post authors");
    manager
        .add_post_tags(post_id, &[tag_id])
        .expect("Failed to add post tags");
    manager
        .add_post_collections(post_id, &[collection_id])
        .expect("Failed to add post collections");

    // Test post -> entities
    let post = manager.get_post(&post_id).expect("Failed to get post");
    let post_authors = manager
        .list_post_authors(&post)
        .expect("Failed to list post authors");
    let post_tags = manager
        .list_post_tags(&post_id)
        .expect("Failed to list post tags");
    let post_collections = manager
        .list_post_collections(&post_id)
        .expect("Failed to list post collections");

    assert_eq!(post_authors.len(), 1);
    assert_eq!(post_authors[0].id, author_id);
    assert_eq!(post_tags.len(), 1);
    assert_eq!(post_tags[0].id, tag_id);
    assert_eq!(post_collections.len(), 1);
    assert_eq!(post_collections[0].id, collection_id);

    // Test entities -> post
    let author_posts = manager
        .list_author_posts(author_id)
        .expect("Failed to list author posts");
    let tag_posts = manager
        .list_tag_posts(&tag_id)
        .expect("Failed to list tag posts");
    let collection_posts = manager
        .list_collection_posts(&collection_id)
        .expect("Failed to list collection posts");

    assert_eq!(author_posts.len(), 1);
    assert_eq!(author_posts[0].id, post_id);
    assert_eq!(tag_posts.len(), 1);
    assert_eq!(tag_posts[0].id, post_id);
    assert_eq!(collection_posts.len(), 1);
    assert_eq!(collection_posts[0].id, post_id);
}

#[test]
fn test_post_content_with_files() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    let post_id = manager
        .add_post("Test Post".to_string(), None, None, Some(now), Some(now))
        .expect("Failed to add post");

    // Add file meta for content
    let file_meta_id = manager
        .add_file_meta(
            post_id,
            "image.jpg".to_string(),
            "image/jpeg".to_string(),
            HashMap::new(),
        )
        .expect("Failed to add file meta");

    let content = vec![
        Content::Text("Check out this image:".to_string()),
        Content::File(file_meta_id),
        Content::Text("What do you think?".to_string()),
    ];

    manager
        .set_post_content(post_id, content.clone())
        .expect("Failed to set post content");

    let post = manager.get_post(&post_id).expect("Failed to get post");

    assert_eq!(post.content, content);
}

#[test]
fn test_empty_post_content_and_comments() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    let post_id = manager
        .add_post("Empty Post".to_string(), None, None, Some(now), Some(now))
        .expect("Failed to add post");

    // Set empty content and comments
    manager
        .set_post_content(post_id, vec![])
        .expect("Failed to set empty post content");
    manager
        .set_post_comments(post_id, vec![])
        .expect("Failed to set empty post comments");

    let post = manager.get_post(&post_id).expect("Failed to get post");

    assert_eq!(post.content, vec![]);
    assert_eq!(post.comments, vec![]);
}
