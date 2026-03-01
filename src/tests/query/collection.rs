//! Tests for `src/query/collection.rs`

use crate::{manager::PostArchiverManager, query::SortDir, tests::helpers};
use chrono::Utc;

// ── get_collection ────────────────────────────────────────────────────────────

#[test]
fn test_get_collection_exists() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let id = helpers::add_collection(&m, "My Series".into(), Some("src:1".into()), None);

    let col = m.get_collection(id).unwrap().unwrap();
    assert_eq!(col.id, id);
    assert_eq!(col.name, "My Series");
    assert_eq!(col.source, Some("src:1".into()));
}

#[test]
fn test_get_collection_not_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    use crate::CollectionId;
    let result = m.get_collection(CollectionId::from(9999u32)).unwrap();
    assert!(result.is_none());
}

// ── find_collection_by_source ─────────────────────────────────────────────────

#[test]
fn test_find_collection_by_source_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let id = helpers::add_collection(&m, "Series".into(), Some("unique-src".into()), None);

    let found = m.find_collection_by_source("unique-src").unwrap();
    assert_eq!(found, Some(id));
}

#[test]
fn test_find_collection_by_source_not_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let result = m.find_collection_by_source("does-not-exist").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_find_collection_no_source() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    helpers::add_collection(&m, "No source".into(), None, None);

    // NULL source → cannot be found by find_collection_by_source with a string
    let result = m.find_collection_by_source("anything").unwrap();
    assert!(result.is_none());
}

// ── collections().query() – basic list ───────────────────────────────────────

#[test]
fn test_collections_empty() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let cols = m.collections().query().unwrap();
    assert!(cols.is_empty());
}

#[test]
fn test_collections_returns_all() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let id1 = helpers::add_collection(&m, "A".into(), None, None);
    let id2 = helpers::add_collection(&m, "B".into(), None, None);

    let cols = m.collections().query().unwrap();
    assert_eq!(cols.len(), 2);
    let ids: Vec<_> = cols.iter().map(|c| c.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

// ── collections().name_contains() ────────────────────────────────────────────

#[test]
fn test_collections_name_contains() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    helpers::add_collection(&m, "Rust Series".into(), None, None);
    helpers::add_collection(&m, "Rust Tutorials".into(), None, None);
    helpers::add_collection(&m, "Python Book".into(), None, None);

    let cols = m.collections().name_contains("Rust").query().unwrap();
    assert_eq!(cols.len(), 2);
    assert!(cols.iter().all(|c| c.name.contains("Rust")));
}

#[test]
fn test_collections_name_contains_no_match() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    helpers::add_collection(&m, "Something".into(), None, None);

    let cols = m.collections().name_contains("xyz").query().unwrap();
    assert!(cols.is_empty());
}

// ── collections().sort_dir() ─────────────────────────────────────────────────

#[test]
fn test_collections_sort_asc() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    helpers::add_collection(&m, "Zebra".into(), None, None);
    helpers::add_collection(&m, "Apple".into(), None, None);
    helpers::add_collection(&m, "Mango".into(), None, None);

    let cols = m.collections().sort_dir(SortDir::Asc).query().unwrap();
    let names: Vec<_> = cols.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, vec!["Apple", "Mango", "Zebra"]);
}

#[test]
fn test_collections_sort_desc() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    helpers::add_collection(&m, "Zebra".into(), None, None);
    helpers::add_collection(&m, "Apple".into(), None, None);
    helpers::add_collection(&m, "Mango".into(), None, None);

    let cols = m.collections().sort_dir(SortDir::Desc).query().unwrap();
    let names: Vec<_> = cols.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, vec!["Zebra", "Mango", "Apple"]);
}

// ── collections().pagination() ───────────────────────────────────────────────

#[test]
fn test_collections_pagination() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    for i in 0..5u32 {
        helpers::add_collection(&m, format!("Col-{i:02}"), None, None);
    }

    let page1 = m
        .collections()
        .sort_dir(SortDir::Asc)
        .pagination(2, 0)
        .query()
        .unwrap();
    let page2 = m
        .collections()
        .sort_dir(SortDir::Asc)
        .pagination(2, 1)
        .query()
        .unwrap();

    assert_eq!(page1.len(), 2);
    assert_eq!(page2.len(), 2);

    let ids1: Vec<_> = page1.iter().map(|c| c.id).collect();
    let ids2: Vec<_> = page2.iter().map(|c| c.id).collect();
    assert!(ids1.iter().all(|id| !ids2.contains(id)));
}

// ── collections().with_total() ───────────────────────────────────────────────

#[test]
fn test_collections_with_total() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    for i in 0..5u32 {
        helpers::add_collection(&m, format!("C{i}"), None, None);
    }

    let result = m
        .collections()
        .pagination(2, 0)
        .with_total()
        .query()
        .unwrap();
    assert_eq!(result.total, 5);
    assert_eq!(result.items.len(), 2);
}

#[test]
fn test_collections_with_total_filtered() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    helpers::add_collection(&m, "Rust vol.1".into(), None, None);
    helpers::add_collection(&m, "Rust vol.2".into(), None, None);
    helpers::add_collection(&m, "Python vol.1".into(), None, None);

    let result = m
        .collections()
        .name_contains("Rust")
        .with_total()
        .query()
        .unwrap();
    assert_eq!(result.total, 2);
}

// ── collection posts via posts() builder ─────────────────────────────────────

#[test]
fn test_collection_posts_via_builder() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let col = helpers::add_collection(&m, "Series".into(), Some("s".into()), None);
    let id1 = helpers::add_post(&m, "P1".into(), None, None, Some(now), Some(now));
    let id2 = helpers::add_post(&m, "P2".into(), None, None, Some(now), Some(now));
    let _id3 = helpers::add_post(&m, "P3-other".into(), None, None, Some(now), Some(now));
    helpers::add_post_collections(&m, id1, &[col]);
    helpers::add_post_collections(&m, id2, &[col]);

    let posts = m.posts().collection(col).query().unwrap();
    assert_eq!(posts.len(), 2);
    let ids: Vec<_> = posts.iter().map(|p| p.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

#[test]
fn test_collection_posts_empty_via_builder() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let col = helpers::add_collection(&m, "Empty".into(), None, None);

    let posts = m.posts().collection(col).query().unwrap();
    assert!(posts.is_empty());
}

#[test]
fn test_collection_posts_multiple_via_builder() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let col_a = helpers::add_collection(&m, "A".into(), Some("a".into()), None);
    let col_b = helpers::add_collection(&m, "B".into(), Some("b".into()), None);
    let id1 = helpers::add_post(&m, "in A".into(), None, None, Some(now), Some(now));
    let id2 = helpers::add_post(&m, "in B".into(), None, None, Some(now), Some(now));
    let id3 = helpers::add_post(&m, "in both".into(), None, None, Some(now), Some(now));
    helpers::add_post_collections(&m, id1, &[col_a]);
    helpers::add_post_collections(&m, id2, &[col_b]);
    helpers::add_post_collections(&m, id3, &[col_a, col_b]);

    let posts_a = m.posts().collection(col_a).query().unwrap();
    assert_eq!(posts_a.len(), 2);
    let posts_b = m.posts().collection(col_b).query().unwrap();
    assert_eq!(posts_b.len(), 2);
    let ids_a: Vec<_> = posts_a.iter().map(|p| p.id).collect();
    assert!(ids_a.contains(&id1));
    assert!(ids_a.contains(&id3));
}
