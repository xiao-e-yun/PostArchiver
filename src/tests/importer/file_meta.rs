//! File metadata importer tests
//!
//! Tests for file metadata import functionality including creation,
//! updating existing file metadata, and handling extra data.

use crate::{importer::UnsyncFileMeta, manager::PostArchiverManager};
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;

#[test]
fn test_import_new_file_meta() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create a post first
    let post_id = manager
        .add_post(
            "Test Post".to_string(),
            None,
            None,
            Some(Utc::now()),
            Some(Utc::now()),
        )
        .expect("Failed to add post");

    let mut extra = HashMap::new();
    extra.insert("width".to_string(), Value::Number(1920.into()));
    extra.insert("height".to_string(), Value::Number(1080.into()));

    let unsync_file_meta = UnsyncFileMeta {
        filename: "test_image.jpg".to_string(),
        mime: "image/jpeg".to_string(),
        extra,
    };

    let file_meta_id = manager
        .import_file_meta(post_id, unsync_file_meta.clone())
        .expect("Failed to import file meta");

    assert!(file_meta_id.raw() > 0);

    // Verify the file meta was created
    let file_meta = manager
        .get_file_meta(&file_meta_id)
        .expect("Failed to get file meta");

    assert_eq!(file_meta.filename, unsync_file_meta.filename);
    assert_eq!(file_meta.mime, unsync_file_meta.mime);
    assert_eq!(file_meta.post, post_id);

    // Verify extra data
    assert_eq!(
        file_meta.extra.get("width"),
        Some(&Value::Number(1920.into()))
    );
    assert_eq!(
        file_meta.extra.get("height"),
        Some(&Value::Number(1080.into()))
    );
}

#[test]
fn test_import_existing_file_meta() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let post_id = manager
        .add_post(
            "Test Post".to_string(),
            None,
            None,
            Some(Utc::now()),
            Some(Utc::now()),
        )
        .expect("Failed to add post");

    let filename = "existing_file.png".to_string();
    let original_mime = "image/png".to_string();

    // First, add a file meta manually
    let mut original_extra = HashMap::new();
    original_extra.insert("version".to_string(), Value::String("1.0".to_string()));

    let existing_file_meta_id = manager
        .add_file_meta(
            post_id,
            filename.clone(),
            original_mime.clone(),
            original_extra.clone(),
        )
        .expect("Failed to add existing file meta");

    // Now import the same file with updated extra data
    let mut updated_extra = HashMap::new();
    updated_extra.insert("version".to_string(), Value::String("2.0".to_string()));
    updated_extra.insert(
        "description".to_string(),
        Value::String("Updated file".to_string()),
    );

    let unsync_file_meta = UnsyncFileMeta {
        filename: filename.clone(),
        mime: original_mime.clone(),
        extra: updated_extra.clone(),
    };

    let imported_file_meta_id = manager
        .import_file_meta(post_id, unsync_file_meta)
        .expect("Failed to import existing file meta");

    // Should return the same ID
    assert_eq!(existing_file_meta_id, imported_file_meta_id);

    // Verify the extra data was updated
    let file_meta = manager
        .get_file_meta(&existing_file_meta_id)
        .expect("Failed to get updated file meta");

    assert_eq!(file_meta.filename, filename);
    assert_eq!(file_meta.mime, original_mime);
    assert_eq!(
        file_meta.extra.get("version"),
        Some(&Value::String("2.0".to_string()))
    );
    assert_eq!(
        file_meta.extra.get("description"),
        Some(&Value::String("Updated file".to_string()))
    );
}

#[test]
fn test_unsync_file_meta_new() {
    let filename = "test_file.txt".to_string();
    let mime = "text/plain".to_string();

    let file_meta = UnsyncFileMeta::new(filename.clone(), mime.clone());

    assert_eq!(file_meta.filename, filename);
    assert_eq!(file_meta.mime, mime);
    assert!(file_meta.extra.is_empty());
}

#[test]
fn test_unsync_file_meta_with_extra() {
    let mut extra = HashMap::new();
    extra.insert("size".to_string(), Value::Number(1024.into()));
    extra.insert("encoding".to_string(), Value::String("UTF-8".to_string()));

    let file_meta = UnsyncFileMeta::new("document.txt".to_string(), "text/plain".to_string())
        .extra(extra.clone());

    assert_eq!(file_meta.filename, "document.txt");
    assert_eq!(file_meta.mime, "text/plain");
    assert_eq!(file_meta.extra, extra);
}

#[test]
fn test_unsync_file_meta_clone() {
    let mut extra = HashMap::new();
    extra.insert("test".to_string(), Value::Bool(true));

    let original = UnsyncFileMeta {
        filename: "clone_test.pdf".to_string(),
        mime: "application/pdf".to_string(),
        extra,
    };

    let cloned = original.clone();

    assert_eq!(original.filename, cloned.filename);
    assert_eq!(original.mime, cloned.mime);
    assert_eq!(original.extra, cloned.extra);
}

#[test]
fn test_unsync_file_meta_equality() {
    let extra1 = {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::String("value".to_string()));
        map
    };

    let extra2 = {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::String("value".to_string()));
        map
    };

    let file_meta1 = UnsyncFileMeta {
        filename: "test.jpg".to_string(),
        mime: "image/jpeg".to_string(),
        extra: extra1,
    };

    let file_meta2 = UnsyncFileMeta {
        filename: "test.jpg".to_string(),
        mime: "image/jpeg".to_string(),
        extra: extra2,
    };

    let file_meta3 = UnsyncFileMeta {
        filename: "different.jpg".to_string(),
        mime: "image/jpeg".to_string(),
        extra: HashMap::new(),
    };

    assert_eq!(file_meta1, file_meta2);
    assert_ne!(file_meta1, file_meta3);
}

#[test]
fn test_unsync_file_meta_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let file_meta1 = UnsyncFileMeta {
        filename: "hash_test.png".to_string(),
        mime: "image/png".to_string(),
        extra: {
            let mut map = HashMap::new();
            map.insert("ignored".to_string(), Value::String("in_hash".to_string()));
            map
        },
    };

    let file_meta2 = UnsyncFileMeta {
        filename: "hash_test.png".to_string(),
        mime: "image/png".to_string(),
        extra: {
            let mut map = HashMap::new();
            map.insert(
                "different".to_string(),
                Value::String("extra_data".to_string()),
            );
            map
        },
    };

    let file_meta3 = UnsyncFileMeta {
        filename: "different_file.png".to_string(),
        mime: "image/png".to_string(),
        extra: HashMap::new(),
    };

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();
    let mut hasher3 = DefaultHasher::new();

    file_meta1.hash(&mut hasher1);
    file_meta2.hash(&mut hasher2);
    file_meta3.hash(&mut hasher3);

    let hash1 = hasher1.finish();
    let hash2 = hasher2.finish();
    let hash3 = hasher3.finish();

    // Same filename and mime should have same hash (extra is not included in hash)
    assert_eq!(hash1, hash2);
    // Different filename should have different hash
    assert_ne!(hash1, hash3);
}

#[test]
fn test_import_file_meta_different_posts() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let post1_id = manager
        .add_post(
            "Post 1".to_string(),
            None,
            None,
            Some(Utc::now()),
            Some(Utc::now()),
        )
        .expect("Failed to add post 1");

    let post2_id = manager
        .add_post(
            "Post 2".to_string(),
            None,
            None,
            Some(Utc::now()),
            Some(Utc::now()),
        )
        .expect("Failed to add post 2");

    let filename = "shared_name.jpg".to_string();

    let file_meta1 = UnsyncFileMeta::new(filename.clone(), "image/jpeg".to_string());
    let file_meta2 = UnsyncFileMeta::new(filename.clone(), "image/jpeg".to_string());

    // Import same filename to different posts
    let file_meta1_id = manager
        .import_file_meta(post1_id, file_meta1)
        .expect("Failed to import file meta to post 1");

    let file_meta2_id = manager
        .import_file_meta(post2_id, file_meta2)
        .expect("Failed to import file meta to post 2");

    // Should create different file metas (different post IDs)
    assert_ne!(file_meta1_id, file_meta2_id);

    // Verify they belong to different posts
    let file1 = manager
        .get_file_meta(&file_meta1_id)
        .expect("Failed to get file 1");
    let file2 = manager
        .get_file_meta(&file_meta2_id)
        .expect("Failed to get file 2");

    assert_eq!(file1.post, post1_id);
    assert_eq!(file2.post, post2_id);
    assert_eq!(file1.filename, file2.filename);
}

#[test]
fn test_import_file_meta_empty_extra() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let post_id = manager
        .add_post(
            "Test Post".to_string(),
            None,
            None,
            Some(Utc::now()),
            Some(Utc::now()),
        )
        .expect("Failed to add post");

    let file_meta = UnsyncFileMeta {
        filename: "simple_file.txt".to_string(),
        mime: "text/plain".to_string(),
        extra: HashMap::new(),
    };

    let file_meta_id = manager
        .import_file_meta(post_id, file_meta)
        .expect("Failed to import file meta with empty extra");

    let stored_file_meta = manager
        .get_file_meta(&file_meta_id)
        .expect("Failed to get file meta");

    assert!(stored_file_meta.extra.is_empty());
}

#[test]
fn test_import_file_meta_complex_extra() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let post_id = manager
        .add_post(
            "Test Post".to_string(),
            None,
            None,
            Some(Utc::now()),
            Some(Utc::now()),
        )
        .expect("Failed to add post");

    let mut extra = HashMap::new();
    extra.insert(
        "metadata".to_string(),
        Value::Object({
            let mut obj = serde_json::Map::new();
            obj.insert("camera".to_string(), Value::String("Canon EOS".to_string()));
            obj.insert("iso".to_string(), Value::Number(800.into()));
            obj
        }),
    );
    extra.insert(
        "tags".to_string(),
        Value::Array(vec![
            Value::String("nature".to_string()),
            Value::String("photography".to_string()),
        ]),
    );

    let file_meta = UnsyncFileMeta {
        filename: "complex_photo.jpg".to_string(),
        mime: "image/jpeg".to_string(),
        extra,
    };

    let file_meta_id = manager
        .import_file_meta(post_id, file_meta.clone())
        .expect("Failed to import file meta with complex extra");

    let stored_file_meta = manager
        .get_file_meta(&file_meta_id)
        .expect("Failed to get file meta");

    assert_eq!(stored_file_meta.extra, file_meta.extra);
}

#[test]
fn test_import_file_meta_preserves_mime() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let post_id = manager
        .add_post(
            "Test Post".to_string(),
            None,
            None,
            Some(Utc::now()),
            Some(Utc::now()),
        )
        .expect("Failed to add post");

    let filename = "mime_test.dat".to_string();
    let original_mime = "application/octet-stream".to_string();

    // First import
    let file_meta1 = UnsyncFileMeta::new(filename.clone(), original_mime.clone());
    let file_meta_id = manager
        .import_file_meta(post_id, file_meta1)
        .expect("Failed to import first file meta");

    // Second import with different MIME type (should not change)
    let different_mime = "text/plain".to_string();
    let file_meta2 = UnsyncFileMeta::new(filename.clone(), different_mime);
    let same_file_meta_id = manager
        .import_file_meta(post_id, file_meta2)
        .expect("Failed to import second file meta");

    assert_eq!(file_meta_id, same_file_meta_id);

    let stored_file_meta = manager
        .get_file_meta(&file_meta_id)
        .expect("Failed to get file meta");

    // MIME should remain the original one
    assert_eq!(stored_file_meta.mime, original_mime);
}

#[test]
fn test_unsync_file_meta_debug() {
    let file_meta = UnsyncFileMeta::new("debug_test.log".to_string(), "text/plain".to_string());

    let debug_string = format!("{:?}", file_meta);

    assert!(debug_string.contains("UnsyncFileMeta"));
    assert!(debug_string.contains("debug_test.log"));
    assert!(debug_string.contains("text/plain"));
}

#[test]
fn test_import_file_meta_with_transaction() {
    let mut manager = PostArchiverManager::open_in_memory().unwrap();

    let post_id = manager
        .add_post(
            "Transaction Test Post".to_string(),
            None,
            None,
            Some(Utc::now()),
            Some(Utc::now()),
        )
        .expect("Failed to add post");

    let tx = manager.transaction().expect("Failed to start transaction");

    let file_meta =
        UnsyncFileMeta::new("transaction_file.txt".to_string(), "text/plain".to_string());

    let file_meta_id = tx
        .import_file_meta(post_id, file_meta)
        .expect("Failed to import file meta in transaction");

    // Verify file exists in transaction
    let stored_file_meta = tx
        .get_file_meta(&file_meta_id)
        .expect("Failed to get file meta in transaction");
    assert_eq!(stored_file_meta.filename, "transaction_file.txt");

    tx.commit().expect("Failed to commit transaction");

    // Verify file still exists after commit
    let stored_file_meta_after_commit = manager
        .get_file_meta(&file_meta_id)
        .expect("Failed to get file meta after commit");
    assert_eq!(
        stored_file_meta_after_commit.filename,
        "transaction_file.txt"
    );
}
