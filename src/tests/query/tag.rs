//! Tests for `src/query/tag.rs`

use crate::{manager::PostArchiverManager, query::SortDir, tests::helpers};
use chrono::Utc;

// ── get_tag ───────────────────────────────────────────────────────────────────

#[test]
fn test_get_tag_exists() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let id = helpers::add_tag(&m, "rust".into(), None);

    let tag = m.get_tag(id).unwrap().unwrap();
    assert_eq!(tag.id, id);
    assert_eq!(tag.name, "rust");
    assert_eq!(tag.platform, None);
}

#[test]
fn test_get_tag_with_platform() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let plt = helpers::add_platform(&m, "github".into());
    let id = helpers::add_tag(&m, "lang:rust".into(), Some(plt));

    let tag = m.get_tag(id).unwrap().unwrap();
    assert_eq!(tag.platform, Some(plt));
}

#[test]
fn test_get_tag_not_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    use crate::TagId;
    let result = m.get_tag(TagId::from(9999u32)).unwrap();
    assert!(result.is_none());
}

// ── find_tag ──────────────────────────────────────────────────────────────────

#[test]
fn test_find_tag_no_platform() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let id = helpers::add_tag(&m, "open-source".into(), None);

    let found = m.find_tag("open-source", None).unwrap();
    assert_eq!(found, Some(id));
}

#[test]
fn test_find_tag_with_platform() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let plt = helpers::add_platform(&m, "gh".into());
    let id = helpers::add_tag(&m, "stars".into(), Some(plt));

    let found = m.find_tag("stars", Some(plt)).unwrap();
    assert_eq!(found, Some(id));
}

#[test]
fn test_find_tag_platform_mismatch() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let _id = helpers::add_tag(&m, "shared".into(), None);

    // Same name but searching with a platform → different row (platform IS ?)
    let plt = helpers::add_platform(&m, "p".into());
    let result = m.find_tag("shared", Some(plt)).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_find_tag_not_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let result = m.find_tag("nonexistent", None).unwrap();
    assert!(result.is_none());
}

// ── tags().query() – basic list ──────────────────────────────────────────────

#[test]
fn test_tags_empty() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let tags = m.tags().query().unwrap();
    assert!(tags.is_empty());
}

#[test]
fn test_tags_returns_all() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let id1 = helpers::add_tag(&m, "alpha".into(), None);
    let id2 = helpers::add_tag(&m, "beta".into(), None);

    let tags = m.tags().query().unwrap();
    assert_eq!(tags.len(), 2);
    let ids: Vec<_> = tags.iter().map(|t| t.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

// ── tags().platform() filter ─────────────────────────────────────────────────

#[test]
fn test_tags_filter_by_platform() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let plt_a = helpers::add_platform(&m, "pA".into());
    let plt_b = helpers::add_platform(&m, "pB".into());

    let id1 = helpers::add_tag(&m, "tagA".into(), Some(plt_a));
    let _id2 = helpers::add_tag(&m, "tagB".into(), Some(plt_b));
    let _id3 = helpers::add_tag(&m, "tagNone".into(), None);

    let tags = m.tags().platform(Some(plt_a)).query().unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].id, id1);
}

#[test]
fn test_tags_filter_by_no_platform() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let plt = helpers::add_platform(&m, "p".into());

    let _id1 = helpers::add_tag(&m, "withPlt".into(), Some(plt));
    let id2 = helpers::add_tag(&m, "noPlt".into(), None);

    let tags = m.tags().platform(None).query().unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].id, id2);
}

#[test]
fn test_tags_filter_by_multiple_platforms_or() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let plt_a = helpers::add_platform(&m, "pA".into());
    let plt_b = helpers::add_platform(&m, "pB".into());
    let plt_c = helpers::add_platform(&m, "pC".into());

    let id1 = helpers::add_tag(&m, "ta".into(), Some(plt_a));
    let id2 = helpers::add_tag(&m, "tb".into(), Some(plt_b));
    let _id3 = helpers::add_tag(&m, "tc".into(), Some(plt_c));

    let tags = m
        .tags()
        .platform(Some(plt_a))
        .platform(Some(plt_b))
        .query()
        .unwrap();
    assert_eq!(tags.len(), 2);
    let ids: Vec<_> = tags.iter().map(|t| t.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

// ── tags().name_contains() ───────────────────────────────────────────────────

#[test]
fn test_tags_name_contains() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    helpers::add_tag(&m, "rust-lang".into(), None);
    helpers::add_tag(&m, "rust-async".into(), None);
    helpers::add_tag(&m, "python".into(), None);

    let tags = m.tags().name_contains("rust").query().unwrap();
    assert_eq!(tags.len(), 2);
    assert!(tags.iter().all(|t| t.name.contains("rust")));
}

// ── tags().sort_dir() ────────────────────────────────────────────────────────

#[test]
fn test_tags_sort_by_name_asc() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    helpers::add_tag(&m, "zebra".into(), None);
    helpers::add_tag(&m, "apple".into(), None);
    helpers::add_tag(&m, "mango".into(), None);

    let tags = m.tags().sort_dir(SortDir::Asc).query().unwrap();
    let names: Vec<_> = tags.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, vec!["apple", "mango", "zebra"]);
}

#[test]
fn test_tags_sort_by_name_desc() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    helpers::add_tag(&m, "zebra".into(), None);
    helpers::add_tag(&m, "apple".into(), None);
    helpers::add_tag(&m, "mango".into(), None);

    let tags = m.tags().sort_dir(SortDir::Desc).query().unwrap();
    let names: Vec<_> = tags.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, vec!["zebra", "mango", "apple"]);
}

// ── tags().pagination() ──────────────────────────────────────────────────────

#[test]
fn test_tags_pagination() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    for i in 0..5u32 {
        helpers::add_tag(&m, format!("tag-{i:02}"), None);
    }

    let page1 = m
        .tags()
        .sort_dir(SortDir::Asc)
        .pagination(2, 0)
        .query()
        .unwrap();
    let page2 = m
        .tags()
        .sort_dir(SortDir::Asc)
        .pagination(2, 1)
        .query()
        .unwrap();

    assert_eq!(page1.len(), 2);
    assert_eq!(page2.len(), 2);

    let ids1: Vec<_> = page1.iter().map(|t| t.id).collect();
    let ids2: Vec<_> = page2.iter().map(|t| t.id).collect();
    assert!(ids1.iter().all(|id| !ids2.contains(id)));
}

// ── tags().with_total() ──────────────────────────────────────────────────────

#[test]
fn test_tags_with_total() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    for i in 0..5u32 {
        helpers::add_tag(&m, format!("t{i}"), None);
    }

    let result = m.tags().pagination(2, 0).with_total().query().unwrap();
    assert_eq!(result.total, 5);
    assert_eq!(result.items.len(), 2);
}

// ── tag posts via posts() builder ────────────────────────────────────────────

#[test]
fn test_tag_posts_via_builder() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let t = helpers::add_tag(&m, "featured".into(), None);
    let id1 = helpers::add_post(&m, "P1".into(), None, None, Some(now), Some(now));
    let id2 = helpers::add_post(&m, "P2".into(), None, None, Some(now), Some(now));
    let _id3 = helpers::add_post(&m, "P3".into(), None, None, Some(now), Some(now));
    helpers::add_post_tags(&m, id1, &[t]);
    helpers::add_post_tags(&m, id2, &[t]);

    let posts = m.posts().tags([t]).query().unwrap();
    assert_eq!(posts.len(), 2);
    let ids: Vec<_> = posts.iter().map(|p| p.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

#[test]
fn test_tag_posts_empty_via_builder() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let t = helpers::add_tag(&m, "unused".into(), None);

    let posts = m.posts().tags([t]).query().unwrap();
    assert!(posts.is_empty());
}
