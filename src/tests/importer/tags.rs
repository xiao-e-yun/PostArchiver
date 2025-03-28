use crate::manager::PostArchiverManager;

#[test]
fn test_import_tag() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Import a tag
    let tag_id = manager.import_tag("example").unwrap();

    // Import the same tag again
    let tag_id2 = manager.import_tag("example").unwrap();
    assert_eq!(tag_id, tag_id2);

    // Import the new tag
    let tag_id3 = manager.import_tag("example2").unwrap();
    assert_ne!(tag_id, tag_id3);
}

#[test]
fn test_import_tags() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Import multiple tags
    let tags = vec!["tag1", "tag2", "tag3"];
    let tag_ids = manager.import_tags(&tags).unwrap();

    assert_eq!(tag_ids.len(), 3);

    // Import same tags again
    let tag_ids2 = manager.import_tags(&tags).unwrap();
    assert_eq!(tag_ids, tag_ids2);
}
