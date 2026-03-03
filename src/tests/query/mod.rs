//! Tests for the query module

use chrono::Utc;

use crate::{
    query::{Countable, FromQuery, Paginate, Query, Sortable},
    Post,
};

pub mod author;
pub mod collection;
pub mod file_meta;
pub mod platform;
pub mod post;
pub mod tag;

#[test]
fn test_query() {
    let manager = crate::manager::PostArchiverManager::open_in_memory().unwrap();

    let mut posts = manager.posts();

    posts
        .published
        .before(Utc::now() - chrono::Duration::days(1));

    struct PostPreview {
        id: i64,
        title: String,
    }
    impl FromQuery for PostPreview {
        type Based = Post;

        fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
            Ok(PostPreview {
                id: row.get("id")?,
                title: row.get("title")?,
            })
        }

        fn select_sql() -> String {
            "SELECT id, title FROM posts".to_string()
        }
    }

    let results = posts
        .pagination(10, 5)
        .sort_random()
        .with_total()
        .query::<PostPreview>()
        .unwrap();
}
