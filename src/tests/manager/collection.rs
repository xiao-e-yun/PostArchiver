//! Collection manager tests
//!
//! Tests for collection CRUD operations, property management,
//! and collection-post relationships.

use crate::{manager::PostArchiverManager, tests::helpers, CollectionId};
use chrono::Utc;
use std::collections::HashMap;

// ── CRUD via helpers ──────────────────────────────────────────

#[test]
fn test_add_collection() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let collection_id = helpers::add_collection(
        &manager,
        "Test Collection".into(),
        Some("test_source".into()),
        None,
    );
    assert!(collection_id.raw() > 0);

    let collection = helpers::get_collection(&manager, collection_id).unwrap();
    assert_eq!(collection.name, "Test Collection");
    assert_eq!(collection.source, Some("test_source".into()));
    assert_eq!(collection.thumb, None);
}

#[test]
fn test_add_collection_with_thumb() {
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

    let collection_id = helpers::add_collection(
        &manager,
        "Collection with Thumb".into(),
        None,
        Some(file_meta_id),
    );

    let collection = helpers::get_collection(&manager, collection_id).unwrap();
    assert_eq!(collection.name, "Collection with Thumb");
    assert_eq!(collection.thumb, Some(file_meta_id));
}

#[test]
fn test_list_collections() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let id1 = helpers::add_collection(
        &manager,
        "Collection 1".into(),
        Some("source1".into()),
        None,
    );
    let id2 = helpers::add_collection(
        &manager,
        "Collection 2".into(),
        Some("source2".into()),
        None,
    );

    let collections = helpers::list_collections(&manager);
    assert_eq!(collections.len(), 2);
    let ids: Vec<CollectionId> = collections.iter().map(|c| c.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

#[test]
fn test_get_collection() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let collection_id = helpers::add_collection(
        &manager,
        "Get Test Collection".into(),
        Some("get_test_source".into()),
        None,
    );

    let collection = helpers::get_collection(&manager, collection_id).unwrap();
    assert_eq!(collection.id, collection_id);
    assert_eq!(collection.name, "Get Test Collection");
    assert_eq!(collection.source, Some("get_test_source".into()));
}

#[test]
fn test_get_nonexistent_collection() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let result = helpers::get_collection(&manager, CollectionId(999));
    assert!(result.is_none());
}

#[test]
fn test_find_collection() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let collection_id = helpers::add_collection(
        &manager,
        "Findable Collection".into(),
        Some("findable_source".into()),
        None,
    );

    let found_id = helpers::find_collection(&manager, "findable_source");
    assert_eq!(found_id, Some(collection_id));

    let not_found = helpers::find_collection(&manager, "nonexistent_source");
    assert_eq!(not_found, None);
}

// ── Binded: Delete ────────────────────────────────────────────

#[test]
fn test_remove_collection() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let collection_id = helpers::add_collection(
        &manager,
        "To Delete".into(),
        Some("delete_source".into()),
        None,
    );

    assert!(helpers::get_collection(&manager, collection_id).is_some());

    manager.bind(collection_id).delete().unwrap();

    assert!(helpers::get_collection(&manager, collection_id).is_none());
}

// ── Binded: Set fields ───────────────────────────────────────

#[test]
fn test_set_collection_name() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let collection_id = helpers::add_collection(&manager, "Original Name".into(), None, None);

    manager
        .bind(collection_id)
        .set_name("Updated Name".into())
        .unwrap();

    let collection = helpers::get_collection(&manager, collection_id).unwrap();
    assert_eq!(collection.name, "Updated Name");
}

#[test]
fn test_set_collection_source() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let collection_id = helpers::add_collection(&manager, "Test Collection".into(), None, None);

    let new_source = Some("new_source".to_string());
    manager
        .bind(collection_id)
        .set_source(new_source.clone())
        .unwrap();

    let collection = helpers::get_collection(&manager, collection_id).unwrap();
    assert_eq!(collection.source, new_source);

    manager.bind(collection_id).set_source(None).unwrap();
    let collection = helpers::get_collection(&manager, collection_id).unwrap();
    assert_eq!(collection.source, None);
}

#[test]
fn test_set_collection_thumb() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let collection_id = helpers::add_collection(&manager, "Test Collection".into(), None, None);
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
        "thumb.jpg".into(),
        "image/jpeg".into(),
        HashMap::new(),
    );

    manager
        .bind(collection_id)
        .set_thumb(Some(file_meta_id))
        .unwrap();

    let collection = helpers::get_collection(&manager, collection_id).unwrap();
    assert_eq!(collection.thumb, Some(file_meta_id));

    manager.bind(collection_id).set_thumb(None).unwrap();
    let collection = helpers::get_collection(&manager, collection_id).unwrap();
    assert_eq!(collection.thumb, None);
}

#[test]
fn test_set_collection_thumb_by_latest() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let collection_id = helpers::add_collection(&manager, "Test Collection".into(), None, None);

    let early_time = chrono::DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let later_time = chrono::DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let latest_time = chrono::DateTime::parse_from_rfc3339("2022-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    let post1 = helpers::add_post(
        &manager,
        "Post 1".into(),
        None,
        None,
        Some(early_time),
        Some(early_time),
    );
    let post2 = helpers::add_post(
        &manager,
        "Post 2".into(),
        None,
        None,
        Some(later_time),
        Some(later_time),
    );
    let post3 = helpers::add_post(
        &manager,
        "Post 3".into(),
        None,
        None,
        Some(latest_time),
        Some(latest_time),
    );

    let _fm1 = helpers::add_file_meta(
        &manager,
        post1,
        "thumb1.jpg".into(),
        "image/jpeg".into(),
        HashMap::new(),
    );
    let _fm2 = helpers::add_file_meta(
        &manager,
        post2,
        "thumb2.jpg".into(),
        "image/jpeg".into(),
        HashMap::new(),
    );
    let fm3 = helpers::add_file_meta(
        &manager,
        post3,
        "thumb3.jpg".into(),
        "image/jpeg".into(),
        HashMap::new(),
    );

    // Set post thumbnails
    manager.bind(post1).set_thumb(Some(_fm1)).unwrap();
    manager.bind(post2).set_thumb(Some(_fm2)).unwrap();
    manager.bind(post3).set_thumb(Some(fm3)).unwrap();

    // Associate posts with collection
    helpers::add_post_collections(&manager, post1, &[collection_id]);
    helpers::add_post_collections(&manager, post2, &[collection_id]);
    helpers::add_post_collections(&manager, post3, &[collection_id]);

    manager.bind(collection_id).set_thumb_by_latest().unwrap();

    let collection = helpers::get_collection(&manager, collection_id).unwrap();
    assert_eq!(collection.thumb, Some(fm3));
}

// ── Binded: Post relationships ───────────────────────────────

#[test]
fn test_list_collection_posts() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let collection_id = helpers::add_collection(&manager, "Test Collection".into(), None, None);
    let post1 = helpers::add_post(&manager, "Post 1".into(), None, None, Some(now), Some(now));
    let post2 = helpers::add_post(&manager, "Post 2".into(), None, None, Some(now), Some(now));

    helpers::add_post_collections(&manager, post1, &[collection_id]);
    helpers::add_post_collections(&manager, post2, &[collection_id]);

    let post_ids = manager.bind(collection_id).list_posts().unwrap();
    assert_eq!(post_ids.len(), 2);
    assert!(post_ids.contains(&post1));
    assert!(post_ids.contains(&post2));
}

#[test]
fn test_add_collection_posts() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let collection_id = helpers::add_collection(&manager, "Test Collection".into(), None, None);
    let post1 = helpers::add_post(&manager, "Post 1".into(), None, None, Some(now), Some(now));
    let post2 = helpers::add_post(&manager, "Post 2".into(), None, None, Some(now), Some(now));

    manager
        .bind(collection_id)
        .add_posts(&[post1, post2])
        .unwrap();

    let posts = helpers::list_collection_posts(&manager, collection_id);
    assert_eq!(posts.len(), 2);
}

#[test]
fn test_remove_collection_posts() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let collection_id = helpers::add_collection(&manager, "Test Collection".into(), None, None);
    let post1 = helpers::add_post(&manager, "Post 1".into(), None, None, Some(now), Some(now));
    let post2 = helpers::add_post(&manager, "Post 2".into(), None, None, Some(now), Some(now));

    helpers::add_post_collections(&manager, post1, &[collection_id]);
    helpers::add_post_collections(&manager, post2, &[collection_id]);

    manager.bind(collection_id).remove_posts(&[post1]).unwrap();

    let remaining = helpers::list_collection_posts(&manager, collection_id);
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, post2);
}

// ── Edge cases ───────────────────────────────────────────────

#[test]
fn test_add_collection_without_source() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let collection_id =
        helpers::add_collection(&manager, "No Source Collection".into(), None, None);

    let collection = helpers::get_collection(&manager, collection_id).unwrap();
    assert_eq!(collection.name, "No Source Collection");
    assert_eq!(collection.source, None);
    assert_eq!(collection.thumb, None);
}

#[test]
fn test_find_collection_with_none_source() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let _collection_id =
        helpers::add_collection(&manager, "No Source Collection".into(), None, None);

    let result = helpers::find_collection(&manager, "");
    assert_eq!(result, None);
}

#[test]
fn test_list_empty_collections() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let collections = helpers::list_collections(&manager);
    assert_eq!(collections.len(), 0);
}

#[test]
fn test_collection_id_consistency() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let collection_id = helpers::add_collection(
        &manager,
        "Consistency Test".into(),
        Some("consistent".into()),
        None,
    );

    let by_id = helpers::get_collection(&manager, collection_id).unwrap();
    let found_id = helpers::find_collection(&manager, "consistent").unwrap();
    let by_source = helpers::get_collection(&manager, found_id).unwrap();

    assert_eq!(by_id.id, by_source.id);
    assert_eq!(by_id.name, by_source.name);
    assert_eq!(by_id.source, by_source.source);
}
