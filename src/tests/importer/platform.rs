//! Platform importer tests
//!
//! Tests for platform import functionality including
//! creation and deduplication.

use crate::manager::PostArchiverManager;

#[test]
fn test_import_platform_new() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_name = "Test Platform".to_string();

    let platform_id = manager
        .import_platform(platform_name.clone())
        .expect("Failed to import new platform");

    assert!(platform_id.raw() > 0);

    // Verify the platform was created
    let platform = manager
        .get_platform(&platform_id)
        .expect("Failed to get platform");

    assert_eq!(platform.name, platform_name);
    assert_eq!(platform.id, platform_id);
}

#[test]
fn test_import_platform_existing() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_name = "Existing Platform".to_string();

    // First, add a platform manually
    let existing_platform_id = manager
        .add_platform(platform_name.clone())
        .expect("Failed to add existing platform");

    // Import the same platform
    let imported_platform_id = manager
        .import_platform(platform_name.clone())
        .expect("Failed to import existing platform");

    // Should return the same ID
    assert_eq!(existing_platform_id, imported_platform_id);

    // Verify only two platforms exist in the database (including the default 'unknown')
    let platforms = manager.list_platforms().expect("Failed to list platforms");
    assert_eq!(platforms.len(), 2);

    // Find our platform (not the default 'unknown' one)
    let our_platform = platforms.iter().find(|p| p.name == platform_name).unwrap();
    assert_eq!(our_platform.name, platform_name);
}

#[test]
fn test_import_platform_multiple_different() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let platform1_name = "Platform 1".to_string();
    let platform2_name = "Platform 2".to_string();

    let platform1_id = manager
        .import_platform(platform1_name.clone())
        .expect("Failed to import platform 1");

    let platform2_id = manager
        .import_platform(platform2_name.clone())
        .expect("Failed to import platform 2");

    // Should be different IDs
    assert_ne!(platform1_id, platform2_id);

    // Verify both platforms exist (plus the default 'unknown')
    let platforms = manager.list_platforms().expect("Failed to list platforms");
    assert_eq!(platforms.len(), 3);

    let platform_names: Vec<String> = platforms.iter().map(|p| p.name.clone()).collect();
    assert!(platform_names.contains(&platform1_name));
    assert!(platform_names.contains(&platform2_name));
}

#[test]
fn test_import_platform_case_insensitive() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let platform1_id = manager
        .import_platform("Twitter".to_string())
        .expect("Failed to import Twitter");

    let platform2_id = manager
        .import_platform("twitter".to_string())
        .expect("Failed to import twitter (lowercase)");

    // Should be the same ID (case insensitive due to COLLATE NOCASE)
    assert_eq!(platform1_id, platform2_id);

    let platforms = manager.list_platforms().expect("Failed to list platforms");
    assert_eq!(platforms.len(), 2); // 'unknown' + our platform
}

#[test]
fn test_import_platform_empty_string() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let platform_id = manager
        .import_platform("".to_string())
        .expect("Failed to import empty platform");

    assert!(platform_id.raw() > 0);

    let platform = manager
        .get_platform(&platform_id)
        .expect("Failed to get empty platform");
    assert_eq!(platform.name, "");
}

#[test]
fn test_import_platform_special_characters() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let special_names = vec![
        "Platform-with-dashes".to_string(),
        "Platform with spaces".to_string(),
        "Platform_with_underscores".to_string(),
        "Platform.with.dots".to_string(),
        "Platform@with@symbols".to_string(),
        "Platform🚀with🚀emojis".to_string(),
    ];

    for name in special_names {
        let platform_id = manager
            .import_platform(name.clone())
            .expect(&format!("Failed to import platform: {}", name));

        let platform = manager
            .get_platform(&platform_id)
            .expect("Failed to get platform");
        assert_eq!(platform.name, name);
    }

    let platforms = manager.list_platforms().expect("Failed to list platforms");
    assert_eq!(platforms.len(), 7); // 6 special platforms + 'unknown'
}

#[test]
fn test_import_platform_long_name() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let long_name = "A".repeat(1000); // Very long platform name

    let platform_id = manager
        .import_platform(long_name.clone())
        .expect("Failed to import platform with long name");

    let platform = manager
        .get_platform(&platform_id)
        .expect("Failed to get platform");
    assert_eq!(platform.name, long_name);
}

#[test]
fn test_import_platform_unicode() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let unicode_names = vec![
        "微博".to_string(),        // Chinese
        "ツイッター".to_string(),  // Japanese
        "Твиттер".to_string(),     // Russian
        "تويتر".to_string(),       // Arabic
        "🐦Twitter🐦".to_string(), // With emojis
    ];

    for name in unicode_names {
        let platform_id = manager
            .import_platform(name.clone())
            .expect(&format!("Failed to import unicode platform: {}", name));

        let platform = manager
            .get_platform(&platform_id)
            .expect("Failed to get unicode platform");
        assert_eq!(platform.name, name);
    }

    let platforms = manager.list_platforms().expect("Failed to list platforms");
    assert_eq!(platforms.len(), 6); // 5 unicode platforms + 'unknown'
}

#[test]
fn test_import_platform_whitespace_variations() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let whitespace_names = vec![
        " Twitter ".to_string(),   // Leading/trailing spaces
        "\tTwitter\t".to_string(), // Tabs
        "\nTwitter\n".to_string(), // Newlines
        "Twit ter".to_string(),    // Space in middle
    ];

    for name in whitespace_names {
        let platform_id = manager
            .import_platform(name.clone())
            .expect(&format!("Failed to import platform: '{}'", name));

        let platform = manager
            .get_platform(&platform_id)
            .expect("Failed to get platform");
        assert_eq!(platform.name, name);
    }

    let platforms = manager.list_platforms().expect("Failed to list platforms");
    assert_eq!(platforms.len(), 5); // 4 whitespace variations + 'unknown'
}

#[test]
fn test_import_platform_cache_behavior() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_name = "Cached Platform".to_string();

    // First import should create and cache
    let platform_id1 = manager
        .import_platform(platform_name.clone())
        .expect("Failed to import platform first time");

    // Second import should use cache
    let platform_id2 = manager
        .import_platform(platform_name.clone())
        .expect("Failed to import platform second time");

    assert_eq!(platform_id1, platform_id2);

    // Should have two platforms (our platform + 'unknown')
    let platforms = manager.list_platforms().expect("Failed to list platforms");
    assert_eq!(platforms.len(), 2);
}

#[test]
fn test_import_platform_with_transaction() {
    let mut manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_name = "Transaction Platform".to_string();

    let tx = manager.transaction().expect("Failed to start transaction");

    let platform_id = tx
        .import_platform(platform_name.clone())
        .expect("Failed to import platform in transaction");

    // Verify platform exists in transaction
    let platform = tx
        .get_platform(&platform_id)
        .expect("Failed to get platform in transaction");
    assert_eq!(platform.name, platform_name);

    tx.commit().expect("Failed to commit transaction");

    // Verify platform still exists after commit
    let platform_after_commit = manager
        .get_platform(&platform_id)
        .expect("Failed to get platform after commit");
    assert_eq!(platform_after_commit.name, platform_name);
}

#[test]
fn test_import_platform_concurrent_behavior() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_name = "Concurrent Platform".to_string();

    // Simulate multiple imports of the same platform
    let mut ids = Vec::new();
    for _ in 0..10 {
        let id = manager
            .import_platform(platform_name.clone())
            .expect("Failed to import platform");
        ids.push(id);
    }

    // All should be the same ID
    for id in ids.iter() {
        assert_eq!(*id, ids[0]);
    }

    // Should have two platforms (our platform + 'unknown')
    let platforms = manager.list_platforms().expect("Failed to list platforms");
    assert_eq!(platforms.len(), 2);
}

#[test]
fn test_import_platform_comparison_with_manual_add() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Add platform manually
    let manual_id = manager
        .add_platform("Manual Platform".to_string())
        .expect("Failed to add platform manually");

    // Import same platform
    let imported_id = manager
        .import_platform("Manual Platform".to_string())
        .expect("Failed to import existing platform");

    // Should be the same
    assert_eq!(manual_id, imported_id);

    let platforms = manager.list_platforms().expect("Failed to list platforms");
    assert_eq!(platforms.len(), 2); // Manual platform + 'unknown'
}
