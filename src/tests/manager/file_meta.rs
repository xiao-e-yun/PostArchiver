//! File metadata manager tests
//!
//! Tests for file metadata CRUD operations, query functionality,
//! and relationships with posts.

use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;

use crate::{manager::PostArchiverManager, tests::helpers, FileMetaId, PostId};

fn create_test_post(manager: &PostArchiverManager) -> PostId {
    let now = Utc::now();
    helpers::add_post(
        manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    )
}

fn create_test_extra() -> HashMap<String, Value> {
    let mut extra = HashMap::new();
    extra.insert("width".into(), Value::Number(1920.into()));
    extra.insert("height".into(), Value::Number(1080.into()));
    extra.insert("description".into(), Value::String("Test image".into()));
    extra
}

// ── CRUD via helpers ──────────────────────────────────────────

#[test]
fn test_add_file_meta() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);
    let extra = create_test_extra();

    let file_meta_id = helpers::add_file_meta(
        &manager,
        post_id,
        "test_image.jpg".into(),
        "image/jpeg".into(),
        extra.clone(),
    );
    assert!(file_meta_id.raw() > 0);

    let fm = helpers::get_file_meta(&manager, file_meta_id);
    assert_eq!(fm.id, file_meta_id);
    assert_eq!(fm.post, post_id);
    assert_eq!(fm.filename, "test_image.jpg");
    assert_eq!(fm.mime, "image/jpeg");
    assert_eq!(fm.extra, extra);
}

#[test]
fn test_get_file_meta() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);
    let extra = create_test_extra();

    let file_meta_id = helpers::add_file_meta(
        &manager,
        post_id,
        "get_test.png".into(),
        "image/png".into(),
        extra.clone(),
    );

    let fm = helpers::get_file_meta(&manager, file_meta_id);
    assert_eq!(fm.id, file_meta_id);
    assert_eq!(fm.post, post_id);
    assert_eq!(fm.filename, "get_test.png");
    assert_eq!(fm.mime, "image/png");
    assert_eq!(fm.extra, extra);
}

#[test]
fn test_find_file_meta() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);

    let file_meta_id = helpers::add_file_meta(
        &manager,
        post_id,
        "findme.gif".into(),
        "image/gif".into(),
        HashMap::new(),
    );

    let found_id = helpers::find_file_meta(&manager, post_id, "findme.gif");
    assert_eq!(found_id, Some(file_meta_id));

    let not_found = helpers::find_file_meta(&manager, post_id, "nonexistent.txt");
    assert_eq!(not_found, None);

    let other_post_id = create_test_post(&manager);
    let not_found = helpers::find_file_meta(&manager, other_post_id, "findme.gif");
    assert_eq!(not_found, None);
}

// ── Binded: Delete ────────────────────────────────────────────

#[test]
fn test_remove_file_meta() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);
    let file_meta_id = helpers::add_file_meta(
        &manager,
        post_id,
        "to_delete.txt".into(),
        "text/plain".into(),
        HashMap::new(),
    );

    let _ = helpers::get_file_meta(&manager, file_meta_id);

    manager.bind(file_meta_id).delete().unwrap();

    // Verify file meta is gone (find returns None)
    let found = helpers::find_file_meta(&manager, post_id, "to_delete.txt");
    assert_eq!(found, None);
}

// ── Binded: Set fields ───────────────────────────────────────

#[test]
fn test_set_file_meta_mime() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);
    let file_meta_id = helpers::add_file_meta(
        &manager,
        post_id,
        "mime_test.file".into(),
        "application/octet-stream".into(),
        HashMap::new(),
    );

    manager
        .bind(file_meta_id)
        .set_mime("text/plain".into())
        .unwrap();

    let fm = helpers::get_file_meta(&manager, file_meta_id);
    assert_eq!(fm.mime, "text/plain");
}

#[test]
fn test_set_file_meta_extra() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);
    let file_meta_id = helpers::add_file_meta(
        &manager,
        post_id,
        "extra_test.json".into(),
        "application/json".into(),
        HashMap::new(),
    );

    let mut new_extra = HashMap::new();
    new_extra.insert("size".into(), Value::Number(12345.into()));
    new_extra.insert("compressed".into(), Value::Bool(true));
    new_extra.insert("algorithm".into(), Value::String("gzip".into()));

    manager
        .bind(file_meta_id)
        .set_extra(new_extra.clone())
        .unwrap();

    let fm = helpers::get_file_meta(&manager, file_meta_id);
    assert_eq!(fm.extra, new_extra);
}

// ── Multiple & edge cases ────────────────────────────────────

#[test]
fn test_multiple_files_same_post() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);

    let file1 = helpers::add_file_meta(
        &manager,
        post_id,
        "file1.jpg".into(),
        "image/jpeg".into(),
        HashMap::new(),
    );
    let file2 = helpers::add_file_meta(
        &manager,
        post_id,
        "file2.png".into(),
        "image/png".into(),
        HashMap::new(),
    );

    let found1 = helpers::find_file_meta(&manager, post_id, "file1.jpg");
    let found2 = helpers::find_file_meta(&manager, post_id, "file2.png");
    assert_eq!(found1, Some(file1));
    assert_eq!(found2, Some(file2));

    let meta1 = helpers::get_file_meta(&manager, file1);
    let meta2 = helpers::get_file_meta(&manager, file2);
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
    let file1 = helpers::add_file_meta(
        &manager,
        post1_id,
        filename.clone(),
        "text/plain".into(),
        HashMap::new(),
    );
    let file2 = helpers::add_file_meta(
        &manager,
        post2_id,
        filename.clone(),
        "text/plain".into(),
        HashMap::new(),
    );

    let found1 = helpers::find_file_meta(&manager, post1_id, &filename);
    let found2 = helpers::find_file_meta(&manager, post2_id, &filename);
    assert_eq!(found1, Some(file1));
    assert_eq!(found2, Some(file2));
    assert_ne!(file1, file2);
}

#[test]
fn test_complex_extra_metadata() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);

    let mut extra = HashMap::new();
    extra.insert("simple_string".into(), Value::String("test".into()));
    extra.insert("number".into(), Value::Number(42.into()));
    extra.insert("boolean".into(), Value::Bool(true));
    extra.insert("null_value".into(), Value::Null);

    let mut nested = HashMap::new();
    nested.insert("inner_key".into(), Value::String("inner_value".into()));
    nested.insert("inner_number".into(), Value::Number(123.into()));
    extra.insert(
        "nested_object".into(),
        Value::Object(nested.into_iter().collect()),
    );

    let array = vec![
        Value::String("item1".into()),
        Value::Number(456.into()),
        Value::Bool(false),
    ];
    extra.insert("array".into(), Value::Array(array));

    let file_meta_id = helpers::add_file_meta(
        &manager,
        post_id,
        "complex.json".into(),
        "application/json".into(),
        extra.clone(),
    );

    let fm = helpers::get_file_meta(&manager, file_meta_id);
    assert_eq!(fm.extra, extra);
}

#[test]
fn test_empty_extra_metadata() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);

    let file_meta_id = helpers::add_file_meta(
        &manager,
        post_id,
        "empty_extra.txt".into(),
        "text/plain".into(),
        HashMap::new(),
    );

    let fm = helpers::get_file_meta(&manager, file_meta_id);
    assert!(fm.extra.is_empty());
}

#[test]
fn test_update_extra_metadata_multiple_times() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);
    let file_meta_id = helpers::add_file_meta(
        &manager,
        post_id,
        "update_test.txt".into(),
        "text/plain".into(),
        HashMap::new(),
    );

    let mut extra1 = HashMap::new();
    extra1.insert("version".into(), Value::Number(1.into()));
    manager
        .bind(file_meta_id)
        .set_extra(extra1.clone())
        .unwrap();

    let fm = helpers::get_file_meta(&manager, file_meta_id);
    assert_eq!(fm.extra, extra1);

    let mut extra2 = HashMap::new();
    extra2.insert("version".into(), Value::Number(2.into()));
    extra2.insert("author".into(), Value::String("test_user".into()));
    manager
        .bind(file_meta_id)
        .set_extra(extra2.clone())
        .unwrap();

    let fm = helpers::get_file_meta(&manager, file_meta_id);
    assert_eq!(fm.extra, extra2);
}

#[test]
fn test_update_nonexistent_file_meta() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let fake_id = FileMetaId::new(99999);

    // Update operations should not fail for nonexistent IDs (they just affect 0 rows)
    let mime_result = manager.bind(fake_id).set_mime("text/plain".into());
    assert!(mime_result.is_ok());

    let extra_result = manager.bind(fake_id).set_extra(HashMap::new());
    assert!(extra_result.is_ok());
}

#[test]
fn test_special_characters_in_filename() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);

    let special_filename =
        "test file with spaces & symbols!@#$%^&()_+-=[]{}|;':\",./<>?.txt".to_string();

    let file_meta_id = helpers::add_file_meta(
        &manager,
        post_id,
        special_filename.clone(),
        "text/plain".into(),
        HashMap::new(),
    );

    let found_id = helpers::find_file_meta(&manager, post_id, &special_filename);
    assert_eq!(found_id, Some(file_meta_id));

    let fm = helpers::get_file_meta(&manager, file_meta_id);
    assert_eq!(fm.filename, special_filename);
}

#[test]
fn test_unicode_filename() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let post_id = create_test_post(&manager);

    let unicode_filename = "测试文件_тест_ファイル_🎉.txt".to_string();

    let file_meta_id = helpers::add_file_meta(
        &manager,
        post_id,
        unicode_filename.clone(),
        "text/plain".into(),
        HashMap::new(),
    );

    let found_id = helpers::find_file_meta(&manager, post_id, &unicode_filename);
    assert_eq!(found_id, Some(file_meta_id));

    let fm = helpers::get_file_meta(&manager, file_meta_id);
    assert_eq!(fm.filename, unicode_filename);
}
