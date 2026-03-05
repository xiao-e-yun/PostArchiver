//! Tests for `src/query/file_meta.rs`

use crate::{manager::PostArchiverManager, tests::helpers};
use chrono::Utc;

// ── get_file_meta ─────────────────────────────────────────────────────────────

#[test]
fn test_get_file_meta_exists() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post_id = helpers::add_post(&m, "Post".into(), None, None, Some(now), Some(now));
    let file_id = helpers::add_file_meta(
        &m,
        post_id,
        "photo.jpg".into(),
        "image/jpeg".into(),
        Default::default(),
    );

    let fm = m.get_file_meta(file_id).unwrap().unwrap();
    assert_eq!(fm.id, file_id);
    assert_eq!(fm.filename, "photo.jpg");
    assert_eq!(fm.mime, "image/jpeg");
    assert_eq!(fm.post, post_id);
}

#[test]
fn test_get_file_meta_not_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    use crate::FileMetaId;
    let result = m.get_file_meta(FileMetaId::from(9999u32)).unwrap();
    assert!(result.is_none());
}

// ── find_file_meta ────────────────────────────────────────────────────────────

#[test]
fn test_find_file_meta_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post_id = helpers::add_post(&m, "Post".into(), None, None, Some(now), Some(now));
    let file_id = helpers::add_file_meta(
        &m,
        post_id,
        "cover.png".into(),
        "image/png".into(),
        Default::default(),
    );

    let found = m.find_file_meta(post_id, "cover.png").unwrap();
    assert_eq!(found, Some(file_id));
}

#[test]
fn test_find_file_meta_not_found() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post_id = helpers::add_post(&m, "Post".into(), None, None, Some(now), Some(now));

    let result = m.find_file_meta(post_id, "nonexistent.png").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_find_file_meta_wrong_post() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post1 = helpers::add_post(&m, "Post 1".into(), None, None, Some(now), Some(now));
    let post2 = helpers::add_post(&m, "Post 2".into(), None, None, Some(now), Some(now));
    helpers::add_file_meta(
        &m,
        post1,
        "img.jpg".into(),
        "image/jpeg".into(),
        Default::default(),
    );

    // same filename but different post → not found
    let result = m.find_file_meta(post2, "img.jpg").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_find_file_meta_same_name_different_posts() {
    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post1 = helpers::add_post(&m, "Post 1".into(), None, None, Some(now), Some(now));
    let post2 = helpers::add_post(&m, "Post 2".into(), None, None, Some(now), Some(now));

    let fid1 = helpers::add_file_meta(
        &m,
        post1,
        "cover.jpg".into(),
        "image/jpeg".into(),
        Default::default(),
    );
    let fid2 = helpers::add_file_meta(
        &m,
        post2,
        "cover.jpg".into(),
        "image/jpeg".into(),
        Default::default(),
    );

    assert_eq!(m.find_file_meta(post1, "cover.jpg").unwrap(), Some(fid1));
    assert_eq!(m.find_file_meta(post2, "cover.jpg").unwrap(), Some(fid2));
    assert_ne!(fid1, fid2);
}

// ── extra metadata preserved ──────────────────────────────────────────────────

#[test]
fn test_get_file_meta_extra() {
    use serde_json::json;
    use std::collections::HashMap;

    let m = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let post_id = helpers::add_post(&m, "Post".into(), None, None, Some(now), Some(now));

    let mut extra = HashMap::new();
    extra.insert("width".to_string(), json!(1920));
    extra.insert("height".to_string(), json!(1080));

    let file_id = helpers::add_file_meta(
        &m,
        post_id,
        "hd.jpg".into(),
        "image/jpeg".into(),
        extra.clone(),
    );

    let fm = m.get_file_meta(file_id).unwrap().unwrap();
    assert_eq!(fm.extra.get("width"), Some(&json!(1920)));
    assert_eq!(fm.extra.get("height"), Some(&json!(1080)));
}
