//! Author manager tests
//!
//! Tests for author CRUD operations, alias management,
//! and author-post relationships.

use crate::tests::common::*;
use crate::{AuthorId, PlatformId, PostId};
use chrono::Utc;

#[test]
fn test_add_author() {
    with_test_db(|manager| {
        let name = "Test Author".to_string();
        let updated = Some(Utc::now());

        let author_id = manager
            .add_author(name.clone(), updated)
            .expect("Failed to add author");

        assert!(author_id.raw() > 0);

        // Verify the author was added
        let author = manager.get_author(author_id).expect("Failed to get author");

        assert_eq!(author.name, name);
        assert_eq!(author.id, author_id);
        assert_eq!(author.thumb, None);
    });
}

#[test]
fn test_list_authors() {
    with_test_db(|manager| {
        // Add multiple authors
        let id1 = manager
            .add_author("Author 1".to_string(), Some(Utc::now()))
            .expect("Failed to add author 1");
        let id2 = manager
            .add_author("Author 2".to_string(), Some(Utc::now()))
            .expect("Failed to add author 2");

        let authors = manager.list_authors().expect("Failed to list authors");

        assert_eq!(authors.len(), 2);

        let ids: Vec<AuthorId> = authors.iter().map(|a| a.id).collect();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    });
}

#[test]
fn test_get_author() {
    with_test_db(|manager| {
        let name = "Get Test Author".to_string();
        let author_id = manager
            .add_author(name.clone(), Some(Utc::now()))
            .expect("Failed to add author");

        let author = manager.get_author(author_id).expect("Failed to get author");

        assert_eq!(author.id, author_id);
        assert_eq!(author.name, name);
    });
}

#[test]
fn test_remove_author() {
    with_test_db(|manager| {
        let author_id = manager
            .add_author("To Delete".to_string(), Some(Utc::now()))
            .expect("Failed to add author");

        // Verify author exists
        manager
            .get_author(author_id)
            .expect("Author should exist before deletion");

        // Remove author
        manager
            .remove_author(author_id)
            .expect("Failed to remove author");

        // Verify author is gone
        let result = manager.get_author(author_id);
        assert!(result.is_err(), "Author should not exist after deletion");
    });
}

#[test]
fn test_set_author_name() {
    with_test_db(|manager| {
        let author_id = manager
            .add_author("Original Name".to_string(), Some(Utc::now()))
            .expect("Failed to add author");

        let new_name = "Updated Name".to_string();
        manager
            .set_author_name(author_id, new_name.clone())
            .expect("Failed to update author name");

        let author = manager.get_author(author_id).expect("Failed to get author");

        assert_eq!(author.name, new_name);
    });
}

#[test]
fn test_set_author_updated() {
    with_test_db(|manager| {
        let author_id = manager
            .add_author("Test Author".to_string(), Some(Utc::now()))
            .expect("Failed to add author");

        let new_updated = Utc::now();
        manager
            .set_author_updated(author_id, new_updated)
            .expect("Failed to update author timestamp");

        let author = manager.get_author(author_id).expect("Failed to get author");

        // Allow small time difference due to precision
        let diff = (author.updated - new_updated).num_milliseconds().abs();
        assert!(diff < 1000, "Updated timestamp should be close to expected");
    });
}

#[test]
fn test_add_author_aliases() {
    with_test_db(|manager| {
        // Add platform and author
        let platform_id = manager
            .add_platform("Test Platform".to_string())
            .expect("Failed to add platform");
        let author_id = manager
            .add_author("Test Author".to_string(), Some(Utc::now()))
            .expect("Failed to add author");

        // Add aliases
        let aliases = vec![
            (
                "alias1".to_string(),
                platform_id,
                Some("http://example.com/alias1".to_string()),
            ),
            ("alias2".to_string(), platform_id, None),
        ];

        manager
            .add_author_aliases(author_id, aliases)
            .expect("Failed to add aliases");

        // Verify aliases were added
        let stored_aliases = manager
            .list_author_aliases(author_id)
            .expect("Failed to list aliases");

        assert_eq!(stored_aliases.len(), 2);

        let alias_sources: Vec<String> = stored_aliases.iter().map(|a| a.source.clone()).collect();
        assert!(alias_sources.contains(&"alias1".to_string()));
        assert!(alias_sources.contains(&"alias2".to_string()));
    });
}

#[test]
fn test_find_author() {
    with_test_db(|manager| {
        // Add platform and author
        let platform_id = manager
            .add_platform("Test Platform".to_string())
            .expect("Failed to add platform");
        let author_id = manager
            .add_author("Test Author".to_string(), Some(Utc::now()))
            .expect("Failed to add author");

        // Add alias
        let aliases = vec![("findme".to_string(), platform_id, None)];
        manager
            .add_author_aliases(author_id, aliases)
            .expect("Failed to add aliases");

        // Find author by alias
        let found_id = manager
            .find_author(&[("findme", platform_id)])
            .expect("Failed to find author");

        assert_eq!(found_id, Some(author_id));

        // Test not found
        let not_found = manager
            .find_author(&[("nonexistent", platform_id)])
            .expect("Failed to search for nonexistent author");

        assert_eq!(not_found, None);
    });
}

#[test]
fn test_remove_author_aliases() {
    with_test_db(|manager| {
        // Setup
        let platform_id = manager
            .add_platform("Test Platform".to_string())
            .expect("Failed to add platform");
        let author_id = manager
            .add_author("Test Author".to_string(), Some(Utc::now()))
            .expect("Failed to add author");

        // Add aliases
        let aliases = vec![
            ("keep".to_string(), platform_id, None),
            ("remove".to_string(), platform_id, None),
        ];
        manager
            .add_author_aliases(author_id, aliases)
            .expect("Failed to add aliases");

        // Remove one alias
        manager
            .remove_author_aliases(author_id, &[("remove".to_string(), platform_id)])
            .expect("Failed to remove alias");

        // Verify only one alias remains
        let remaining_aliases = manager
            .list_author_aliases(author_id)
            .expect("Failed to list aliases");

        assert_eq!(remaining_aliases.len(), 1);
        assert_eq!(remaining_aliases[0].source, "keep");
    });
}

#[test]
fn test_author_post_relationships() {
    with_test_db(|manager| {
        // Setup
        let author_id = manager
            .add_author("Test Author".to_string(), Some(Utc::now()))
            .expect("Failed to add author");
        let post_id = manager
            .add_post(
                "Test Post".to_string(),
                None,
                None,
                Some(Utc::now()),
                Some(Utc::now()),
            )
            .expect("Failed to add post");

        // Add author-post relationship
        manager
            .add_post_authors(post_id, &[author_id])
            .expect("Failed to add post authors");

        // Test author's posts
        let author_posts = manager
            .list_author_posts(author_id)
            .expect("Failed to list author posts");

        assert_eq!(author_posts.len(), 1);
        assert_eq!(author_posts[0].id, post_id);

        // Test post's authors
        let post = manager.get_post(&post_id).expect("Failed to get post");
        let post_authors = manager
            .list_post_authors(&post)
            .expect("Failed to list post authors");

        assert_eq!(post_authors.len(), 1);
        assert_eq!(post_authors[0].id, author_id);
    });
}

#[test]
fn test_set_author_updated_by_latest() {
    with_test_db(|manager| {
        let author_id = manager
            .add_author("Test Author".to_string(), Some(Utc::now()))
            .expect("Failed to add author");

        // Add a post with a later timestamp
        let later_time = Utc::now();
        let post_id = manager
            .add_post(
                "Latest Post".to_string(),
                None,
                None,
                Some(later_time),
                Some(later_time),
            )
            .expect("Failed to add post");

        manager
            .add_post_authors(post_id, &[author_id])
            .expect("Failed to add post authors");

        // Update author timestamp by latest post
        manager
            .set_author_updated_by_latest(author_id)
            .expect("Failed to update author by latest");

        let author = manager.get_author(author_id).expect("Failed to get author");

        // The author's updated timestamp should be close to the post's
        let diff = (author.updated - later_time).num_milliseconds().abs();
        assert!(diff < 1000, "Author timestamp should match latest post");
    });
}

#[test]
fn test_empty_find_author() {
    with_test_db(|manager| {
        // Test with empty alias list
        let result = manager
            .find_author(&[] as &[(&str, PlatformId)])
            .expect("Failed to find author with empty list");

        assert_eq!(result, None);
    });
}
