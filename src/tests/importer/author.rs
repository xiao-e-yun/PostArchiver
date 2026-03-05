//! Author importer tests
//!
//! Tests for author import functionality including creation,
//! updating existing authors, and alias management.

use crate::{
    importer::author::{UnsyncAlias, UnsyncAuthor},
    manager::PostArchiverManager,
    tests::helpers,
};
use chrono::Utc;

#[test]
fn test_import_new_author() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".to_string());

    let alias = UnsyncAlias::new(platform_id, "test_author".to_string())
        .link("http://example.com/test_author".to_string());

    let unsync_author = UnsyncAuthor::new("Test Author".to_string())
        .aliases(vec![alias])
        .updated(Some(Utc::now()));

    let author_id = manager
        .import_author(unsync_author)
        .expect("Failed to import author");

    assert!(author_id.raw() > 0);

    let author = helpers::get_author(&manager, author_id);
    assert_eq!(author.name, "Test Author");

    let aliases = helpers::list_author_aliases(&manager, author_id);
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
    let platform_id = helpers::add_platform(&manager, "Test Platform".to_string());

    let existing_author_id =
        helpers::add_author(&manager, "Original Name".to_string(), Some(Utc::now()));

    helpers::add_author_aliases(
        &manager,
        existing_author_id,
        vec![("existing_alias".to_string(), platform_id, None)],
    );

    let alias = UnsyncAlias::new(platform_id, "existing_alias".to_string());
    let new_alias = UnsyncAlias::new(platform_id, "new_alias".to_string());

    let unsync_author = UnsyncAuthor::new("Updated Name".to_string())
        .aliases(vec![alias, new_alias])
        .updated(Some(Utc::now()));

    let author_id = manager
        .import_author(unsync_author)
        .expect("Failed to import existing author");

    assert_eq!(author_id, existing_author_id);

    let author = helpers::get_author(&manager, author_id);
    assert_eq!(author.name, "Updated Name");

    let aliases = helpers::list_author_aliases(&manager, author_id);
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

    let author_id = manager
        .import_author(author)
        .expect("Failed to import author");

    let stored_author = helpers::get_author(&manager, author_id);
    assert_eq!(stored_author.name, "Updated Builder Test");

    let diff = (stored_author.updated - updated_time)
        .num_milliseconds()
        .abs();
    assert!(
        diff < 1000,
        "Updated timestamp should be close to expected time"
    );

    let aliases = helpers::list_author_aliases(&manager, author_id);
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

    let author = UnsyncAuthor::new("Alias Builder Test".to_string()).aliases(vec![alias]);

    let author_id = manager
        .import_author(author)
        .expect("Failed to import author");

    let aliases = helpers::list_author_aliases(&manager, author_id);
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

    let stored_author = helpers::get_author(&manager, author_id);
    assert_eq!(stored_author.name, "Sync Test Author");
}

#[test]
fn test_import_author_no_aliases() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let author = UnsyncAuthor::new("No Aliases Author".to_string()).updated(Some(Utc::now()));

    let author_id = manager
        .import_author(author)
        .expect("Failed to import author without aliases");

    let stored_author = helpers::get_author(&manager, author_id);
    assert_eq!(stored_author.name, "No Aliases Author");

    let aliases = helpers::list_author_aliases(&manager, author_id);
    assert_eq!(aliases.len(), 0);
}

#[test]
fn test_import_author_multiple_aliases() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform1_id = helpers::add_platform(&manager, "Platform 1".to_string());
    let platform2_id = helpers::add_platform(&manager, "Platform 2".to_string());

    let alias1 = UnsyncAlias::new(platform1_id, "author_p1".to_string())
        .link("http://platform1.com/author".to_string());
    let alias2 = UnsyncAlias::new(platform2_id, "author_p2".to_string())
        .link("http://platform2.com/author".to_string());

    let author =
        UnsyncAuthor::new("Multi Platform Author".to_string()).aliases(vec![alias1, alias2]);

    let author_id = manager
        .import_author(author)
        .expect("Failed to import author with multiple aliases");

    let aliases = helpers::list_author_aliases(&manager, author_id);
    assert_eq!(aliases.len(), 2);

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
    let platform_id = helpers::add_platform(&manager, "Test Platform".to_string());

    let old_time = chrono::DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let existing_author_id =
        helpers::add_author(&manager, "Test Author".to_string(), Some(old_time));

    helpers::add_author_aliases(
        &manager,
        existing_author_id,
        vec![("test_alias".to_string(), platform_id, None)],
    );

    let new_time = Utc::now();
    let alias = UnsyncAlias::new(platform_id, "test_alias".to_string());
    let author = UnsyncAuthor::new("Test Author".to_string())
        .aliases(vec![alias])
        .updated(Some(new_time));

    let author_id = manager
        .import_author(author)
        .expect("Failed to import author");

    assert_eq!(author_id, existing_author_id);

    let stored_author = helpers::get_author(&manager, author_id);
    let diff = (stored_author.updated - new_time).num_milliseconds().abs();
    assert!(diff < 1000, "Updated timestamp should be close to new time");
}

#[test]
fn test_import_author_no_updated_timestamp() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".to_string());

    let old_time = chrono::DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let existing_author_id =
        helpers::add_author(&manager, "Test Author".to_string(), Some(old_time));

    helpers::add_author_aliases(
        &manager,
        existing_author_id,
        vec![("test_alias".to_string(), platform_id, None)],
    );

    let alias = UnsyncAlias::new(platform_id, "test_alias".to_string());
    let author = UnsyncAuthor::new("Updated Name".to_string()).aliases(vec![alias]);

    let author_id = manager
        .import_author(author)
        .expect("Failed to import author");

    assert_eq!(author_id, existing_author_id);

    let stored_author = helpers::get_author(&manager, author_id);
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

    let aliases = helpers::list_author_aliases(&manager, author_id);
    assert_eq!(aliases.len(), 1);
    assert_eq!(aliases[0].link, None);
}

#[test]
fn test_import_author_duplicate_aliases_in_same_import() {
    let (manager, platform_id) = manager_with_platform();

    let alias1 = UnsyncAlias::new(platform_id, "duplicate_source".to_string())
        .link("http://link1.com".to_string());
    let alias2 = UnsyncAlias::new(platform_id, "duplicate_source".to_string())
        .link("http://link2.com".to_string());

    let author = UnsyncAuthor::new("Duplicate Test".to_string()).aliases(vec![alias1, alias2]);

    let author_id = manager
        .import_author(author)
        .expect("Failed to import author with duplicate aliases");

    let stored_author = helpers::get_author(&manager, author_id);
    assert_eq!(stored_author.name, "Duplicate Test");
}

#[test]
fn test_import_author_preserves_existing_aliases() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".to_string());

    let existing_author_id =
        helpers::add_author(&manager, "Existing Author".to_string(), Some(Utc::now()));

    helpers::add_author_aliases(
        &manager,
        existing_author_id,
        vec![
            ("old_alias1".to_string(), platform_id, None),
            (
                "old_alias2".to_string(),
                platform_id,
                Some("http://old.com".to_string()),
            ),
        ],
    );

    let new_alias = UnsyncAlias::new(platform_id, "old_alias1".to_string());
    let another_new_alias = UnsyncAlias::new(platform_id, "new_alias".to_string());

    let author =
        UnsyncAuthor::new("Updated Author".to_string()).aliases(vec![new_alias, another_new_alias]);

    let author_id = manager
        .import_author(author)
        .expect("Failed to import author");

    assert_eq!(author_id, existing_author_id);

    let aliases = helpers::list_author_aliases(&manager, author_id);
    assert_eq!(aliases.len(), 3);

    let alias_sources: Vec<String> = aliases.iter().map(|a| a.source.clone()).collect();
    assert!(alias_sources.contains(&"old_alias1".to_string()));
    assert!(alias_sources.contains(&"old_alias2".to_string()));
    assert!(alias_sources.contains(&"new_alias".to_string()));
}

// Helper function to create a manager with a platform
fn manager_with_platform() -> (PostArchiverManager, crate::PlatformId) {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = helpers::add_platform(&manager, "Test Platform".to_string());
    (manager, platform_id)
}
