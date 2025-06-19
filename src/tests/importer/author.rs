//! Author importer tests
//!
//! Tests for author import functionality including creation,
//! updating existing authors, and alias management.

use crate::{
    importer::author::{UnsyncAlias, UnsyncAuthor},
    manager::PostArchiverManager,
};
use chrono::Utc;

#[test]
fn test_import_new_author() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    let alias = UnsyncAlias::new(platform_id, "test_author".to_string())
        .link("http://example.com/test_author".to_string());

    let unsync_author = UnsyncAuthor::new("Test Author".to_string())
        .aliases(vec![alias])
        .updated(Some(Utc::now()));

    let author_id = manager
        .import_author(unsync_author)
        .expect("Failed to import author");

    assert!(author_id.raw() > 0);

    // Verify the author was created
    let author = manager.get_author(author_id).expect("Failed to get author");
    assert_eq!(author.name, "Test Author");

    // Verify aliases were added
    let aliases = manager
        .list_author_aliases(author_id)
        .expect("Failed to list aliases");
    assert_eq!(aliases.len(), 1);
    assert_eq!(aliases[0].source, "test_author");
    assert_eq!(aliases[0].platform, platform_id);
    assert_eq!(
        aliases[0].link,
        Some("http://example.com/test_author".to_string())
    );
}

#[test]
fn test_import_existing_author_by_alias() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    // Create an existing author
    let existing_author_id = manager
        .add_author("Original Name".to_string(), Some(Utc::now()))
        .expect("Failed to add author");

    let existing_aliases = vec![("existing_alias".to_string(), platform_id, None)];
    manager
        .add_author_aliases(existing_author_id, existing_aliases)
        .expect("Failed to add aliases");

    // Try to import an author with the same alias
    let alias = UnsyncAlias::new(platform_id, "existing_alias".to_string());
    let new_alias = UnsyncAlias::new(platform_id, "new_alias".to_string());

    let unsync_author = UnsyncAuthor::new("Updated Name".to_string())
        .aliases(vec![alias, new_alias])
        .updated(Some(Utc::now()));

    let author_id = manager
        .import_author(unsync_author)
        .expect("Failed to import existing author");

    // Should return the same ID
    assert_eq!(author_id, existing_author_id);

    // Verify the name was updated
    let author = manager.get_author(author_id).expect("Failed to get author");
    assert_eq!(author.name, "Updated Name");

    // Verify aliases include both old and new
    let aliases = manager
        .list_author_aliases(author_id)
        .expect("Failed to list aliases");
    assert_eq!(aliases.len(), 2);

    let alias_sources: Vec<String> = aliases.iter().map(|a| a.source.clone()).collect();
    assert!(alias_sources.contains(&"existing_alias".to_string()));
    assert!(alias_sources.contains(&"new_alias".to_string()));
}

#[test]
fn test_unsync_author_builder() {
    let (manager, platform_id) = manager_with_platform();

    let updated_time = Utc::now();
    let alias = UnsyncAlias::new(platform_id, "builder_test".to_string())
        .link("http://example.com/builder".to_string());

    let author = UnsyncAuthor::new("Builder Test".to_string())
        .name("Updated Builder Test".to_string())
        .aliases(vec![alias])
        .updated(Some(updated_time));

    // Test by importing the author and verifying the result
    let author_id = manager
        .import_author(author)
        .expect("Failed to import author");

    let stored_author = manager.get_author(author_id).expect("Failed to get author");
    assert_eq!(stored_author.name, "Updated Builder Test");

    let diff = (stored_author.updated - updated_time)
        .num_milliseconds()
        .abs();
    assert!(
        diff < 1000,
        "Updated timestamp should be close to expected time"
    );

    let aliases = manager
        .list_author_aliases(author_id)
        .expect("Failed to list aliases");
    assert_eq!(aliases.len(), 1);
    assert_eq!(aliases[0].source, "builder_test");
    assert_eq!(aliases[0].platform, platform_id);
    assert_eq!(
        aliases[0].link,
        Some("http://example.com/builder".to_string())
    );
}

#[test]
fn test_unsync_alias_builder() {
    let (manager, platform_id) = manager_with_platform();

    let alias = UnsyncAlias::new(platform_id, "original_source".to_string())
        .source("updated_source")
        .platform(platform_id)
        .link("http://example.com/updated");

    // Test by using the alias in an author import and verifying the result
    let author = UnsyncAuthor::new("Alias Builder Test".to_string()).aliases(vec![alias]);

    let author_id = manager
        .import_author(author)
        .expect("Failed to import author");

    let aliases = manager
        .list_author_aliases(author_id)
        .expect("Failed to list aliases");

    assert_eq!(aliases.len(), 1);
    assert_eq!(aliases[0].source, "updated_source");
    assert_eq!(aliases[0].platform, platform_id);
    assert_eq!(
        aliases[0].link,
        Some("http://example.com/updated".to_string())
    );
}

#[test]
fn test_unsync_author_sync_method() {
    let (manager, platform_id) = manager_with_platform();

    let alias = UnsyncAlias::new(platform_id, "sync_test".to_string());
    let author = UnsyncAuthor::new("Sync Test Author".to_string()).aliases(vec![alias]);

    let author_id = author.sync(&manager).expect("Failed to sync author");

    assert!(author_id.raw() > 0);

    // Verify the author was created
    let stored_author = manager.get_author(author_id).expect("Failed to get author");
    assert_eq!(stored_author.name, "Sync Test Author");
}

#[test]
fn test_import_author_no_aliases() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let author = UnsyncAuthor::new("No Aliases Author".to_string()).updated(Some(Utc::now()));

    let author_id = manager
        .import_author(author)
        .expect("Failed to import author without aliases");

    // Should create new author since no aliases to match against
    let stored_author = manager.get_author(author_id).expect("Failed to get author");
    assert_eq!(stored_author.name, "No Aliases Author");

    // Verify no aliases exist
    let aliases = manager
        .list_author_aliases(author_id)
        .expect("Failed to list aliases");
    assert_eq!(aliases.len(), 0);
}

#[test]
fn test_import_author_multiple_aliases() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform1_id = manager
        .add_platform("Platform 1".to_string())
        .expect("Failed to add platform 1");
    let platform2_id = manager
        .add_platform("Platform 2".to_string())
        .expect("Failed to add platform 2");

    let alias1 = UnsyncAlias::new(platform1_id, "author_p1".to_string())
        .link("http://platform1.com/author".to_string());
    let alias2 = UnsyncAlias::new(platform2_id, "author_p2".to_string())
        .link("http://platform2.com/author".to_string());

    let author =
        UnsyncAuthor::new("Multi Platform Author".to_string()).aliases(vec![alias1, alias2]);

    let author_id = manager
        .import_author(author)
        .expect("Failed to import author with multiple aliases");

    // Verify all aliases were added
    let aliases = manager
        .list_author_aliases(author_id)
        .expect("Failed to list aliases");
    assert_eq!(aliases.len(), 2);

    // Check each alias
    let alias_data: Vec<(String, crate::PlatformId, Option<String>)> = aliases
        .iter()
        .map(|a| (a.source.clone(), a.platform, a.link.clone()))
        .collect();

    assert!(alias_data.contains(&(
        "author_p1".to_string(),
        platform1_id,
        Some("http://platform1.com/author".to_string())
    )));
    assert!(alias_data.contains(&(
        "author_p2".to_string(),
        platform2_id,
        Some("http://platform2.com/author".to_string())
    )));
}

#[test]
fn test_import_author_updates_timestamp() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    // Create existing author with old timestamp
    let old_time = chrono::DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let existing_author_id = manager
        .add_author("Test Author".to_string(), Some(old_time))
        .expect("Failed to add author");

    let existing_aliases = vec![("test_alias".to_string(), platform_id, None)];
    manager
        .add_author_aliases(existing_author_id, existing_aliases)
        .expect("Failed to add aliases");

    // Import with newer timestamp
    let new_time = Utc::now();
    let alias = UnsyncAlias::new(platform_id, "test_alias".to_string());
    let author = UnsyncAuthor::new("Test Author".to_string())
        .aliases(vec![alias])
        .updated(Some(new_time));

    let author_id = manager
        .import_author(author)
        .expect("Failed to import author");

    assert_eq!(author_id, existing_author_id);

    // Verify timestamp was updated
    let stored_author = manager.get_author(author_id).expect("Failed to get author");
    let diff = (stored_author.updated - new_time).num_milliseconds().abs();
    assert!(diff < 1000, "Updated timestamp should be close to new time");
}

#[test]
fn test_import_author_no_updated_timestamp() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    // Create existing author
    let old_time = chrono::DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let existing_author_id = manager
        .add_author("Test Author".to_string(), Some(old_time))
        .expect("Failed to add author");

    let existing_aliases = vec![("test_alias".to_string(), platform_id, None)];
    manager
        .add_author_aliases(existing_author_id, existing_aliases)
        .expect("Failed to add aliases");

    // Import without updated timestamp
    let alias = UnsyncAlias::new(platform_id, "test_alias".to_string());
    let author = UnsyncAuthor::new("Updated Name".to_string()).aliases(vec![alias]);

    let author_id = manager
        .import_author(author)
        .expect("Failed to import author");

    assert_eq!(author_id, existing_author_id);

    // Verify timestamp wasn't changed
    let stored_author = manager.get_author(author_id).expect("Failed to get author");
    let diff = (stored_author.updated - old_time).num_milliseconds().abs();
    assert!(diff < 1000, "Timestamp should remain the same");
}

#[test]
fn test_unsync_alias_no_link() {
    let (manager, platform_id) = manager_with_platform();

    let alias = UnsyncAlias::new(platform_id, "no_link_test".to_string());
    let author = UnsyncAuthor::new("No Link Author".to_string()).aliases(vec![alias]);

    let author_id = manager
        .import_author(author)
        .expect("Failed to import author");

    let aliases = manager
        .list_author_aliases(author_id)
        .expect("Failed to list aliases");
    assert_eq!(aliases.len(), 1);
    assert_eq!(aliases[0].link, None);
}

#[test]
fn test_import_author_duplicate_aliases_in_same_import() {
    let (manager, platform_id) = manager_with_platform();

    // Create aliases with same source and platform
    let alias1 = UnsyncAlias::new(platform_id, "duplicate_source".to_string())
        .link("http://link1.com".to_string());
    let alias2 = UnsyncAlias::new(platform_id, "duplicate_source".to_string())
        .link("http://link2.com".to_string());

    let author = UnsyncAuthor::new("Duplicate Test".to_string()).aliases(vec![alias1, alias2]);

    let author_id = manager
        .import_author(author)
        .expect("Failed to import author with duplicate aliases");

    // Should handle duplicates gracefully (database constraint should handle this)
    let stored_author = manager.get_author(author_id).expect("Failed to get author");
    assert_eq!(stored_author.name, "Duplicate Test");
}

#[test]
fn test_import_author_preserves_existing_aliases() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    // Create existing author with aliases
    let existing_author_id = manager
        .add_author("Existing Author".to_string(), Some(Utc::now()))
        .expect("Failed to add author");

    let existing_aliases = vec![
        ("old_alias1".to_string(), platform_id, None),
        (
            "old_alias2".to_string(),
            platform_id,
            Some("http://old.com".to_string()),
        ),
    ];
    manager
        .add_author_aliases(existing_author_id, existing_aliases)
        .expect("Failed to add existing aliases");

    // Import with new aliases
    let new_alias = UnsyncAlias::new(platform_id, "old_alias1".to_string()); // Same as existing
    let another_new_alias = UnsyncAlias::new(platform_id, "new_alias".to_string());

    let author =
        UnsyncAuthor::new("Updated Author".to_string()).aliases(vec![new_alias, another_new_alias]);

    let author_id = manager
        .import_author(author)
        .expect("Failed to import author");

    assert_eq!(author_id, existing_author_id);

    // Verify all aliases exist (old + new)
    let aliases = manager
        .list_author_aliases(author_id)
        .expect("Failed to list aliases");
    assert_eq!(aliases.len(), 3);

    let alias_sources: Vec<String> = aliases.iter().map(|a| a.source.clone()).collect();
    assert!(alias_sources.contains(&"old_alias1".to_string()));
    assert!(alias_sources.contains(&"old_alias2".to_string()));
    assert!(alias_sources.contains(&"new_alias".to_string()));
}

// Helper function to create a manager with a platform
fn manager_with_platform() -> (PostArchiverManager, crate::PlatformId) {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");
    (manager, platform_id)
}
