//! Tests for `src/query/post.rs`

use crate::{manager::PostArchiverManager, query::post::PostSort, query::SortDir, tests::helpers};
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

// ── posts().platform() filter ─────────────────────────────────────────────────

#[test]
fn test_posts_filter_by_platform() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let p1 = helpers::add_platform(&m, "platformA".into());
    let p2 = helpers::add_platform(&m, "platformB".into());

    let id1 = helpers::add_post(&m, "P1 post".into(), None, Some(p1), Some(now), Some(now));
    let _id2 = helpers::add_post(&m, "P2 post".into(), None, Some(p2), Some(now), Some(now));

    let posts = m.posts().platform(p1).query().unwrap();
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

    let posts = m.posts().platforms([p1, p2]).query().unwrap();
    assert_eq!(posts.len(), 2);
    let ids: Vec<_> = posts.iter().map(|p| p.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

// ── posts().tags() filter – AND semantics ────────────────────────────────────

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

    let posts = m.posts().tags([tag_a]).query().unwrap();
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
    let posts = m.posts().tags([tag_a, tag_b]).query().unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].id, id1);
}

// ── posts().author() filter ───────────────────────────────────────────────────

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

    let posts = m.posts().author(author_a).query().unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].id, id1);
}

// ── posts().id() / .ids() filter ─────────────────────────────────────────────

#[test]
fn test_posts_filter_by_single_id() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let id1 = helpers::add_post(&m, "One".into(), None, None, Some(now), Some(now));
    let _id2 = helpers::add_post(&m, "Two".into(), None, None, Some(now), Some(now));

    let posts = m.posts().id(id1).query().unwrap();
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

    let posts = m.posts().ids([id1, id2]).query().unwrap();
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

    let result = m.posts().platform(p).with_total().query().unwrap();
    assert_eq!(result.total, 4);
    assert_eq!(result.items.len(), 4);
}

// ── posts().relations() ───────────────────────────────────────────────────────

#[test]
fn test_posts_with_relations_empty_associations() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let id = helpers::add_post(&m, "Solo".into(), None, None, Some(now), Some(now));

    let result = m.posts().id(id).relations().query().unwrap();
    assert_eq!(result.len(), 1);
    let wr = &result[0];
    assert_eq!(wr.post.id, id);
    assert!(wr.authors.is_empty());
    assert!(wr.tags.is_empty());
    assert!(wr.files.is_empty());
    assert!(wr.collections.is_empty());
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

    let result = m.posts().id(post_id).relations().query().unwrap();
    let wr = &result[0];
    assert_eq!(wr.authors.len(), 1);
    assert_eq!(wr.authors[0].id, author_id);
    assert_eq!(wr.tags.len(), 1);
    assert_eq!(wr.tags[0].id, tag_id);
    assert_eq!(wr.files.len(), 1);
    assert_eq!(wr.files[0].id, file_id);
    assert_eq!(wr.collections.len(), 1);
    assert_eq!(wr.collections[0].id, coll_id);
}

// ── posts().relations().with_total() ─────────────────────────────────────────

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

    let result = m
        .posts()
        .tags([tag])
        .pagination(2, 0)
        .relations()
        .with_total()
        .query()
        .unwrap();
    assert_eq!(result.total, 5);
    assert_eq!(result.items.len(), 2);
}

// ── get_post_with_relations ───────────────────────────────────────────────────

#[test]
fn test_get_post_with_relations_not_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    use crate::PostId;
    let result = m.get_post_with_relations(PostId::from(999u32)).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_get_post_with_relations_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post_id = helpers::add_post(&m, "Full".into(), None, None, Some(now), Some(now));
    let author_id = helpers::add_author(&m, "Author".into(), Some(now));
    helpers::add_post_authors(&m, post_id, &[author_id]);

    let wr = m.get_post_with_relations(post_id).unwrap().unwrap();
    assert_eq!(wr.post.id, post_id);
    assert_eq!(wr.authors.len(), 1);
}

// ── list_post_authors / list_post_tags / list_post_files / list_post_collections

#[test]
fn test_list_post_authors() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let a1 = helpers::add_author(&m, "A1".into(), Some(now));
    let a2 = helpers::add_author(&m, "A2".into(), Some(now));
    let pid = helpers::add_post(&m, "Post".into(), None, None, Some(now), Some(now));
    helpers::add_post_authors(&m, pid, &[a1, a2]);

    let authors = m.list_post_authors(pid).unwrap();
    assert_eq!(authors.len(), 2);
    let ids: Vec<_> = authors.iter().map(|a| a.id).collect();
    assert!(ids.contains(&a1));
    assert!(ids.contains(&a2));
}

#[test]
fn test_list_post_tags() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let t1 = helpers::add_tag(&m, "t1".into(), None);
    let t2 = helpers::add_tag(&m, "t2".into(), None);
    let pid = helpers::add_post(&m, "Post".into(), None, None, Some(now), Some(now));
    helpers::add_post_tags(&m, pid, &[t1, t2]);

    let tags = m.list_post_tags(pid).unwrap();
    assert_eq!(tags.len(), 2);
    let ids: Vec<_> = tags.iter().map(|t| t.id).collect();
    assert!(ids.contains(&t1));
    assert!(ids.contains(&t2));
}

#[test]
fn test_list_post_files() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let pid = helpers::add_post(&m, "Post".into(), None, None, Some(now), Some(now));
    let f1 = helpers::add_file_meta(
        &m,
        pid,
        "a.jpg".into(),
        "image/jpeg".into(),
        Default::default(),
    );
    let f2 = helpers::add_file_meta(
        &m,
        pid,
        "b.mp4".into(),
        "video/mp4".into(),
        Default::default(),
    );

    let files = m.list_post_files(pid).unwrap();
    assert_eq!(files.len(), 2);
    let ids: Vec<_> = files.iter().map(|f| f.id).collect();
    assert!(ids.contains(&f1));
    assert!(ids.contains(&f2));
}

#[test]
fn test_list_post_collections() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let c1 = helpers::add_collection(&m, "C1".into(), Some("s1".into()), None);
    let c2 = helpers::add_collection(&m, "C2".into(), Some("s2".into()), None);
    let pid = helpers::add_post(&m, "Post".into(), None, None, Some(now), Some(now));
    helpers::add_post_collections(&m, pid, &[c1, c2]);

    let cols = m.list_post_collections(pid).unwrap();
    assert_eq!(cols.len(), 2);
    let ids: Vec<_> = cols.iter().map(|c| c.id).collect();
    assert!(ids.contains(&c1));
    assert!(ids.contains(&c2));
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

    let posts = m
        .posts()
        .platform(plt)
        .author(author)
        .tags([tag])
        .query()
        .unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].id, id_match);
}
