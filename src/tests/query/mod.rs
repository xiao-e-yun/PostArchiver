//! Tests for the query module

use chrono::Utc;

use crate::{
    impl_from_query,
    query::{Countable, Paginate, Query, Sortable},
    Comment, Post,
};

pub mod author;
pub mod collection;
pub mod file_meta;
pub mod platform;
pub mod post;
pub mod tag;

#[test]
fn test_custom_query() {
    let manager = crate::manager::PostArchiverManager::open_in_memory().unwrap();

    // Add a post published 2 days ago — should pass the filter
    let old_id = crate::tests::helpers::add_post(
        &manager,
        "Old Post".into(),
        None,
        None,
        Some(Utc::now() - chrono::Duration::days(2)),
        Some(Utc::now() - chrono::Duration::days(2)),
    );

    // Add a post published just now — should be excluded by the filter
    crate::tests::helpers::add_post(
        &manager,
        "New Post".into(),
        None,
        None,
        Some(Utc::now()),
        Some(Utc::now()),
    );

    let mut posts = manager.posts();

    posts
        .published
        .before(Utc::now() - chrono::Duration::days(1));

    struct PostPreview {
        id: i64,
        title: String,
        // Test json-serialized fields that are not in the Post struct, to ensure FromQuery can handle it
        comment: Vec<Comment>,
    }
    impl_from_query! {
        PostPreview extends Post {
            id: "id",
            title: "title",
            comment: "comments" => json
        }
    }

    let results = posts
        .pagination(10, 0)
        .sort_random()
        .with_total()
        .query::<PostPreview>()
        .unwrap();

    // Only the old post matches the published-before filter
    assert_eq!(results.total, 1);
    assert_eq!(results.items.len(), 1);

    let preview = &results.items[0];
    assert_eq!(preview.id, *old_id as i64);
    assert_eq!(preview.title, "Old Post");
    assert!(preview.comment.is_empty());
}
