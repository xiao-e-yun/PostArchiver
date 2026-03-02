//! Tests for `src/query/author.rs`

use crate::{
    manager::PostArchiverManager,
    query::{author::AuthorSort, Countable, Paginate, Query, SortDir, Sortable},
    tests::helpers,
};
use chrono::Utc;

// ── get_author ────────────────────────────────────────────────────────────────

#[test]
fn test_get_author_exists() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let id = helpers::add_author(&m, "Alice".into(), Some(now));

    let author = m.get_author(id).unwrap().unwrap();
    assert_eq!(author.id, id);
    assert_eq!(author.name, "Alice");
}

#[test]
fn test_get_author_not_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    use crate::AuthorId;
    let result = m.get_author(AuthorId::from(9999u32)).unwrap();
    assert!(result.is_none());
}

// ── find_author_by_alias ──────────────────────────────────────────────────────

#[test]
fn test_find_author_by_alias_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let plt = helpers::add_platform(&m, "github".into());
    let id = helpers::add_author(&m, "Octocat".into(), Some(now));
    helpers::add_author_aliases(&m, id, vec![("octocat".into(), plt, None)]);

    let found = m.find_author_by_alias("octocat", plt).unwrap();
    assert_eq!(found, Some(id));
}

#[test]
fn test_find_author_by_alias_not_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let plt = helpers::add_platform(&m, "x".into());

    let result = m.find_author_by_alias("nobody", plt).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_find_author_by_alias_wrong_platform() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let plt_a = helpers::add_platform(&m, "platformA".into());
    let plt_b = helpers::add_platform(&m, "platformB".into());
    let id = helpers::add_author(&m, "Alice".into(), Some(now));
    helpers::add_author_aliases(&m, id, vec![("alice".into(), plt_a, None)]);

    // same source but different platform → not found
    let result = m.find_author_by_alias("alice", plt_b).unwrap();
    assert!(result.is_none());
}

// ── authors().query() – basic list ───────────────────────────────────────────

#[test]
fn test_authors_empty() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let authors = m.authors().query().unwrap();
    assert!(authors.is_empty());
}

#[test]
fn test_authors_returns_all() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let id1 = helpers::add_author(&m, "Alice".into(), Some(now));
    let id2 = helpers::add_author(&m, "Bob".into(), Some(now));

    let authors = m.authors().query().unwrap();
    assert_eq!(authors.len(), 2);
    let ids: Vec<_> = authors.iter().map(|a| a.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

// ── authors().name.contains() ────────────────────────────────────────────────

#[test]
fn test_authors_name_contains() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    helpers::add_author(&m, "Alice Smith".into(), Some(now));
    helpers::add_author(&m, "Bob Jones".into(), Some(now));
    helpers::add_author(&m, "Alice Wonderland".into(), Some(now));

    let mut q = m.authors();
    q.name.contains("Alice");
    let results = q.query().unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|a| a.name.contains("Alice")));
}

#[test]
fn test_authors_name_contains_case_insensitive() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    helpers::add_author(&m, "alice".into(), Some(now));
    helpers::add_author(&m, "ALICE".into(), Some(now));

    // LIKE '%alice%' is case-insensitive for ASCII in SQLite
    let mut q = m.authors();
    q.name.contains("alice");
    let results = q.query().unwrap();
    assert_eq!(results.len(), 2);
}

#[test]
fn test_authors_name_contains_no_match() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    helpers::add_author(&m, "Alice".into(), Some(now));

    let mut q = m.authors();
    q.name.contains("xyz");
    let results = q.query().unwrap();
    assert!(results.is_empty());
}

// ── authors().sort() ─────────────────────────────────────────────────────────

#[test]
fn test_authors_sort_by_name_asc() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    helpers::add_author(&m, "Zara".into(), Some(now));
    helpers::add_author(&m, "Anna".into(), Some(now));
    helpers::add_author(&m, "Mike".into(), Some(now));

    let authors = m
        .authors()
        .sort(AuthorSort::Name, SortDir::Asc)
        .query()
        .unwrap();
    let names: Vec<_> = authors.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, vec!["Anna", "Mike", "Zara"]);
}

#[test]
fn test_authors_sort_by_name_desc() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    helpers::add_author(&m, "Zara".into(), Some(now));
    helpers::add_author(&m, "Anna".into(), Some(now));
    helpers::add_author(&m, "Mike".into(), Some(now));

    let authors = m
        .authors()
        .sort(AuthorSort::Name, SortDir::Desc)
        .query()
        .unwrap();
    let names: Vec<_> = authors.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, vec!["Zara", "Mike", "Anna"]);
}

// ── authors().pagination() ───────────────────────────────────────────────────

#[test]
fn test_authors_pagination() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    for i in 0..5u32 {
        helpers::add_author(&m, format!("Author-{i:02}"), Some(now));
    }

    let page1 = m
        .authors()
        .sort(AuthorSort::Name, SortDir::Asc)
        .pagination(2, 0)
        .query()
        .unwrap();
    let page2 = m
        .authors()
        .sort(AuthorSort::Name, SortDir::Asc)
        .pagination(2, 1)
        .query()
        .unwrap();

    assert_eq!(page1.len(), 2);
    assert_eq!(page2.len(), 2);

    let ids1: Vec<_> = page1.iter().map(|a| a.id).collect();
    let ids2: Vec<_> = page2.iter().map(|a| a.id).collect();
    assert!(ids1.iter().all(|id| !ids2.contains(id)));
}

// ── authors().with_total() ───────────────────────────────────────────────────

#[test]
fn test_authors_with_total() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    for i in 0..6u32 {
        helpers::add_author(&m, format!("A{i}"), Some(now));
    }

    let result = m.authors().pagination(2, 0).with_total().query().unwrap();
    assert_eq!(result.total, 6);
    assert_eq!(result.items.len(), 2);
}

#[test]
fn test_authors_with_total_filtered() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    helpers::add_author(&m, "Alice A".into(), Some(now));
    helpers::add_author(&m, "Alice B".into(), Some(now));
    helpers::add_author(&m, "Bob C".into(), Some(now));

    let mut q = m.authors();
    q.name.contains("Alice");
    let result = q.with_total().query().unwrap();
    assert_eq!(result.total, 2);
    assert_eq!(result.items.len(), 2);
}

// ── author relations via bind() ──────────────────────────────────────────────

#[test]
fn test_authors_with_relations() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let plt = helpers::add_platform(&m, "gh".into());
    let author_id = helpers::add_author(&m, "Dev".into(), Some(now));
    helpers::add_author_aliases(&m, author_id, vec![("dev".into(), plt, None)]);
    let post_id = helpers::add_post(&m, "Post".into(), None, None, Some(now), Some(now));
    helpers::add_post_authors(&m, post_id, &[author_id]);

    let authors = m.authors().query().unwrap();
    assert_eq!(authors.len(), 1);
    assert_eq!(authors[0].id, author_id);

    let aliases = m.bind(author_id).list_aliases().unwrap();
    assert_eq!(aliases.len(), 1);

    let posts = m.bind(author_id).list_posts().unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0], post_id);
}

#[test]
fn test_authors_relations_with_total() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    for i in 0..4u32 {
        helpers::add_author(&m, format!("A{i}"), Some(now));
    }

    let result = m.authors().pagination(2, 0).with_total().query().unwrap();
    assert_eq!(result.total, 4);
    assert_eq!(result.items.len(), 2);
}

// ── author aliases via bind() ───────────────────────────────────────────────

#[test]
fn test_author_aliases_via_bind() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let plt1 = helpers::add_platform(&m, "github".into());
    let plt2 = helpers::add_platform(&m, "twitter".into());
    let id = helpers::add_author(&m, "Dev".into(), Some(now));
    helpers::add_author_aliases(
        &m,
        id,
        vec![
            ("dev-gh".into(), plt1, None),
            (
                "dev-tw".into(),
                plt2,
                Some("https://twitter.com/dev".into()),
            ),
        ],
    );

    let aliases = m.bind(id).list_aliases().unwrap();
    assert_eq!(aliases.len(), 2);
    let sources: Vec<_> = aliases.iter().map(|a| a.source.as_str()).collect();
    assert!(sources.contains(&"dev-gh"));
    assert!(sources.contains(&"dev-tw"));
}

#[test]
fn test_author_aliases_empty_via_bind() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let id = helpers::add_author(&m, "NoAlias".into(), Some(now));

    let aliases = m.bind(id).list_aliases().unwrap();
    assert!(aliases.is_empty());
}

// ── author posts via posts() builder ────────────────────────────────────────

#[test]
fn test_author_posts_via_builder() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let author = helpers::add_author(&m, "Writer".into(), Some(now));
    let id1 = helpers::add_post(&m, "P1".into(), None, None, Some(now), Some(now));
    let id2 = helpers::add_post(&m, "P2".into(), None, None, Some(now), Some(now));
    let _id3 = helpers::add_post(&m, "P3-other".into(), None, None, Some(now), Some(now));
    helpers::add_post_authors(&m, id1, &[author]);
    helpers::add_post_authors(&m, id2, &[author]);

    let mut q = m.posts();
    q.authors.insert(author);
    let posts = q.query().unwrap();
    assert_eq!(posts.len(), 2);
    let ids: Vec<_> = posts.iter().map(|p| p.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

#[test]
fn test_author_posts_empty_via_builder() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let author = helpers::add_author(&m, "Silent".into(), Some(now));

    let mut q = m.posts();
    q.authors.insert(author);
    let posts = q.query().unwrap();
    assert!(posts.is_empty());
}
