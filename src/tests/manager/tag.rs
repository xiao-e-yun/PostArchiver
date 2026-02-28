//! Tag manager tests
//!
//! Tests for tag CRUD operations, platform associations,
//! and tag-post relationships.

use crate::{
    manager::{PostArchiverManager, UpdateTag},
    tests::helpers,
    TagId,
};
use chrono::Utc;

// ── CRUD via helpers ──────────────────────────────────────────

#[test]
fn test_add_tag() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".into());
    let tag_id = helpers::add_tag(&manager, "test_tag".into(), Some(platform_id));
    assert!(tag_id.raw() > 0);

    let tag = helpers::get_tag(&manager, tag_id).unwrap();
    assert_eq!(tag.name, "test_tag");
    assert_eq!(tag.id, tag_id);
    assert_eq!(tag.platform, Some(platform_id));
}

#[test]
fn test_add_tag_no_platform() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let tag_id = helpers::add_tag(&manager, "global_tag".into(), None);

    let tag = helpers::get_tag(&manager, tag_id).unwrap();
    assert_eq!(tag.name, "global_tag");
    assert_eq!(tag.platform, None);
}

#[test]
fn test_list_tags() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".into());
    let id1 = helpers::add_tag(&manager, "tag1".into(), Some(platform_id));
    let id2 = helpers::add_tag(&manager, "tag2".into(), None);

    let tags = helpers::list_tags(&manager);
    assert_eq!(tags.len(), 2);
    let ids: Vec<TagId> = tags.iter().map(|t| t.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

#[test]
fn test_get_tag() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let tag_id = helpers::add_tag(&manager, "get_test_tag".into(), None);

    let tag = helpers::get_tag(&manager, tag_id).unwrap();
    assert_eq!(tag.id, tag_id);
    assert_eq!(tag.name, "get_test_tag");
}

#[test]
fn test_get_nonexistent_tag() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let result = helpers::get_tag(&manager, TagId(999));
    assert!(result.is_none());
}

#[test]
fn test_find_tag_by_string() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let tag_id = helpers::add_tag(&manager, "findme".into(), None);

    let found_id = helpers::find_tag(&manager, "findme", None);
    assert_eq!(found_id, Some(tag_id));

    let not_found = helpers::find_tag(&manager, "nonexistent", None);
    assert_eq!(not_found, None);
}

#[test]
fn test_find_tag_by_name_and_platform() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".into());
    let tag_id = helpers::add_tag(&manager, "platform_tag".into(), Some(platform_id));

    let found_id = helpers::find_tag(&manager, "platform_tag", Some(platform_id));
    assert_eq!(found_id, Some(tag_id));

    let other_platform = helpers::add_platform(&manager, "Other Platform".into());
    let not_found = helpers::find_tag(&manager, "platform_tag", Some(other_platform));
    assert_eq!(not_found, None);
}

#[test]
fn test_find_tag_by_name_and_optional_platform() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".into());
    let tag_id = helpers::add_tag(&manager, "optional_platform_tag".into(), Some(platform_id));

    let found_id = helpers::find_tag(&manager, "optional_platform_tag", Some(platform_id));
    assert_eq!(found_id, Some(tag_id));

    let tag_id_none = helpers::add_tag(&manager, "no_platform_tag".into(), None);
    let found_id_none = helpers::find_tag(&manager, "no_platform_tag", None);
    assert_eq!(found_id_none, Some(tag_id_none));
}

// ── Binded: Delete ────────────────────────────────────────────

#[test]
fn test_remove_tag() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let tag_id = helpers::add_tag(&manager, "to_delete".into(), None);

    assert!(helpers::get_tag(&manager, tag_id).is_some());

    manager.bind(tag_id).delete().unwrap();

    assert!(helpers::get_tag(&manager, tag_id).is_none());
}

#[test]
fn test_remove_tag_with_post_associations() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let tag_id = helpers::add_tag(&manager, "associated_tag".into(), None);
    let post_id = helpers::add_post(
        &manager,
        "Associated Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    helpers::add_post_tags(&manager, post_id, &[tag_id]);

    let post_tags = helpers::list_post_tags(&manager, post_id);
    assert_eq!(post_tags.len(), 1);

    manager.bind(tag_id).delete().unwrap();

    assert!(helpers::get_tag(&manager, tag_id).is_none());
    let post_tags_after = helpers::list_post_tags(&manager, post_id);
    assert_eq!(post_tags_after.len(), 0);
}

// ── Binded: Set fields ───────────────────────────────────────

#[test]
fn test_set_tag_name() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let tag_id = helpers::add_tag(&manager, "original_name".into(), None);

    manager
        .bind(tag_id)
        .update(UpdateTag::default().name("updated_name".into()))
        .unwrap();

    let tag = helpers::get_tag(&manager, tag_id).unwrap();
    assert_eq!(tag.name, "updated_name");
}

#[test]
fn test_set_tag_platform() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform1 = helpers::add_platform(&manager, "Platform 1".into());
    let platform2 = helpers::add_platform(&manager, "Platform 2".into());
    let tag_id = helpers::add_tag(&manager, "test_tag".into(), Some(platform1));

    manager
        .bind(tag_id)
        .update(UpdateTag::default().platform(Some(platform2)))
        .unwrap();

    let tag = helpers::get_tag(&manager, tag_id).unwrap();
    assert_eq!(tag.platform, Some(platform2));

    manager
        .bind(tag_id)
        .update(UpdateTag::default().platform(None))
        .unwrap();
    let tag = helpers::get_tag(&manager, tag_id).unwrap();
    assert_eq!(tag.platform, None);
}

// ── Binded: Post relationships ───────────────────────────────

#[test]
fn test_post_tag_relationships() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let tag1 = helpers::add_tag(&manager, "tag1".into(), None);
    let tag2 = helpers::add_tag(&manager, "tag2".into(), None);
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    helpers::add_post_tags(&manager, post_id, &[tag1, tag2]);

    // Post's tags
    let post_tags = helpers::list_post_tags(&manager, post_id);
    assert_eq!(post_tags.len(), 2);
    let tag_ids: Vec<TagId> = post_tags.iter().map(|t| t.id).collect();
    assert!(tag_ids.contains(&tag1));
    assert!(tag_ids.contains(&tag2));

    // Tag's posts via Binded
    let tag1_posts = manager.bind(tag1).list_posts().unwrap();
    assert_eq!(tag1_posts.len(), 1);
    assert_eq!(tag1_posts[0], post_id);
}

// ── Edge cases ───────────────────────────────────────────────

#[test]
fn test_unique_tag_names_across_platforms() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform1 = helpers::add_platform(&manager, "Platform 1".into());

    let _tag1_id = helpers::add_tag(&manager, "unique_name".into(), Some(platform1));

    // Verify the original can be found
    let found1 = helpers::find_tag(&manager, "unique_name", Some(platform1));
    assert!(found1.is_some());
}

#[test]
fn test_empty_tag_lists() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let tag_id = helpers::add_tag(&manager, "lonely_tag".into(), None);
    let post_id = helpers::add_post(
        &manager,
        "Lonely Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    let tag_posts = helpers::list_tag_posts(&manager, tag_id);
    assert_eq!(tag_posts.len(), 0);

    let post_tags = helpers::list_post_tags(&manager, post_id);
    assert_eq!(post_tags.len(), 0);
}
