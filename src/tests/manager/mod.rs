use chrono::{TimeZone, Utc};
use tempdir::TempDir;

use crate::manager::PostArchiverManager;

#[test]
fn test_open_manager() {
    let mut manager =
        PostArchiverManager::open_in_memory().expect("Failed to open in-memory manager");
    let tx = manager.transaction().expect("Failed to create transaction");
    tx.commit().expect("Failed to commit transaction");

    let temp = TempDir::new("post_archiver_test").expect("Failed to create temporary directory");
    let path = temp.path();

    assert!(PostArchiverManager::open(path)
        .expect("Failed to open manager when not exists")
        .is_none());
    PostArchiverManager::create(path).expect("Failed to create manager");
    assert!(PostArchiverManager::open(path)
        .expect("Failed to open manager")
        .is_some());
    assert!(PostArchiverManager::open_uncheck(path)
        .expect("Failed to open manager without checking")
        .is_some());
}

#[test]
fn test_functional_get_feature() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let value = manager
        .get_feature("example")
        .expect("Failed to get feature");
    assert_eq!(value, 0);

    let (value, extra) = manager
        .get_feature_with_extra("example_extra")
        .expect("Failed to get feature with extra");
    assert_eq!(value, 0);
    assert!(extra.is_empty());
}

#[test]
fn test_functional_set_feature() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    manager.set_feature("test_feature", 42);
    let value = manager.get_feature("test_feature").unwrap();
    assert_eq!(value, 42);
}

#[test]
fn test_functional_check_author() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Check when the database is empty
    assert!(manager
        .check_author(&[(&"github", "octocat")])
        .expect("Failed to check author")
        .is_none());
}

#[test]
fn test_functional_check_post() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    assert!(manager.check_post("https://example.com").unwrap().is_none());

    let updated = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();
    assert!(manager
        .check_post_with_updated("https://example.com", &updated)
        .expect("Failed to check post with updated")
        .is_none());
}

#[test]
fn test_functional_check_file_meta() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    assert!(manager
        .check_file_meta(crate::PostId(1), "thumb.png")
        .expect("Failed to check file meta")
        .is_none());
}
