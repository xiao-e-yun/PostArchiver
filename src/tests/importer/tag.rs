//! Tag importer tests
//!
//! Tests for tag import functionality including
//! single and batch imports with deduplication.

use crate::{importer::tag::UnsyncTag, manager::PostArchiverManager, PlatformId};

#[test]
fn test_import_tag_new() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    let unsync_tag = UnsyncTag {
        name: "new_tag".to_string(),
        platform: Some(platform_id),
    };

    let tag_id = manager
        .import_tag(unsync_tag.clone())
        .expect("Failed to import tag");

    // Verify the tag was created
    let tag = manager
        .get_tag(&tag_id)
        .expect("Failed to get tag")
        .unwrap();
    assert_eq!(tag.name, unsync_tag.name);
    assert_eq!(tag.platform, unsync_tag.platform);
}

#[test]
fn test_import_tag_existing() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    // First, add a tag manually
    let existing_tag_id = manager
        .add_tag("existing_tag".to_string(), Some(platform_id))
        .expect("Failed to add existing tag");

    let unsync_tag = UnsyncTag {
        name: "existing_tag".to_string(),
        platform: Some(platform_id),
    };

    // Import the same tag
    let imported_tag_id = manager
        .import_tag(unsync_tag)
        .expect("Failed to import existing tag");

    // Should return the same ID
    assert_eq!(existing_tag_id, imported_tag_id);

    // Verify only one tag exists in the database
    let tags = manager.list_tags().expect("Failed to list tags");
    assert_eq!(tags.len(), 1);
}

#[test]
fn test_import_tag_no_platform() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let unsync_tag = UnsyncTag {
        name: "global_tag".to_string(),
        platform: None,
    };

    let tag_id = manager
        .import_tag(unsync_tag.clone())
        .expect("Failed to import tag without platform");

    let tag = manager
        .get_tag(&tag_id)
        .expect("Failed to get tag")
        .unwrap();
    assert_eq!(tag.name, unsync_tag.name);
    assert_eq!(tag.platform, None);
}

#[test]
fn test_import_tags_multiple_new() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    let unsync_tags = vec![
        UnsyncTag {
            name: "tag1".to_string(),
            platform: Some(platform_id),
        },
        UnsyncTag {
            name: "tag2".to_string(),
            platform: Some(platform_id),
        },
        UnsyncTag {
            name: "tag3".to_string(),
            platform: None,
        },
    ];

    let tag_ids = manager
        .import_tags(unsync_tags.clone())
        .expect("Failed to import multiple tags");

    assert_eq!(tag_ids.len(), 3);

    // Verify all tags were created
    let all_tags = manager.list_tags().expect("Failed to list tags");
    assert_eq!(all_tags.len(), 3);

    // Verify each tag has correct properties
    for (i, tag_id) in tag_ids.iter().enumerate() {
        let tag = manager.get_tag(tag_id).expect("Failed to get tag").unwrap();
        assert_eq!(tag.name, unsync_tags[i].name);
        assert_eq!(tag.platform, unsync_tags[i].platform);
    }
}

#[test]
fn test_import_tags_mixed_new_and_existing() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    // Add one tag manually first
    let existing_tag_id = manager
        .add_tag("existing_tag".to_string(), Some(platform_id))
        .expect("Failed to add existing tag");

    let unsync_tags = vec![
        UnsyncTag {
            name: "existing_tag".to_string(),
            platform: Some(platform_id),
        },
        UnsyncTag {
            name: "new_tag".to_string(),
            platform: Some(platform_id),
        },
    ];

    let tag_ids = manager
        .import_tags(unsync_tags)
        .expect("Failed to import mixed tags");

    assert_eq!(tag_ids.len(), 2);

    // First ID should match the existing tag
    assert_eq!(tag_ids[0], existing_tag_id);

    // Second ID should be new
    assert_ne!(tag_ids[1], existing_tag_id);

    // Verify total tags in database
    let all_tags = manager.list_tags().expect("Failed to list tags");
    assert_eq!(all_tags.len(), 2);
}

#[test]
fn test_import_tags_duplicates_in_batch() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    let unsync_tags = vec![
        UnsyncTag {
            name: "duplicate_tag".to_string(),
            platform: Some(platform_id),
        },
        UnsyncTag {
            name: "unique_tag".to_string(),
            platform: Some(platform_id),
        },
        UnsyncTag {
            name: "duplicate_tag".to_string(),
            platform: Some(platform_id),
        },
    ];

    let tag_ids = manager
        .import_tags(unsync_tags)
        .expect("Failed to import tags with duplicates");

    assert_eq!(tag_ids.len(), 3);

    // First and third should be the same ID (duplicates)
    assert_eq!(tag_ids[0], tag_ids[2]);

    // Second should be different
    assert_ne!(tag_ids[0], tag_ids[1]);

    // Verify only 2 unique tags in database
    let all_tags = manager.list_tags().expect("Failed to list tags");
    assert_eq!(all_tags.len(), 2);
}

#[test]
fn test_import_tags_different_names_different_platforms() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform1_id = manager
        .add_platform("Platform 1".to_string())
        .expect("Failed to add platform 1");
    let platform2_id = manager
        .add_platform("Platform 2".to_string())
        .expect("Failed to add platform 2");

    let unsync_tags = vec![
        UnsyncTag {
            name: "platform1_tag".to_string(),
            platform: Some(platform1_id),
        },
        UnsyncTag {
            name: "platform2_tag".to_string(),
            platform: Some(platform2_id),
        },
        UnsyncTag {
            name: "global_tag".to_string(),
            platform: None,
        },
    ];

    let tag_ids = manager
        .import_tags(unsync_tags)
        .expect("Failed to import tags with different names");

    assert_eq!(tag_ids.len(), 3);

    // All should be different IDs
    assert_ne!(tag_ids[0], tag_ids[1]);
    assert_ne!(tag_ids[0], tag_ids[2]);
    assert_ne!(tag_ids[1], tag_ids[2]);

    // Verify all 3 tags exist in database
    let all_tags = manager.list_tags().expect("Failed to list tags");
    assert_eq!(all_tags.len(), 3);
}

#[test]
fn test_import_tags_empty_list() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let empty_tags: Vec<UnsyncTag> = vec![];
    let tag_ids = manager
        .import_tags(empty_tags)
        .expect("Failed to import empty tag list");

    assert_eq!(tag_ids.len(), 0);

    let all_tags = manager.list_tags().expect("Failed to list tags");
    assert_eq!(all_tags.len(), 0);
}

#[test]
fn test_unsync_tag_creation() {
    let platform_id = PlatformId(1);

    let tag_with_platform = UnsyncTag {
        name: "test_tag".to_string(),
        platform: Some(platform_id),
    };

    let tag_without_platform = UnsyncTag {
        name: "global_tag".to_string(),
        platform: None,
    };

    assert_eq!(tag_with_platform.name, "test_tag");
    assert_eq!(tag_with_platform.platform, Some(platform_id));

    assert_eq!(tag_without_platform.name, "global_tag");
    assert_eq!(tag_without_platform.platform, None);
}

#[test]
fn test_unsync_tag_clone_and_equality() {
    let platform_id = PlatformId(1);

    let tag1 = UnsyncTag {
        name: "test_tag".to_string(),
        platform: Some(platform_id),
    };

    let tag2 = tag1.clone();
    let tag3 = UnsyncTag {
        name: "test_tag".to_string(),
        platform: Some(platform_id),
    };

    let tag4 = UnsyncTag {
        name: "different_tag".to_string(),
        platform: Some(platform_id),
    };

    // Test equality
    assert_eq!(tag1, tag2);
    assert_eq!(tag1, tag3);
    assert_ne!(tag1, tag4);

    // Test clone
    assert_eq!(tag1.name, tag2.name);
    assert_eq!(tag1.platform, tag2.platform);
}

#[test]
fn test_import_tag_with_transaction() {
    let mut manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    let tx = manager.transaction().expect("Failed to start transaction");

    let unsync_tag = UnsyncTag {
        name: "transaction_tag".to_string(),
        platform: Some(platform_id),
    };

    let tag_id = tx
        .import_tag(unsync_tag)
        .expect("Failed to import tag in transaction");

    // Verify tag exists in transaction
    let tag = tx.get_tag(&tag_id).expect("Failed to get tag").unwrap();
    assert_eq!(tag.name, "transaction_tag");

    tx.commit().expect("Failed to commit transaction");

    // Verify tag still exists after commit
    let tag_after_commit = manager
        .get_tag(&tag_id)
        .expect("Failed to get tag after commit")
        .unwrap();
    assert_eq!(tag_after_commit.name, "transaction_tag");
}

#[test]
fn test_import_tags_iterator_types() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    // Test with Vec
    let vec_tags = vec![UnsyncTag {
        name: "vec_tag".to_string(),
        platform: Some(platform_id),
    }];

    let vec_result = manager
        .import_tags(vec_tags)
        .expect("Failed to import from Vec");
    assert_eq!(vec_result.len(), 1);

    // Test with array
    let array_tags = [UnsyncTag {
        name: "array_tag".to_string(),
        platform: Some(platform_id),
    }];

    let array_result = manager
        .import_tags(array_tags)
        .expect("Failed to import from array");
    assert_eq!(array_result.len(), 1);

    // Test with iterator
    let iter_tags = (0..2).map(|i| UnsyncTag {
        name: format!("iter_tag_{}", i),
        platform: Some(platform_id),
    });

    let iter_result = manager
        .import_tags(iter_tags)
        .expect("Failed to import from iterator");
    assert_eq!(iter_result.len(), 2);
}
