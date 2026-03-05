//! Platform importer tests
//!
//! Tests for platform import functionality including
//! creation and deduplication.

use crate::{manager::PostArchiverManager, tests::helpers};

#[test]
fn test_import_platform_new() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_name = "Test Platform".to_string();

    let platform_id = manager
        .import_platform(platform_name.clone())
        .expect("Failed to import new platform");

    assert!(platform_id.raw() > 0);

    let platform = helpers::get_platform(&manager, platform_id);
    assert_eq!(platform.name, platform_name);
    assert_eq!(platform.id, platform_id);
}

#[test]
fn test_import_platform_existing() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_name = "Existing Platform".to_string();

    // First, add a platform manually
    let existing_platform_id = helpers::add_platform(&manager, platform_name.clone());

    // Import the same platform
    let imported_platform_id = manager
        .import_platform(platform_name.clone())
        .expect("Failed to import existing platform");

    // Should return the same ID
    assert_eq!(existing_platform_id, imported_platform_id);

    // Verify only two platforms exist in the database (including the default 'unknown')
    let platforms = helpers::list_platforms(&manager);
    assert_eq!(platforms.len(), 2);

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

    assert_ne!(platform1_id, platform2_id);

    let platforms = helpers::list_platforms(&manager);
    assert_eq!(platforms.len(), 3); // plus 'unknown'

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

    assert_eq!(platform1_id, platform2_id);

    let platforms = helpers::list_platforms(&manager);
    assert_eq!(platforms.len(), 2); // 'unknown' + our platform
}

#[test]
fn test_import_platform_empty_string() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let platform_id = manager
        .import_platform("".to_string())
        .expect("Failed to import empty platform");

    assert!(platform_id.raw() > 0);

    let platform = helpers::get_platform(&manager, platform_id);
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

        let platform = helpers::get_platform(&manager, platform_id);
        assert_eq!(platform.name, name);
    }

    let platforms = helpers::list_platforms(&manager);
    assert_eq!(platforms.len(), 7); // 6 special + 'unknown'
}

#[test]
fn test_import_platform_long_name() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let long_name = "A".repeat(1000);

    let platform_id = manager
        .import_platform(long_name.clone())
        .expect("Failed to import platform with long name");

    let platform = helpers::get_platform(&manager, platform_id);
    assert_eq!(platform.name, long_name);
}

#[test]
fn test_import_platform_unicode() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let unicode_names = vec![
        "微博".to_string(),
        "ツイッター".to_string(),
        "Твиттер".to_string(),
        "تويتر".to_string(),
        "🐦Twitter🐦".to_string(),
    ];

    for name in unicode_names {
        let platform_id = manager
            .import_platform(name.clone())
            .expect(&format!("Failed to import unicode platform: {}", name));

        let platform = helpers::get_platform(&manager, platform_id);
        assert_eq!(platform.name, name);
    }

    let platforms = helpers::list_platforms(&manager);
    assert_eq!(platforms.len(), 6); // 5 unicode + 'unknown'
}

#[test]
fn test_import_platform_whitespace_variations() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let whitespace_names = vec![
        " Twitter ".to_string(),
        "\tTwitter\t".to_string(),
        "\nTwitter\n".to_string(),
        "Twit ter".to_string(),
    ];

    for name in whitespace_names {
        let platform_id = manager
            .import_platform(name.clone())
            .expect(&format!("Failed to import platform: '{}'", name));

        let platform = helpers::get_platform(&manager, platform_id);
        assert_eq!(platform.name, name);
    }

    let platforms = helpers::list_platforms(&manager);
    assert_eq!(platforms.len(), 5); // 4 whitespace variations + 'unknown'
}

#[test]
fn test_import_platform_dedup_behavior() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_name = "Cached Platform".to_string();

    // First import should create
    let platform_id1 = manager
        .import_platform(platform_name.clone())
        .expect("Failed to import platform first time");

    // Second import should find existing
    let platform_id2 = manager
        .import_platform(platform_name.clone())
        .expect("Failed to import platform second time");

    assert_eq!(platform_id1, platform_id2);

    let platforms = helpers::list_platforms(&manager);
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

    tx.commit().expect("Failed to commit transaction");

    // Verify platform exists after commit
    let platform = helpers::get_platform(&manager, platform_id);
    assert_eq!(platform.name, platform_name);
}

#[test]
fn test_import_platform_concurrent_behavior() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_name = "Concurrent Platform".to_string();

    let mut ids = Vec::new();
    for _ in 0..10 {
        let id = manager
            .import_platform(platform_name.clone())
            .expect("Failed to import platform");
        ids.push(id);
    }

    for id in ids.iter() {
        assert_eq!(*id, ids[0]);
    }

    let platforms = helpers::list_platforms(&manager);
    assert_eq!(platforms.len(), 2);
}

#[test]
fn test_import_platform_comparison_with_manual_add() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Add platform via helpers (same as manual add)
    let manual_id = helpers::add_platform(&manager, "Manual Platform".to_string());

    // Import same platform
    let imported_id = manager
        .import_platform("Manual Platform".to_string())
        .expect("Failed to import existing platform");

    assert_eq!(manual_id, imported_id);

    let platforms = helpers::list_platforms(&manager);
    assert_eq!(platforms.len(), 2); // Manual platform + 'unknown'
}
