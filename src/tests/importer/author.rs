use crate::{
    importer::{UnsyncAlias, UnsyncAuthor},
    manager::platform::PlatformIdOrRaw,
    manager::PostArchiverManager,
};
use chrono::{Duration, TimeZone, Utc};

#[test]
fn test_import_author() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author with object-oriented API
    let author = UnsyncAuthor::new("octocat".to_string())
        .aliases(vec![UnsyncAlias::new(
            &PlatformIdOrRaw::Raw("github".to_string()),
            "octocat",
        )])
        .sync(&manager)
        .unwrap();

    // Verify the author was created
    let saved_author = manager.get_author(&author.id).unwrap();
    assert_eq!(saved_author.name, "octocat");
}

#[test]
fn test_functional_import_author() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let author = UnsyncAuthor::new("octocat".to_string()).aliases(vec![UnsyncAlias::new(
        &PlatformIdOrRaw::Raw("github".to_string()),
        "octocat",
    )]);

    // Import the author using functional API
    let id = manager.import_author(&author).unwrap();

    // Get the author
    let saved_author = manager.get_author(&id).unwrap();
    assert_eq!(saved_author.name, "octocat");
}

#[test]
fn test_functional_import_author_by_create() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let author = UnsyncAuthor::new("octocat".to_string());

    // Import the author by create
    let id = manager.import_author_by_create(&author).unwrap();

    // Get the author
    let saved_author = manager.get_author(&id).unwrap();
    assert_eq!(saved_author.name, "octocat");
}

#[test]
fn test_functional_import_author_by_update() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let author = UnsyncAuthor::new("octocat".to_string());

    // Import the author first
    let id = manager.import_author_by_create(&author).unwrap();

    // Update the author
    let updated_author = author.name("octocatdog".to_string());
    let updated_id = manager
        .import_author_by_update(id, &updated_author)
        .unwrap();

    // Verify update
    assert_eq!(id, updated_id);
    let saved_author = manager.get_author(&id).unwrap();
    assert_eq!(saved_author.name, "octocatdog");
}

#[test]
fn test_functional_get_author() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create and import an author
    let author = UnsyncAuthor::new("test_author".to_string());
    let id = manager.import_author_by_create(&author).unwrap();

    // Test get_author functional API
    let retrieved_author = manager.get_author(&id).unwrap();
    assert_eq!(retrieved_author.name, "test_author");
}

#[test]
fn test_functional_get_author_aliases() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create and import an author with aliases
    let author = UnsyncAuthor::new("test_author".to_string()).aliases(vec![UnsyncAlias::new(
        &PlatformIdOrRaw::Raw("github".to_string()),
        "test",
    )]);
    let id = manager.import_author(&author).unwrap();

    // Test get_author_aliases functional API
    let aliases = manager.get_author_aliases(&id).unwrap();
    assert!(!aliases.is_empty());
}

#[test]
fn test_functional_set_author_name() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create and import an author
    let author = UnsyncAuthor::new("original_name".to_string());
    let id = manager.import_author_by_create(&author).unwrap();

    // Test set_author_name functional API
    let new_name = "updated_name";
    manager.set_author_name(&id, new_name).unwrap();

    let updated_author = manager.get_author(&id).unwrap();
    assert_eq!(updated_author.name, new_name);
}

#[test]
fn test_functional_set_author_updated() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create and import an author
    let author = UnsyncAuthor::new("test_author".to_string());
    let id = manager.import_author_by_create(&author).unwrap();

    // Test set_author_updated functional API
    let new_time = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
    manager.set_author_updated(id, &new_time).unwrap();

    let updated_author = manager.get_author(&id).unwrap();
    assert_eq!(updated_author.updated, new_time);
}

#[test]
fn test_functional_set_author_thumb() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create and import an author
    let author = UnsyncAuthor::new("test_author".to_string());
    let id = manager.import_author_by_create(&author).unwrap();

    // Test set_author_thumb functional API (with None)
    manager.set_author_thumb(id, None).unwrap();

    let updated_author = manager.get_author(&id).unwrap();
    assert_eq!(updated_author.thumb, None);
}
