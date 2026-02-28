//! Post manager tests
//!
//! Tests for post Binded operations: update/delete and relationship management.

use crate::{
    manager::{PostArchiverManager, UpdatePost},
    tests::helpers,
    AuthorId, CollectionId, Comment, Content, PostId, TagId,
};
use chrono::Utc;
use std::collections::HashMap;

// ── CRUD via helpers ──────────────────────────────────────────

#[test]
fn test_add_post() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );
    assert!(post_id.raw() > 0);

    let post = helpers::get_post(&manager, post_id);
    assert_eq!(post.title, "Test Post");
    assert_eq!(post.source, None);
    assert_eq!(post.platform, None);
}

#[test]
fn test_add_post_with_source_and_platform() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".into());
    let now = Utc::now();
    let source = Some("https://example.com/post/123".to_string());

    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        source.clone(),
        Some(platform_id),
        Some(now),
        Some(now),
    );

    let post = helpers::get_post(&manager, post_id);
    assert_eq!(post.title, "Test Post");
    assert_eq!(post.source, source);
    assert_eq!(post.platform, Some(platform_id));
}

#[test]
fn test_list_posts() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let id1 = helpers::add_post(&manager, "Post 1".into(), None, None, Some(now), Some(now));
    let id2 = helpers::add_post(&manager, "Post 2".into(), None, None, Some(now), Some(now));

    let posts = helpers::list_posts(&manager);
    assert_eq!(posts.len(), 2);
    let ids: Vec<PostId> = posts.iter().map(|p| p.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

#[test]
fn test_get_post() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post_id = helpers::add_post(
        &manager,
        "Get Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    let post = helpers::get_post(&manager, post_id);
    assert_eq!(post.id, post_id);
    assert_eq!(post.title, "Get Test Post");
}

#[test]
fn test_find_post() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let source = "https://example.com/unique-post".to_string();
    let now = Utc::now();
    let post_id = helpers::add_post(
        &manager,
        "Findable Post".into(),
        Some(source.clone()),
        None,
        Some(now),
        Some(now),
    );

    let found_id = helpers::find_post(&manager, &source);
    assert_eq!(found_id, Some(post_id));

    let not_found = helpers::find_post(&manager, "https://example.com/nonexistent");
    assert_eq!(not_found, None);
}

#[test]
fn test_find_post_with_updated() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let source = "https://example.com/updated-post".to_string();
    let early_time = Utc::now();
    let later_time = early_time + chrono::Duration::hours(1);

    let post_id = helpers::add_post(
        &manager,
        "Updated Post".into(),
        Some(source.clone()),
        None,
        Some(early_time),
        Some(later_time),
    );

    let found_id = helpers::find_post_with_updated(&manager, &source, &early_time);
    assert_eq!(found_id, Some(post_id));

    let much_later = later_time + chrono::Duration::hours(2);
    let not_found = helpers::find_post_with_updated(&manager, &source, &much_later);
    assert_eq!(not_found, None);
}

// ── Binded: Delete ────────────────────────────────────────────

#[test]
fn test_remove_post() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post_id = helpers::add_post(
        &manager,
        "To Delete".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    let _ = helpers::get_post(&manager, post_id);
    manager.bind(post_id).delete().unwrap();

    let posts = helpers::list_posts(&manager);
    assert!(posts.iter().all(|p| p.id != post_id));
}

// ── Binded: Set fields ───────────────────────────────────────

#[test]
fn test_set_post_title() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post_id = helpers::add_post(
        &manager,
        "Original Title".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    manager
        .bind(post_id)
        .update(UpdatePost::default().title("Updated Title".into()))
        .unwrap();

    let post = helpers::get_post(&manager, post_id);
    assert_eq!(post.title, "Updated Title");
}

#[test]
fn test_set_post_source() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    let new_source = Some("https://example.com/new-source".to_string());
    manager
        .bind(post_id)
        .update(UpdatePost::default().source(new_source.clone()))
        .unwrap();

    let post = helpers::get_post(&manager, post_id);
    assert_eq!(post.source, new_source);

    // Set to None
    manager
        .bind(post_id)
        .update(UpdatePost::default().source(None))
        .unwrap();
    let post = helpers::get_post(&manager, post_id);
    assert_eq!(post.source, None);
}

#[test]
fn test_set_post_platform() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let platform_id = helpers::add_platform(&manager, "Test Platform".into());
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    manager
        .bind(post_id)
        .update(UpdatePost::default().platform(Some(platform_id)))
        .unwrap();

    let post = helpers::get_post(&manager, post_id);
    assert_eq!(post.platform, Some(platform_id));

    manager
        .bind(post_id)
        .update(UpdatePost::default().platform(None))
        .unwrap();
    let post = helpers::get_post(&manager, post_id);
    assert_eq!(post.platform, None);
}

#[test]
fn test_set_post_published() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    let new_published = Utc::now();
    manager
        .bind(post_id)
        .update(UpdatePost::default().published(new_published))
        .unwrap();

    let post = helpers::get_post(&manager, post_id);
    let diff = (post.published - new_published).num_milliseconds().abs();
    assert!(diff < 1000);
}

#[test]
fn test_set_post_updated() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    let new_updated = Utc::now();
    manager
        .bind(post_id)
        .update(UpdatePost::default().updated(new_updated))
        .unwrap();

    let post = helpers::get_post(&manager, post_id);
    let diff = (post.updated - new_updated).num_milliseconds().abs();
    assert!(diff < 1000);
}

#[test]
fn test_set_post_updated_by_latest() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let old_time = Utc::now() - chrono::Duration::hours(1);
    let new_time = Utc::now();

    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(old_time),
        Some(old_time),
    );

    manager
        .bind(post_id)
        .update(UpdatePost::default().updated_by_latest(new_time))
        .unwrap();

    let post = helpers::get_post(&manager, post_id);
    let diff = (post.updated - new_time).num_milliseconds().abs();
    assert!(diff < 1000, "Should be updated to newer time");

    let even_older = old_time - chrono::Duration::hours(1);
    manager
        .bind(post_id)
        .update(UpdatePost::default().updated_by_latest(even_older))
        .unwrap();

    let post = helpers::get_post(&manager, post_id);
    let diff = (post.updated - new_time).num_milliseconds().abs();
    assert!(diff < 1000, "Should not change to older time");
}

#[test]
fn test_set_post_content() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    let content = vec![
        Content::Text("Hello world!".to_string()),
        Content::Text("This is test content.".to_string()),
    ];

    manager
        .bind(post_id)
        .update(UpdatePost::default().content(content.clone()))
        .unwrap();

    let post = helpers::get_post(&manager, post_id);
    assert_eq!(post.content, content);
}

#[test]
fn test_set_post_comments() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

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
        .bind(post_id)
        .update(UpdatePost::default().comments(comments.clone()))
        .unwrap();

    let post = helpers::get_post(&manager, post_id);
    assert_eq!(post.comments, comments);
}

#[test]
fn test_set_post_thumb() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    let file_meta_id = helpers::add_file_meta(
        &manager,
        post_id,
        "thumbnail.jpg".into(),
        "image/jpeg".into(),
        HashMap::new(),
    );

    manager
        .bind(post_id)
        .update(UpdatePost::default().thumb(Some(file_meta_id)))
        .unwrap();

    let post = helpers::get_post(&manager, post_id);
    assert_eq!(post.thumb, Some(file_meta_id));

    manager
        .bind(post_id)
        .update(UpdatePost::default().thumb(None))
        .unwrap();
    let post = helpers::get_post(&manager, post_id);
    assert_eq!(post.thumb, None);
}

// ── Binded: Author relationships ─────────────────────────────

#[test]
fn test_add_post_authors() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let author1 = helpers::add_author(&manager, "Author 1".into(), Some(now));
    let author2 = helpers::add_author(&manager, "Author 2".into(), Some(now));
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    manager
        .bind(post_id)
        .add_authors(&[author1, author2])
        .unwrap();

    let authors = helpers::list_post_authors(&manager, post_id);
    assert_eq!(authors.len(), 2);
    let ids: Vec<AuthorId> = authors.iter().map(|a| a.id).collect();
    assert!(ids.contains(&author1));
    assert!(ids.contains(&author2));
}

#[test]
fn test_remove_post_authors() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let author1 = helpers::add_author(&manager, "Author 1".into(), Some(now));
    let author2 = helpers::add_author(&manager, "Author 2".into(), Some(now));
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    helpers::add_post_authors(&manager, post_id, &[author1, author2]);

    manager.bind(post_id).remove_authors(&[author1]).unwrap();

    let remaining = helpers::list_post_authors(&manager, post_id);
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, author2);
}

#[test]
fn test_list_post_authors() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let author = helpers::add_author(&manager, "Author".into(), Some(now));
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    helpers::add_post_authors(&manager, post_id, &[author]);

    let ids = manager.bind(post_id).list_authors().unwrap();
    assert_eq!(ids, vec![author]);
}

// ── Binded: Tag relationships ────────────────────────────────

#[test]
fn test_add_post_tags() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let platform = helpers::add_platform(&manager, "Test Platform".into());
    let tag1 = helpers::add_tag(&manager, "tag1".into(), Some(platform));
    let tag2 = helpers::add_tag(&manager, "tag2".into(), Some(platform));
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    manager.bind(post_id).add_tags(&[tag1, tag2]).unwrap();

    let tags = helpers::list_post_tags(&manager, post_id);
    assert_eq!(tags.len(), 2);
    let ids: Vec<TagId> = tags.iter().map(|t| t.id).collect();
    assert!(ids.contains(&tag1));
    assert!(ids.contains(&tag2));
}

#[test]
fn test_remove_post_tags() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let platform = helpers::add_platform(&manager, "Test Platform".into());
    let tag1 = helpers::add_tag(&manager, "tag1".into(), Some(platform));
    let tag2 = helpers::add_tag(&manager, "tag2".into(), Some(platform));
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    helpers::add_post_tags(&manager, post_id, &[tag1, tag2]);

    manager.bind(post_id).remove_tags(&[tag1]).unwrap();

    let remaining = helpers::list_post_tags(&manager, post_id);
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, tag2);
}

#[test]
fn test_list_post_tags() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let tag = helpers::add_tag(&manager, "tag1".into(), None);
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    helpers::add_post_tags(&manager, post_id, &[tag]);

    let ids = manager.bind(post_id).list_tags().unwrap();
    assert_eq!(ids, vec![tag]);
}

// ── Binded: Collection relationships ─────────────────────────

#[test]
fn test_add_post_collections() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let col1 = helpers::add_collection(&manager, "Collection 1".into(), None, None);
    let col2 = helpers::add_collection(&manager, "Collection 2".into(), None, None);
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    manager
        .bind(post_id)
        .add_collections(&[col1, col2])
        .unwrap();

    let cols = helpers::list_post_collections(&manager, post_id);
    assert_eq!(cols.len(), 2);
    let ids: Vec<CollectionId> = cols.iter().map(|c| c.id).collect();
    assert!(ids.contains(&col1));
    assert!(ids.contains(&col2));
}

#[test]
fn test_remove_post_collections() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let col1 = helpers::add_collection(&manager, "Collection 1".into(), None, None);
    let col2 = helpers::add_collection(&manager, "Collection 2".into(), None, None);
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    helpers::add_post_collections(&manager, post_id, &[col1, col2]);

    manager.bind(post_id).remove_collections(&[col1]).unwrap();

    let remaining = helpers::list_post_collections(&manager, post_id);
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, col2);
}

#[test]
fn test_list_post_collections() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let col = helpers::add_collection(&manager, "Col".into(), None, None);
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    helpers::add_post_collections(&manager, post_id, &[col]);

    let ids = manager.bind(post_id).list_collections().unwrap();
    assert_eq!(ids, vec![col]);
}

// ── Binded: Bidirectional relationships ──────────────────────

#[test]
fn test_post_relationships_bidirectional() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    let author_id = helpers::add_author(&manager, "Test Author".into(), Some(now));
    let platform_id = helpers::add_platform(&manager, "Test Platform".into());
    let tag_id = helpers::add_tag(&manager, "test-tag".into(), Some(platform_id));
    let collection_id = helpers::add_collection(&manager, "Test Collection".into(), None, None);
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    // Add relationships via Binded
    manager.bind(post_id).add_authors(&[author_id]).unwrap();
    manager.bind(post_id).add_tags(&[tag_id]).unwrap();
    manager
        .bind(post_id)
        .add_collections(&[collection_id])
        .unwrap();

    // Verify post -> entities
    let post_authors = helpers::list_post_authors(&manager, post_id);
    let post_tags = helpers::list_post_tags(&manager, post_id);
    let post_collections = helpers::list_post_collections(&manager, post_id);

    assert_eq!(post_authors.len(), 1);
    assert_eq!(post_authors[0].id, author_id);
    assert_eq!(post_tags.len(), 1);
    assert_eq!(post_tags[0].id, tag_id);
    assert_eq!(post_collections.len(), 1);
    assert_eq!(post_collections[0].id, collection_id);

    // Verify entities -> post
    let author_posts = helpers::list_author_posts(&manager, author_id);
    let tag_posts = helpers::list_tag_posts(&manager, tag_id);
    let collection_posts = helpers::list_collection_posts(&manager, collection_id);

    assert_eq!(author_posts.len(), 1);
    assert_eq!(author_posts[0].id, post_id);
    assert_eq!(tag_posts.len(), 1);
    assert_eq!(tag_posts[0].id, post_id);
    assert_eq!(collection_posts.len(), 1);
    assert_eq!(collection_posts[0].id, post_id);
}

// ── Content edge cases ───────────────────────────────────────

#[test]
fn test_post_content_with_files() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    let file_meta_id = helpers::add_file_meta(
        &manager,
        post_id,
        "image.jpg".into(),
        "image/jpeg".into(),
        HashMap::new(),
    );

    let content = vec![
        Content::Text("Check out this image:".into()),
        Content::File(file_meta_id),
        Content::Text("What do you think?".into()),
    ];

    manager
        .bind(post_id)
        .update(UpdatePost::default().content(content.clone()))
        .unwrap();

    let post = helpers::get_post(&manager, post_id);
    assert_eq!(post.content, content);
}

#[test]
fn test_empty_post_content_and_comments() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post_id = helpers::add_post(
        &manager,
        "Empty Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    manager
        .bind(post_id)
        .update(UpdatePost::default().content(vec![]))
        .unwrap();
    manager
        .bind(post_id)
        .update(UpdatePost::default().comments(vec![]))
        .unwrap();

    let post = helpers::get_post(&manager, post_id);
    assert_eq!(post.content, vec![]);
    assert_eq!(post.comments, vec![]);
}
