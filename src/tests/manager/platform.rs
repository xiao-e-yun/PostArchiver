//! Platform manager tests
//!
//! Tests for platform CRUD operations
//! and platform-tag/post relationships.

use crate::{
    manager::{PostArchiverManager, UpdatePlatform},
    tests::helpers,
    PlatformId,
};
use chrono::Utc;

// ── CRUD via helpers ──────────────────────────────────────────

#[test]
fn test_add_platform() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".into());
    assert!(platform_id.raw() > 0);

    let platform = helpers::get_platform(&manager, platform_id);
    assert_eq!(platform.name, "Test Platform");
    assert_eq!(platform.id, platform_id);
}

#[test]
fn test_list_platforms() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let id1 = helpers::add_platform(&manager, "Platform 1".into());
    let id2 = helpers::add_platform(&manager, "Platform 2".into());

    let platforms = helpers::list_platforms(&manager);
    // The default "unknown" platform is ID 1
    let ids: Vec<PlatformId> = platforms.iter().map(|p| p.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

#[test]
fn test_get_platform() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Get Test Platform".into());

    let platform = helpers::get_platform(&manager, platform_id);
    assert_eq!(platform.id, platform_id);
    assert_eq!(platform.name, "Get Test Platform");
}

#[test]
fn test_find_platform() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Find Test Platform".into());

    let found_id = helpers::find_platform(&manager, "Find Test Platform");
    assert_eq!(found_id, Some(platform_id));

    let not_found = helpers::find_platform(&manager, "Non-existent Platform");
    assert_eq!(not_found, None);
}

// ── Binded: Delete ────────────────────────────────────────────

#[test]
fn test_remove_platform() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "To Delete".into());

    let _ = helpers::get_platform(&manager, platform_id);
    manager.bind(platform_id).delete().unwrap();

    let platforms = helpers::list_platforms(&manager);
    assert!(platforms.iter().all(|p| p.id != platform_id));
}

// ── Binded: Set fields ───────────────────────────────────────

#[test]
fn test_set_platform_name() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Original Name".into());

    manager
        .bind(platform_id)
        .update(UpdatePlatform::default().name("Updated Name".into()))
        .unwrap();

    let platform = helpers::get_platform(&manager, platform_id);
    assert_eq!(platform.name, "Updated Name");
}

// ── Binded: Relations ────────────────────────────────────────

#[test]
fn test_list_platform_tags() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".into());
    let tag1 = helpers::add_tag(&manager, "tag1".into(), Some(platform_id));
    let tag2 = helpers::add_tag(&manager, "tag2".into(), Some(platform_id));
    let _tag3 = helpers::add_tag(&manager, "tag3".into(), None);

    let platform_tags = manager.bind(platform_id).list_tags().unwrap();
    assert_eq!(platform_tags.len(), 2);
    assert!(platform_tags.contains(&tag1));
    assert!(platform_tags.contains(&tag2));
}

#[test]
fn test_list_platform_posts() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let platform_id = helpers::add_platform(&manager, "Test Platform".into());
    let post1 = helpers::add_post(
        &manager,
        "Post 1".into(),
        None,
        Some(platform_id),
        Some(now),
        Some(now),
    );
    let post2 = helpers::add_post(
        &manager,
        "Post 2".into(),
        None,
        Some(platform_id),
        Some(now),
        Some(now),
    );
    let _post3 = helpers::add_post(&manager, "Post 3".into(), None, None, Some(now), Some(now));

    let platform_posts = manager.bind(platform_id).list_posts().unwrap();
    assert_eq!(platform_posts.len(), 2);
    assert!(platform_posts.contains(&post1));
    assert!(platform_posts.contains(&post2));
}

// ── Edge cases ───────────────────────────────────────────────

#[test]
fn test_empty_platform_relationships() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Empty Platform".into());

    let tags = manager.bind(platform_id).list_tags().unwrap();
    assert_eq!(tags.len(), 0);

    let posts = manager.bind(platform_id).list_posts().unwrap();
    assert_eq!(posts.len(), 0);
}

#[test]
fn test_duplicate_platform_name() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let _id1 = helpers::add_platform(&manager, "Duplicate Platform".into());

    // Try to add duplicate - should panic (unwrap in helper) due to UNIQUE constraint
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        helpers::add_platform(&manager, "Duplicate Platform".into());
    }));
    assert!(result.is_err(), "Should fail to add duplicate platform");
}
