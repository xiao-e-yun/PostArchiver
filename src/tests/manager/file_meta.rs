//! File metadata manager tests
//!
//! Tests for file metadata CRUD operations, query functionality,
//! and relationships with posts.

use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;

use crate::{manager::PostArchiverManager, FileMetaId, PostId};

fn create_test_post(manager: &PostArchiverManager) -> PostId {
    manager
        .add_post(
            "Test Post".to_string(),
            None,
            None,
            Some(Utc::now()),
            Some(Utc::now()),
        )
        .expect("Failed to create test post")
}

fn create_test_extra() -> HashMap<String, Value> {
    let mut extra = HashMap::new();
    extra.insert("width".to_string(), Value::Number(1920.into()));
    extra.insert("height".to_string(), Value::Number(1080.into()));
    extra.insert(
        "description".to_string(),
        Value::String("Test image".to_string()),
    );
    extra
}

#[test]
fn test_add_file_meta() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);

    let filename = "test_image.jpg".to_string();
    let mime = "image/jpeg".to_string();
    let extra = create_test_extra();

    let file_meta_id = manager
        .add_file_meta(post_id, filename.clone(), mime.clone(), extra.clone())
        .expect("Failed to add file meta");

    assert!(file_meta_id.raw() > 0);

    // Verify the file meta was added correctly
    let file_meta = manager
        .get_file_meta(&file_meta_id)
        .expect("Failed to get file meta");

    assert_eq!(file_meta.id, file_meta_id);
    assert_eq!(file_meta.post, post_id);
    assert_eq!(file_meta.filename, filename);
    assert_eq!(file_meta.mime, mime);
    assert_eq!(file_meta.extra, extra);
}

#[test]
fn test_get_file_meta() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);

    let filename = "get_test.png".to_string();
    let mime = "image/png".to_string();
    let extra = create_test_extra();

    let file_meta_id = manager
        .add_file_meta(post_id, filename.clone(), mime.clone(), extra.clone())
        .expect("Failed to add file meta");

    let file_meta = manager
        .get_file_meta(&file_meta_id)
        .expect("Failed to get file meta");

    assert_eq!(file_meta.id, file_meta_id);
    assert_eq!(file_meta.post, post_id);
    assert_eq!(file_meta.filename, filename);
    assert_eq!(file_meta.mime, mime);
    assert_eq!(file_meta.extra, extra);
}

#[test]
fn test_find_file_meta() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);

    let filename = "findme.gif".to_string();
    let mime = "image/gif".to_string();
    let extra = HashMap::new();

    let file_meta_id = manager
        .add_file_meta(post_id, filename.clone(), mime, extra)
        .expect("Failed to add file meta");

    // Test finding existing file
    let found_id = manager
        .find_file_meta(post_id, &filename)
        .expect("Failed to find file meta");

    assert_eq!(found_id, Some(file_meta_id));

    // Test not found cases
    let not_found = manager
        .find_file_meta(post_id, "nonexistent.txt")
        .expect("Failed to search for nonexistent file");

    assert_eq!(not_found, None);

    // Test with different post ID
    let other_post_id = create_test_post(&manager);
    let not_found_different_post = manager
        .find_file_meta(other_post_id, &filename)
        .expect("Failed to search for file in different post");

    assert_eq!(not_found_different_post, None);
}

#[test]
fn test_remove_file_meta() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);

    let file_meta_id = manager
        .add_file_meta(
            post_id,
            "to_delete.txt".to_string(),
            "text/plain".to_string(),
            HashMap::new(),
        )
        .expect("Failed to add file meta");

    // Verify file meta exists
    manager
        .get_file_meta(&file_meta_id)
        .expect("File meta should exist before deletion");

    // Remove file meta
    manager
        .remove_file_meta(file_meta_id)
        .expect("Failed to remove file meta");

    // Verify file meta is gone
    let result = manager.get_file_meta(&file_meta_id);
    assert!(result.is_err(), "File meta should not exist after deletion");
}

#[test]
fn test_set_file_meta_mime() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);

    let file_meta_id = manager
        .add_file_meta(
            post_id,
            "mime_test.file".to_string(),
            "application/octet-stream".to_string(),
            HashMap::new(),
        )
        .expect("Failed to add file meta");

    let new_mime = "text/plain".to_string();
    manager
        .set_file_meta_mime(file_meta_id, new_mime.clone())
        .expect("Failed to update file meta MIME type");

    let file_meta = manager
        .get_file_meta(&file_meta_id)
        .expect("Failed to get file meta");

    assert_eq!(file_meta.mime, new_mime);
}

#[test]
fn test_set_file_meta_extra() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);

    let file_meta_id = manager
        .add_file_meta(
            post_id,
            "extra_test.json".to_string(),
            "application/json".to_string(),
            HashMap::new(),
        )
        .expect("Failed to add file meta");

    let mut new_extra = HashMap::new();
    new_extra.insert("size".to_string(), Value::Number(12345.into()));
    new_extra.insert("compressed".to_string(), Value::Bool(true));
    new_extra.insert("algorithm".to_string(), Value::String("gzip".to_string()));

    manager
        .set_file_meta_extra(file_meta_id, new_extra.clone())
        .expect("Failed to update file meta extra");

    let file_meta = manager
        .get_file_meta(&file_meta_id)
        .expect("Failed to get file meta");

    assert_eq!(file_meta.extra, new_extra);
}

#[test]
fn test_multiple_files_same_post() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);

    // Add multiple files to the same post
    let file1_id = manager
        .add_file_meta(
            post_id,
            "file1.jpg".to_string(),
            "image/jpeg".to_string(),
            HashMap::new(),
        )
        .expect("Failed to add file1");

    let file2_id = manager
        .add_file_meta(
            post_id,
            "file2.png".to_string(),
            "image/png".to_string(),
            HashMap::new(),
        )
        .expect("Failed to add file2");

    // Verify both files can be found
    let found1 = manager
        .find_file_meta(post_id, "file1.jpg")
        .expect("Failed to find file1");
    let found2 = manager
        .find_file_meta(post_id, "file2.png")
        .expect("Failed to find file2");

    assert_eq!(found1, Some(file1_id));
    assert_eq!(found2, Some(file2_id));

    // Verify both files can be retrieved
    let meta1 = manager
        .get_file_meta(&file1_id)
        .expect("Failed to get file1 meta");
    let meta2 = manager
        .get_file_meta(&file2_id)
        .expect("Failed to get file2 meta");

    assert_eq!(meta1.filename, "file1.jpg");
    assert_eq!(meta2.filename, "file2.png");
    assert_eq!(meta1.post, post_id);
    assert_eq!(meta2.post, post_id);
}

#[test]
fn test_same_filename_different_posts() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post1_id = create_test_post(&manager);
    let post2_id = create_test_post(&manager);

    let filename = "shared_name.txt".to_string();
    let mime = "text/plain".to_string();

    // Add files with same filename to different posts
    let file1_id = manager
        .add_file_meta(post1_id, filename.clone(), mime.clone(), HashMap::new())
        .expect("Failed to add file to post1");

    let file2_id = manager
        .add_file_meta(post2_id, filename.clone(), mime.clone(), HashMap::new())
        .expect("Failed to add file to post2");

    // Verify files are found correctly for their respective posts
    let found1 = manager
        .find_file_meta(post1_id, &filename)
        .expect("Failed to find file in post1");
    let found2 = manager
        .find_file_meta(post2_id, &filename)
        .expect("Failed to find file in post2");

    assert_eq!(found1, Some(file1_id));
    assert_eq!(found2, Some(file2_id));
    assert_ne!(file1_id, file2_id);
}

#[test]
fn test_complex_extra_metadata() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);

    // Create complex nested extra metadata
    let mut extra = HashMap::new();
    extra.insert(
        "simple_string".to_string(),
        Value::String("test".to_string()),
    );
    extra.insert("number".to_string(), Value::Number(42.into()));
    extra.insert("boolean".to_string(), Value::Bool(true));
    extra.insert("null_value".to_string(), Value::Null);

    // Nested object
    let mut nested = HashMap::new();
    nested.insert(
        "inner_key".to_string(),
        Value::String("inner_value".to_string()),
    );
    nested.insert("inner_number".to_string(), Value::Number(123.into()));
    extra.insert(
        "nested_object".to_string(),
        Value::Object(nested.into_iter().collect()),
    );

    // Array
    let array = vec![
        Value::String("item1".to_string()),
        Value::Number(456.into()),
        Value::Bool(false),
    ];
    extra.insert("array".to_string(), Value::Array(array));

    let file_meta_id = manager
        .add_file_meta(
            post_id,
            "complex.json".to_string(),
            "application/json".to_string(),
            extra.clone(),
        )
        .expect("Failed to add file meta with complex extra");

    let file_meta = manager
        .get_file_meta(&file_meta_id)
        .expect("Failed to get file meta");

    assert_eq!(file_meta.extra, extra);
}

#[test]
fn test_empty_extra_metadata() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);

    let file_meta_id = manager
        .add_file_meta(
            post_id,
            "empty_extra.txt".to_string(),
            "text/plain".to_string(),
            HashMap::new(),
        )
        .expect("Failed to add file meta with empty extra");

    let file_meta = manager
        .get_file_meta(&file_meta_id)
        .expect("Failed to get file meta");

    assert!(file_meta.extra.is_empty());
}

#[test]
fn test_update_extra_metadata_multiple_times() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);

    let file_meta_id = manager
        .add_file_meta(
            post_id,
            "update_test.txt".to_string(),
            "text/plain".to_string(),
            HashMap::new(),
        )
        .expect("Failed to add file meta");

    // First update
    let mut extra1 = HashMap::new();
    extra1.insert("version".to_string(), Value::Number(1.into()));
    manager
        .set_file_meta_extra(file_meta_id, extra1.clone())
        .expect("Failed to update extra first time");

    let file_meta = manager
        .get_file_meta(&file_meta_id)
        .expect("Failed to get file meta");
    assert_eq!(file_meta.extra, extra1);

    // Second update
    let mut extra2 = HashMap::new();
    extra2.insert("version".to_string(), Value::Number(2.into()));
    extra2.insert("author".to_string(), Value::String("test_user".to_string()));
    manager
        .set_file_meta_extra(file_meta_id, extra2.clone())
        .expect("Failed to update extra second time");

    let file_meta = manager
        .get_file_meta(&file_meta_id)
        .expect("Failed to get file meta");
    assert_eq!(file_meta.extra, extra2);
}

#[test]
fn test_get_nonexistent_file_meta() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Try to get a file meta that doesn't exist
    let fake_id = FileMetaId::new(99999);
    let result = manager.get_file_meta(&fake_id);

    assert!(result.is_err(), "Getting nonexistent file meta should fail");
}

#[test]
fn test_update_nonexistent_file_meta() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let fake_id = FileMetaId::new(99999);

    // Try to update MIME type of nonexistent file
    let mime_result = manager.set_file_meta_mime(fake_id, "text/plain".to_string());
    assert!(
        mime_result.is_ok(),
        "Update operations should not fail for nonexistent IDs"
    );

    // Try to update extra of nonexistent file
    let extra_result = manager.set_file_meta_extra(fake_id, HashMap::new());
    assert!(
        extra_result.is_ok(),
        "Update operations should not fail for nonexistent IDs"
    );
}

#[test]
fn test_special_characters_in_filename() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);

    let special_filename =
        "test file with spaces & symbols!@#$%^&()_+-=[]{}|;':\",./<>?.txt".to_string();
    let mime = "text/plain".to_string();

    let file_meta_id = manager
        .add_file_meta(post_id, special_filename.clone(), mime, HashMap::new())
        .expect("Failed to add file meta with special characters in filename");

    // Verify the file can be found by its special filename
    let found_id = manager
        .find_file_meta(post_id, &special_filename)
        .expect("Failed to find file with special characters");

    assert_eq!(found_id, Some(file_meta_id));

    let file_meta = manager
        .get_file_meta(&file_meta_id)
        .expect("Failed to get file meta");

    assert_eq!(file_meta.filename, special_filename);
}

#[test]
fn test_unicode_filename() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);

    let unicode_filename = "测试文件_тест_ファイル_🎉.txt".to_string();
    let mime = "text/plain".to_string();

    let file_meta_id = manager
        .add_file_meta(post_id, unicode_filename.clone(), mime, HashMap::new())
        .expect("Failed to add file meta with Unicode filename");

    let found_id = manager
        .find_file_meta(post_id, &unicode_filename)
        .expect("Failed to find file with Unicode filename");

    assert_eq!(found_id, Some(file_meta_id));

    let file_meta = manager
        .get_file_meta(&file_meta_id)
        .expect("Failed to get file meta");

    assert_eq!(file_meta.filename, unicode_filename);
}
