use crate::{manager::PostArchiverManager, COLLECTION_CATEGORY};

#[test]
fn test_import_tag() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Import a tag
    let tag_id = manager.import_tag(COLLECTION_CATEGORY, "example").unwrap();

    // Import the same tag again
    let tag_id2 = manager.import_tag(COLLECTION_CATEGORY, "example").unwrap();
    assert_eq!(tag_id, tag_id2);

    // Import the new tag
    let tag_id3 = manager.import_tag(COLLECTION_CATEGORY, "example2").unwrap();
    assert_ne!(tag_id, tag_id3);

    // Import multiple tags
    let tags = vec![
        (COLLECTION_CATEGORY, "tag1"),
        (COLLECTION_CATEGORY, "tag2"),
        (COLLECTION_CATEGORY, "tag3"),
    ];
    let tag_ids = manager.import_tags(&tags).unwrap();

    assert_eq!(tag_ids.len(), 3);

    // Import same tags again
    let tag_ids2 = manager.import_tags(&tags).unwrap();
    assert_eq!(tag_ids, tag_ids2);
}
