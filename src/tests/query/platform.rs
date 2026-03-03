//! Tests for `src/query/platform.rs`

use crate::{
    manager::PostArchiverManager,
    query::{platform::PlatformSort, Query, SortDir, Sortable},
    tests::helpers,
    Platform, Post, Tag,
};
use chrono::Utc;

// ── get_platform ──────────────────────────────────────────────────────────────

#[test]
fn test_get_platform_exists() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let id = helpers::add_platform(&m, "github".into());

    let plt = m.get_platform(id).unwrap().unwrap();
    assert_eq!(plt.id, id);
    assert_eq!(plt.name, "github");
}

#[test]
fn test_get_platform_not_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    use crate::PlatformId;
    let result = m.get_platform(PlatformId::from(9999u32)).unwrap();
    assert!(result.is_none());
}

// ── find_platform ─────────────────────────────────────────────────────────────

#[test]
fn test_find_platform_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let id = helpers::add_platform(&m, "twitter".into());

    let found = m.find_platform("twitter").unwrap();
    assert_eq!(found, Some(id));
}

#[test]
fn test_find_platform_not_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let result = m.find_platform("nonexistent").unwrap();
    assert!(result.is_none());
}

// ── platforms().query() ───────────────────────────────────────────────────────

#[test]
fn test_platforms_empty() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let platforms = m.platforms().query::<Platform>().unwrap();
    // may include the built-in "unknown" platform (id=1 / id=0)
    // but user-added ones should be absent
    assert!(!platforms.iter().any(|p| p.name == "userPlatform"));
}

#[test]
fn test_platforms_returns_added() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let id1 = helpers::add_platform(&m, "github".into());
    let id2 = helpers::add_platform(&m, "twitter".into());

    let platforms = m.platforms().query::<Platform>().unwrap();
    let ids: Vec<_> = platforms.iter().map(|p| p.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

#[test]
fn test_platforms_sorted_by_name() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    helpers::add_platform(&m, "zzz".into());
    helpers::add_platform(&m, "aaa".into());
    helpers::add_platform(&m, "mmm".into());

    let platforms = m
        .platforms()
        .sort(PlatformSort::Name, SortDir::Asc)
        .query::<Platform>()
        .unwrap();
    // filter user-added (exclude the built-in "unknown")
    let user: Vec<_> = platforms
        .iter()
        .filter(|p| p.name == "aaa" || p.name == "mmm" || p.name == "zzz")
        .collect();
    assert_eq!(user.len(), 3);
    assert_eq!(user[0].name, "aaa");
    assert_eq!(user[1].name, "mmm");
    assert_eq!(user[2].name, "zzz");
}
// ── platform posts / tags via builders ───────────────────────────────────────

#[test]
fn test_platform_posts_via_builder() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let plt = helpers::add_platform(&m, "p".into());
    let id1 = helpers::add_post(&m, "A".into(), None, Some(plt), Some(now), Some(now));
    let id2 = helpers::add_post(&m, "B".into(), None, Some(plt), Some(now), Some(now));
    helpers::add_post(&m, "C".into(), None, None, Some(now), Some(now)); // different platform

    let mut q = m.posts();
    q.platforms.insert(plt);
    let posts = q.query::<Post>().unwrap();
    assert_eq!(posts.len(), 2);
    let ids: Vec<_> = posts.iter().map(|p| p.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

#[test]
fn test_platform_posts_empty_via_builder() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let plt = helpers::add_platform(&m, "empty".into());

    let mut q = m.posts();
    q.platforms.insert(plt);
    let posts = q.query::<Post>().unwrap();
    assert!(posts.is_empty());
}

#[test]
fn test_platform_tags_via_builder() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let plt = helpers::add_platform(&m, "p".into());
    let t1 = helpers::add_tag(&m, "tag-one".into(), Some(plt));
    let t2 = helpers::add_tag(&m, "tag-two".into(), Some(plt));
    helpers::add_tag(&m, "global".into(), None); // no platform

    let mut q = m.tags();
    q.platforms.insert(plt);
    let tags = q.query::<Tag>().unwrap();
    assert_eq!(tags.len(), 2);
    let ids: Vec<_> = tags.iter().map(|t| t.id).collect();
    assert!(ids.contains(&t1));
    assert!(ids.contains(&t2));
}

#[test]
fn test_platform_tags_empty_via_builder() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let plt = helpers::add_platform(&m, "empty".into());

    let mut q = m.tags();
    q.platforms.insert(plt);
    let tags = q.query::<Tag>().unwrap();
    assert!(tags.is_empty());
}

#[test]
fn test_platform_tags_isolates_correctly() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let plt_a = helpers::add_platform(&m, "A".into());
    let plt_b = helpers::add_platform(&m, "B".into());
    helpers::add_tag(&m, "ta".into(), Some(plt_a));
    let tb = helpers::add_tag(&m, "tb".into(), Some(plt_b));

    let mut q = m.tags();
    q.platforms.insert(plt_b);
    let tags = q.query::<Tag>().unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].id, tb);
}
