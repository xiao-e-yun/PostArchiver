//! Tests for `src/query/post.rs`

use crate::{
    manager::PostArchiverManager,
    query::{post::PostSort, Countable, Paginate, Query, SortDir, Sortable},
    tests::helpers,
};
use chrono::{Duration, Utc};

// ── get_post ──────────────────────────────────────────────────────────────────

#[test]
fn test_get_post_exists() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let id = helpers::add_post(&m, "Hello".into(), None, None, Some(now), Some(now));

    let post = m.get_post(id).unwrap().unwrap();
    assert_eq!(post.id, id);
    assert_eq!(post.title, "Hello");
}

#[test]
fn test_get_post_not_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    use crate::PostId;
    let result = m.get_post(PostId::from(9999u32)).unwrap();
    assert!(result.is_none());
}

// ── find_post_by_source ───────────────────────────────────────────────────────

#[test]
fn test_find_post_by_source_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let id = helpers::add_post(
        &m,
        "Source Post".into(),
        Some("https://example.com/1".into()),
        None,
        Some(now),
        Some(now),
    );

    let found = m.find_post_by_source("https://example.com/1").unwrap();
    assert_eq!(found, Some(id));
}

#[test]
fn test_find_post_by_source_not_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let result = m
        .find_post_by_source("https://nonexistent.example")
        .unwrap();
    assert!(result.is_none());
}

// ── posts().query() – basic list ─────────────────────────────────────────────

#[test]
fn test_posts_empty() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let posts = m.posts().query().unwrap();
    assert!(posts.is_empty());
}

#[test]
fn test_posts_returns_all() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let id1 = helpers::add_post(&m, "A".into(), None, None, Some(now), Some(now));
    let id2 = helpers::add_post(&m, "B".into(), None, None, Some(now), Some(now));
    let id3 = helpers::add_post(&m, "C".into(), None, None, Some(now), Some(now));

    let posts = m.posts().query().unwrap();
    assert_eq!(posts.len(), 3);
    let ids: Vec<_> = posts.iter().map(|p| p.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
    assert!(ids.contains(&id3));
}

// ── posts().platforms filter ──────────────────────────────────────────────────

#[test]
fn test_posts_filter_by_platform() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let p1 = helpers::add_platform(&m, "platformA".into());
    let p2 = helpers::add_platform(&m, "platformB".into());

    let id1 = helpers::add_post(&m, "P1 post".into(), None, Some(p1), Some(now), Some(now));
    let _id2 = helpers::add_post(&m, "P2 post".into(), None, Some(p2), Some(now), Some(now));

    let mut q = m.posts();
    q.platforms.insert(p1);
    let posts = q.query().unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].id, id1);
}

#[test]
fn test_posts_filter_by_multiple_platforms_or() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let p1 = helpers::add_platform(&m, "plt1".into());
    let p2 = helpers::add_platform(&m, "plt2".into());
    let p3 = helpers::add_platform(&m, "plt3".into());

    let id1 = helpers::add_post(&m, "A".into(), None, Some(p1), Some(now), Some(now));
    let id2 = helpers::add_post(&m, "B".into(), None, Some(p2), Some(now), Some(now));
    let _id3 = helpers::add_post(&m, "C".into(), None, Some(p3), Some(now), Some(now));

    let mut q = m.posts();
    q.platforms.extend([p1, p2]);
    let posts = q.query().unwrap();
    assert_eq!(posts.len(), 2);
    let ids: Vec<_> = posts.iter().map(|p| p.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

// ── posts().tags filter – AND semantics ──────────────────────────────────────

#[test]
fn test_posts_filter_by_single_tag() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let tag_a = helpers::add_tag(&m, "rust".into(), None);
    let _tag_b = helpers::add_tag(&m, "python".into(), None);

    let id1 = helpers::add_post(&m, "Rust post".into(), None, None, Some(now), Some(now));
    let id2 = helpers::add_post(&m, "Python post".into(), None, None, Some(now), Some(now));
    helpers::add_post_tags(&m, id1, &[tag_a]);
    helpers::add_post_tags(&m, id2, &[_tag_b]);

    let mut q = m.posts();
    q.tags.insert(tag_a);
    let posts = q.query().unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].id, id1);
}

#[test]
fn test_posts_filter_by_tags_and_semantics() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let tag_a = helpers::add_tag(&m, "rust".into(), None);
    let tag_b = helpers::add_tag(&m, "async".into(), None);
    let tag_c = helpers::add_tag(&m, "python".into(), None);

    // post1 has both rust + async
    let id1 = helpers::add_post(&m, "async rust".into(), None, None, Some(now), Some(now));
    helpers::add_post_tags(&m, id1, &[tag_a, tag_b]);

    // post2 has only rust
    let id2 = helpers::add_post(&m, "rust only".into(), None, None, Some(now), Some(now));
    helpers::add_post_tags(&m, id2, &[tag_a]);

    // post3 has python
    let _id3 = helpers::add_post(&m, "python".into(), None, None, Some(now), Some(now));
    helpers::add_post_tags(&m, _id3, &[tag_c]);

    // Filter by both rust AND async → only post1
    let mut q = m.posts();
    q.tags.extend([tag_a, tag_b]);
    let posts = q.query().unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].id, id1);
}

// ── posts().authors filter ────────────────────────────────────────────────────

#[test]
fn test_posts_filter_by_author() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let author_a = helpers::add_author(&m, "Alice".into(), Some(now));
    let author_b = helpers::add_author(&m, "Bob".into(), Some(now));

    let id1 = helpers::add_post(&m, "By Alice".into(), None, None, Some(now), Some(now));
    let id2 = helpers::add_post(&m, "By Bob".into(), None, None, Some(now), Some(now));
    helpers::add_post_authors(&m, id1, &[author_a]);
    helpers::add_post_authors(&m, id2, &[author_b]);

    let mut q = m.posts();
    q.authors.insert(author_a);
    let posts = q.query().unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].id, id1);
}

// ── posts().ids filter ───────────────────────────────────────────────────────

#[test]
fn test_posts_filter_by_single_id() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let id1 = helpers::add_post(&m, "One".into(), None, None, Some(now), Some(now));
    let _id2 = helpers::add_post(&m, "Two".into(), None, None, Some(now), Some(now));

    let mut q = m.posts();
    q.ids.insert(id1);
    let posts = q.query().unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].id, id1);
}

#[test]
fn test_posts_filter_by_multiple_ids() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let id1 = helpers::add_post(&m, "One".into(), None, None, Some(now), Some(now));
    let id2 = helpers::add_post(&m, "Two".into(), None, None, Some(now), Some(now));
    let _id3 = helpers::add_post(&m, "Three".into(), None, None, Some(now), Some(now));

    let mut q = m.posts();
    q.ids.extend([id1, id2]);
    let posts = q.query().unwrap();
    assert_eq!(posts.len(), 2);
    let ids: Vec<_> = posts.iter().map(|p| p.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

// ── posts().pagination() ─────────────────────────────────────────────────────

#[test]
fn test_posts_pagination() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    for i in 0..10u32 {
        helpers::add_post(&m, format!("Post {i}"), None, None, Some(now), Some(now));
    }

    let page1 = m.posts().pagination(3, 0).query().unwrap();
    let page2 = m.posts().pagination(3, 1).query().unwrap();
    let page4 = m.posts().pagination(3, 3).query().unwrap();

    assert_eq!(page1.len(), 3);
    assert_eq!(page2.len(), 3);
    assert_eq!(page4.len(), 1); // 10 posts: page 3 (0-based) has 1 item

    // no overlap between pages
    let ids1: Vec<_> = page1.iter().map(|p| p.id).collect();
    let ids2: Vec<_> = page2.iter().map(|p| p.id).collect();
    assert!(ids1.iter().all(|id| !ids2.contains(id)));
}

// ── posts().sort() ────────────────────────────────────────────────────────────

#[test]
fn test_posts_sort_by_title_asc() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    helpers::add_post(&m, "Zebra".into(), None, None, Some(now), Some(now));
    helpers::add_post(&m, "Apple".into(), None, None, Some(now), Some(now));
    helpers::add_post(&m, "Mango".into(), None, None, Some(now), Some(now));

    let posts = m
        .posts()
        .sort(PostSort::Title, SortDir::Asc)
        .query()
        .unwrap();
    let titles: Vec<_> = posts.iter().map(|p| p.title.as_str()).collect();
    assert_eq!(titles, vec!["Apple", "Mango", "Zebra"]);
}

#[test]
fn test_posts_sort_by_title_desc() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    helpers::add_post(&m, "Zebra".into(), None, None, Some(now), Some(now));
    helpers::add_post(&m, "Apple".into(), None, None, Some(now), Some(now));
    helpers::add_post(&m, "Mango".into(), None, None, Some(now), Some(now));

    let posts = m
        .posts()
        .sort(PostSort::Title, SortDir::Desc)
        .query()
        .unwrap();
    let titles: Vec<_> = posts.iter().map(|p| p.title.as_str()).collect();
    assert_eq!(titles, vec!["Zebra", "Mango", "Apple"]);
}

#[test]
fn test_posts_sort_by_updated_desc() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let t0 = Utc::now() - Duration::seconds(60);
    let t1 = Utc::now() - Duration::seconds(30);
    let t2 = Utc::now();

    let id_old = helpers::add_post(&m, "Old".into(), None, None, None, Some(t0));
    let id_mid = helpers::add_post(&m, "Mid".into(), None, None, None, Some(t1));
    let id_new = helpers::add_post(&m, "New".into(), None, None, None, Some(t2));

    let posts = m
        .posts()
        .sort(PostSort::Updated, SortDir::Desc)
        .query()
        .unwrap();
    let ids: Vec<_> = posts.iter().map(|p| p.id).collect();
    assert_eq!(ids, vec![id_new, id_mid, id_old]);
}

// ── posts().with_total() ─────────────────────────────────────────────────────

#[test]
fn test_posts_with_total_empty() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let result = m.posts().with_total().query().unwrap();
    assert_eq!(result.total, 0);
    assert!(result.items.is_empty());
}

#[test]
fn test_posts_with_total_counts_all() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    for i in 0..7u32 {
        helpers::add_post(&m, format!("Post {i}"), None, None, Some(now), Some(now));
    }

    let result = m.posts().pagination(3, 0).with_total().query().unwrap();
    assert_eq!(result.total, 7); // total is 7 regardless of page
    assert_eq!(result.items.len(), 3); // page 0 has 3 items
}

#[test]
fn test_posts_with_total_filtered() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let p = helpers::add_platform(&m, "x".into());
    for _ in 0..4u32 {
        helpers::add_post(&m, "with plt".into(), None, Some(p), Some(now), Some(now));
    }
    for _ in 0..3u32 {
        helpers::add_post(&m, "no plt".into(), None, None, Some(now), Some(now));
    }

    let mut q = m.posts();
    q.platforms.insert(p);
    let result = q.with_total().query().unwrap();
    assert_eq!(result.total, 4);
    assert_eq!(result.items.len(), 4);
}

// ── posts relations via bind() ────────────────────────────────────────────────

#[test]
fn test_posts_with_relations_empty_associations() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let id = helpers::add_post(&m, "Solo".into(), None, None, Some(now), Some(now));

    let mut q = m.posts();
    q.ids.insert(id);
    let posts = q.query().unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].id, id);

    let bound = m.bind(id);
    assert!(bound.list_authors().unwrap().is_empty());
    assert!(bound.list_tags().unwrap().is_empty());
    assert!(bound.list_file_metas().unwrap().is_empty());
    assert!(bound.list_collections().unwrap().is_empty());
}

#[test]
fn test_posts_with_relations_loaded() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();

    let author_id = helpers::add_author(&m, "Writer".into(), Some(now));
    let tag_id = helpers::add_tag(&m, "tech".into(), None);
    let coll_id = helpers::add_collection(&m, "Series".into(), Some("src:1".into()), None);
    let post_id = helpers::add_post(&m, "Rich Post".into(), None, None, Some(now), Some(now));
    let file_id = helpers::add_file_meta(
        &m,
        post_id,
        "img.png".into(),
        "image/png".into(),
        Default::default(),
    );
    helpers::add_post_authors(&m, post_id, &[author_id]);
    helpers::add_post_tags(&m, post_id, &[tag_id]);
    helpers::add_post_collections(&m, post_id, &[coll_id]);

    let bound = m.bind(post_id);
    let authors = bound.list_authors().unwrap();
    assert_eq!(authors.len(), 1);
    assert_eq!(authors[0], author_id);

    let tags = bound.list_tags().unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0], tag_id);

    let file_metas = bound.list_file_metas().unwrap();
    assert_eq!(file_metas.len(), 1);
    assert_eq!(file_metas[0], file_id);

    let collections = bound.list_collections().unwrap();
    assert_eq!(collections.len(), 1);
    assert_eq!(collections[0], coll_id);
}

// ── posts pagination with tag filter ─────────────────────────────────────────

#[test]
fn test_posts_relations_with_total() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let tag = helpers::add_tag(&m, "t".into(), None);
    for i in 0..5u32 {
        let id = helpers::add_post(&m, format!("P{i}"), None, None, Some(now), Some(now));
        helpers::add_post_tags(&m, id, &[tag]);
    }
    // 3 posts without the tag
    for i in 0..3u32 {
        helpers::add_post(&m, format!("Q{i}"), None, None, Some(now), Some(now));
    }

    let mut q = m.posts();
    q.tags.insert(tag);
    let result = q.pagination(2, 0).with_total().query().unwrap();
    assert_eq!(result.total, 5);
    assert_eq!(result.items.len(), 2);
}

// ── get_post not found ───────────────────────────────────────────────────────

#[test]
fn test_get_post_not_found_for_relations() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    use crate::PostId;
    let result = m.get_post(PostId::from(999u32)).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_get_post_with_relations_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post_id = helpers::add_post(&m, "Full".into(), None, None, Some(now), Some(now));
    let author_id = helpers::add_author(&m, "Author".into(), Some(now));
    helpers::add_post_authors(&m, post_id, &[author_id]);

    let post = m.get_post(post_id).unwrap().unwrap();
    assert_eq!(post.id, post_id);

    let authors = m.bind(post_id).list_authors().unwrap();
    assert_eq!(authors.len(), 1);
}

// ── posts().collections filter ────────────────────────────────────────────────

#[test]
fn test_posts_filter_by_collection() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let col = helpers::add_collection(&m, "Series".into(), Some("s".into()), None);
    let id1 = helpers::add_post(&m, "P1".into(), None, None, Some(now), Some(now));
    let id2 = helpers::add_post(&m, "P2".into(), None, None, Some(now), Some(now));
    let _id3 = helpers::add_post(&m, "P3-other".into(), None, None, Some(now), Some(now));
    helpers::add_post_collections(&m, id1, &[col]);
    helpers::add_post_collections(&m, id2, &[col]);

    let mut q = m.posts();
    q.collections.insert(col);
    let posts = q.query().unwrap();
    assert_eq!(posts.len(), 2);
    let ids: Vec<_> = posts.iter().map(|p| p.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

#[test]
fn test_posts_filter_by_collection_empty() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let col = helpers::add_collection(&m, "Empty".into(), None, None);

    let mut q = m.posts();
    q.collections.insert(col);
    let posts = q.query().unwrap();
    assert!(posts.is_empty());
}

#[test]
fn test_posts_filter_by_collection_isolates_correctly() {
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

    let mut q_a = m.posts();
    q_a.collections.insert(col_a);
    let posts_a = q_a.query().unwrap();
    assert_eq!(posts_a.len(), 2);

    let mut q_b = m.posts();
    q_b.collections.insert(col_b);
    let posts_b = q_b.query().unwrap();
    assert_eq!(posts_b.len(), 2);

    let ids_a: Vec<_> = posts_a.iter().map(|p| p.id).collect();
    assert!(ids_a.contains(&id1));
    assert!(ids_a.contains(&id3));
}

// ── combined filters ──────────────────────────────────────────────────────────

#[test]
fn test_posts_filter_platform_and_author_and_tag() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let plt = helpers::add_platform(&m, "p".into());
    let author = helpers::add_author(&m, "Alice".into(), Some(now));
    let tag = helpers::add_tag(&m, "cool".into(), None);

    // matches all three
    let id_match = helpers::add_post(&m, "Match".into(), None, Some(plt), Some(now), Some(now));
    helpers::add_post_authors(&m, id_match, &[author]);
    helpers::add_post_tags(&m, id_match, &[tag]);

    // wrong platform
    let id_wp = helpers::add_post(&m, "WrongPlt".into(), None, None, Some(now), Some(now));
    helpers::add_post_authors(&m, id_wp, &[author]);
    helpers::add_post_tags(&m, id_wp, &[tag]);

    // no author
    let id_na = helpers::add_post(&m, "NoAuthor".into(), None, Some(plt), Some(now), Some(now));
    helpers::add_post_tags(&m, id_na, &[tag]);

    let mut q = m.posts();
    q.platforms.insert(plt);
    q.authors.insert(author);
    q.tags.insert(tag);
    let posts = q.query().unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].id, id_match);
}
