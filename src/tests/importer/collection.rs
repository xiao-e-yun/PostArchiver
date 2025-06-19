//! Collection importer tests
//!
//! Tests for collection import functionality including creation,
//! updating existing collections, and batch imports.

use crate::{importer::collection::UnsyncCollection, manager::PostArchiverManager};

#[test]
fn test_import_new_collection() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let unsync_collection = UnsyncCollection {
        name: "Test Collection".to_string(),
        source: "https://example.com/collection/1".to_string(),
    };

    let collection_id = manager
        .import_collection(unsync_collection.clone())
        .expect("Failed to import collection");

    assert!(collection_id.raw() > 0);

    // Verify the collection was created
    let collection = manager
        .get_collection(&collection_id)
        .expect("Failed to get collection")
        .expect("Collection should exist");

    assert_eq!(collection.name, unsync_collection.name);
    assert_eq!(collection.source, Some(unsync_collection.source));
}

#[test]
fn test_import_existing_collection() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let source = "https://example.com/existing_collection".to_string();

    // First, add a collection manually
    let existing_collection_id = manager
        .add_collection("Original Name".to_string(), Some(source.clone()), None)
        .expect("Failed to add existing collection");

    // Now import the same collection with updated name
    let unsync_collection = UnsyncCollection {
        name: "Updated Name".to_string(),
        source: source.clone(),
    };

    let imported_collection_id = manager
        .import_collection(unsync_collection)
        .expect("Failed to import existing collection");

    // Should return the same ID
    assert_eq!(existing_collection_id, imported_collection_id);

    // Verify the name was updated
    let collection = manager
        .get_collection(&existing_collection_id)
        .expect("Failed to get updated collection")
        .expect("Collection should exist");

    assert_eq!(collection.name, "Updated Name");
    assert_eq!(collection.source, Some(source));
}

#[test]
fn test_unsync_collection_new() {
    let name = "New Collection".to_string();
    let source = "https://example.com/new".to_string();

    let collection = UnsyncCollection::new(name.clone(), source.clone());

    assert_eq!(collection.name, name);
    assert_eq!(collection.source, source);
}

#[test]
fn test_unsync_collection_builder() {
    let collection = UnsyncCollection::new(
        "Original Name".to_string(),
        "https://example.com/original".to_string(),
    )
    .name("Updated Name".to_string())
    .source("https://example.com/updated".to_string());

    assert_eq!(collection.name, "Updated Name");
    assert_eq!(collection.source, "https://example.com/updated");
}

#[test]
fn test_unsync_collection_clone() {
    let original = UnsyncCollection {
        name: "Clone Test".to_string(),
        source: "https://example.com/clone".to_string(),
    };

    let cloned = original.clone();

    assert_eq!(original.name, cloned.name);
    assert_eq!(original.source, cloned.source);
}

#[test]
fn test_unsync_collection_equality() {
    let collection1 = UnsyncCollection {
        name: "Test Collection".to_string(),
        source: "https://example.com/test".to_string(),
    };

    let collection2 = UnsyncCollection {
        name: "Test Collection".to_string(),
        source: "https://example.com/test".to_string(),
    };

    let collection3 = UnsyncCollection {
        name: "Different Collection".to_string(),
        source: "https://example.com/test".to_string(),
    };

    assert_eq!(collection1, collection2);
    assert_ne!(collection1, collection3);
}

#[test]
fn test_unsync_collection_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let collection1 = UnsyncCollection {
        name: "Hash Test".to_string(),
        source: "https://example.com/hash".to_string(),
    };

    let collection2 = UnsyncCollection {
        name: "Hash Test".to_string(),
        source: "https://example.com/hash".to_string(),
    };

    let collection3 = UnsyncCollection {
        name: "Different Hash".to_string(),
        source: "https://example.com/hash".to_string(),
    };

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();
    let mut hasher3 = DefaultHasher::new();

    collection1.hash(&mut hasher1);
    collection2.hash(&mut hasher2);
    collection3.hash(&mut hasher3);

    let hash1 = hasher1.finish();
    let hash2 = hasher2.finish();
    let hash3 = hasher3.finish();

    // Same collections should have same hash
    assert_eq!(hash1, hash2);
    // Different collections should have different hash
    assert_ne!(hash1, hash3);
}

#[test]
fn test_import_collections_multiple() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let collections = vec![
        UnsyncCollection {
            name: "Collection 1".to_string(),
            source: "https://example.com/collection/1".to_string(),
        },
        UnsyncCollection {
            name: "Collection 2".to_string(),
            source: "https://example.com/collection/2".to_string(),
        },
        UnsyncCollection {
            name: "Collection 3".to_string(),
            source: "https://example.com/collection/3".to_string(),
        },
    ];

    let collection_ids = manager
        .import_collections(collections.clone())
        .expect("Failed to import multiple collections");

    assert_eq!(collection_ids.len(), 3);

    // Verify all collections were created
    for (i, collection_id) in collection_ids.iter().enumerate() {
        let collection = manager
            .get_collection(collection_id)
            .expect("Failed to get collection")
            .expect("Collection should exist");

        assert_eq!(collection.name, collections[i].name);
        assert_eq!(collection.source, Some(collections[i].source.clone()));
    }
}

#[test]
fn test_import_collections_mixed_new_and_existing() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let existing_source = "https://example.com/existing".to_string();

    // Add one collection manually first
    let existing_collection_id = manager
        .add_collection(
            "Existing Collection".to_string(),
            Some(existing_source.clone()),
            None,
        )
        .expect("Failed to add existing collection");

    let collections = vec![
        UnsyncCollection {
            name: "Updated Existing Collection".to_string(),
            source: existing_source,
        },
        UnsyncCollection {
            name: "New Collection".to_string(),
            source: "https://example.com/new".to_string(),
        },
    ];

    let collection_ids = manager
        .import_collections(collections)
        .expect("Failed to import mixed collections");

    assert_eq!(collection_ids.len(), 2);

    // First ID should match the existing collection
    assert_eq!(collection_ids[0], existing_collection_id);

    // Second ID should be new
    assert_ne!(collection_ids[1], existing_collection_id);

    // Verify the existing collection was updated
    let updated_collection = manager
        .get_collection(&existing_collection_id)
        .expect("Failed to get updated collection")
        .expect("Collection should exist");
    assert_eq!(updated_collection.name, "Updated Existing Collection");

    // Verify total collections in database
    let all_collections = manager
        .list_collections()
        .expect("Failed to list collections");
    assert_eq!(all_collections.len(), 2);
}

#[test]
fn test_import_collections_duplicates_in_batch() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let collections = vec![
        UnsyncCollection {
            name: "Duplicate Collection".to_string(),
            source: "https://example.com/duplicate".to_string(),
        },
        UnsyncCollection {
            name: "Unique Collection".to_string(),
            source: "https://example.com/unique".to_string(),
        },
        UnsyncCollection {
            name: "Updated Duplicate Collection".to_string(),
            source: "https://example.com/duplicate".to_string(),
        },
    ];

    let collection_ids = manager
        .import_collections(collections)
        .expect("Failed to import collections with duplicates");

    assert_eq!(collection_ids.len(), 3);

    // First and third should be the same ID (duplicates)
    assert_eq!(collection_ids[0], collection_ids[2]);

    // Second should be different
    assert_ne!(collection_ids[0], collection_ids[1]);

    // Verify only 2 unique collections in database
    let all_collections = manager
        .list_collections()
        .expect("Failed to list collections");
    assert_eq!(all_collections.len(), 2);

    // Verify the duplicate was updated with the latest name
    let duplicate_collection = manager
        .get_collection(&collection_ids[0])
        .expect("Failed to get duplicate collection")
        .expect("Collection should exist");
    assert_eq!(duplicate_collection.name, "Updated Duplicate Collection");
}

#[test]
fn test_import_collections_empty_list() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let empty_collections: Vec<UnsyncCollection> = vec![];
    let collection_ids = manager
        .import_collections(empty_collections)
        .expect("Failed to import empty collection list");

    assert_eq!(collection_ids.len(), 0);

    let all_collections = manager
        .list_collections()
        .expect("Failed to list collections");
    assert_eq!(all_collections.len(), 0);
}

#[test]
fn test_unsync_collection_debug() {
    let collection = UnsyncCollection {
        name: "Debug Test".to_string(),
        source: "https://example.com/debug".to_string(),
    };

    let debug_string = format!("{:?}", collection);

    assert!(debug_string.contains("UnsyncCollection"));
    assert!(debug_string.contains("Debug Test"));
    assert!(debug_string.contains("https://example.com/debug"));
}

#[test]
fn test_import_collection_with_transaction() {
    let mut manager = PostArchiverManager::open_in_memory().unwrap();

    let tx = manager.transaction().expect("Failed to start transaction");

    let collection = UnsyncCollection {
        name: "Transaction Collection".to_string(),
        source: "https://example.com/transaction".to_string(),
    };

    let collection_id = tx
        .import_collection(collection)
        .expect("Failed to import collection in transaction");

    // Verify collection exists in transaction
    let stored_collection = tx
        .get_collection(&collection_id)
        .expect("Failed to get collection in transaction")
        .expect("Collection should exist");
    assert_eq!(stored_collection.name, "Transaction Collection");

    tx.commit().expect("Failed to commit transaction");

    // Verify collection still exists after commit
    let stored_collection_after_commit = manager
        .get_collection(&collection_id)
        .expect("Failed to get collection after commit")
        .expect("Collection should exist");
    assert_eq!(
        stored_collection_after_commit.name,
        "Transaction Collection"
    );
}

#[test]
fn test_import_collections_iterator_types() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Test with Vec
    let vec_collections = vec![UnsyncCollection {
        name: "Vec Collection".to_string(),
        source: "https://example.com/vec".to_string(),
    }];

    let vec_result = manager
        .import_collections(vec_collections)
        .expect("Failed to import from Vec");
    assert_eq!(vec_result.len(), 1);

    // Test with array
    let array_collections = [UnsyncCollection {
        name: "Array Collection".to_string(),
        source: "https://example.com/array".to_string(),
    }];

    let array_result = manager
        .import_collections(array_collections)
        .expect("Failed to import from array");
    assert_eq!(array_result.len(), 1);

    // Test with iterator
    let iter_collections = (0..2).map(|i| UnsyncCollection {
        name: format!("Iter Collection {}", i),
        source: format!("https://example.com/iter/{}", i),
    });

    let iter_result = manager
        .import_collections(iter_collections)
        .expect("Failed to import from iterator");
    assert_eq!(iter_result.len(), 2);
}

#[test]
fn test_import_collection_sets_no_thumbnail() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let collection = UnsyncCollection {
        name: "No Thumbnail Collection".to_string(),
        source: "https://example.com/no_thumb".to_string(),
    };

    let collection_id = manager
        .import_collection(collection)
        .expect("Failed to import collection");

    let stored_collection = manager
        .get_collection(&collection_id)
        .expect("Failed to get collection")
        .expect("Collection should exist");

    // Should have no thumbnail when imported
    assert_eq!(stored_collection.thumb, None);
}

#[test]
fn test_import_collection_name_update_only() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let source = "https://example.com/name_update".to_string();

    // Create initial collection
    let collection_id = manager
        .add_collection("Initial Name".to_string(), Some(source.clone()), None)
        .expect("Failed to add initial collection");

    // Import with updated name multiple times
    let names = vec!["First Update", "Second Update", "Final Update"];

    for name in names {
        let updated_collection = UnsyncCollection {
            name: name.to_string(),
            source: source.clone(),
        };

        let same_collection_id = manager
            .import_collection(updated_collection)
            .expect("Failed to import collection update");

        assert_eq!(collection_id, same_collection_id);

        let stored_collection = manager
            .get_collection(&collection_id)
            .expect("Failed to get collection")
            .expect("Collection should exist");
        assert_eq!(stored_collection.name, name);
    }
}
