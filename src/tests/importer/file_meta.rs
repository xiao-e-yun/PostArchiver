use crate::{
    importer::{UnsyncAlias, UnsyncAuthor, UnsyncFileMeta, UnsyncPost},
    manager::platform::PlatformIdOrRaw,
    manager::PostArchiverManager,
};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_import_file_meta() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let author = UnsyncAuthor::new("octocat".to_string())
        .aliases(vec![UnsyncAlias::new(
            &PlatformIdOrRaw::Raw("github".to_string()),
            "octocat",
        )])
        .sync(&manager)
        .unwrap();

    // Create a post
    let (post, _) = UnsyncPost::new()
        .authors(vec![author.id])
        .sync(&manager, HashMap::<String, Vec<u8>>::new())
        .unwrap();

    let mut file_meta = UnsyncFileMeta {
        filename: "thumb.png".to_string(),
        mime: "image/png".to_string(),
        extra: HashMap::new(),
    };

    // Test object-oriented API
    let saved_file = manager
        .import_file_meta(post.id, file_meta.clone())
        .unwrap();

    manager
        .set_post_thumb(post.id, &Some(saved_file.id))
        .unwrap();
    assert_eq!(saved_file.filename, file_meta.filename);

    // Test updating with extra data
    file_meta.extra.insert("width".to_string(), json!(800));
    let updated_file = manager
        .import_file_meta(post.id, file_meta.clone())
        .unwrap();
    assert_eq!(
        manager.get_file_meta(&updated_file.id).unwrap().extra,
        file_meta.extra
    );
}

#[test]
fn test_functional_import_file_meta_by_create() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let author = UnsyncAuthor::new("octocat".to_string())
        .sync(&manager)
        .unwrap();

    // Create a post
    let (post, _) = UnsyncPost::new()
        .authors(vec![author.id])
        .sync(&manager, HashMap::<String, Vec<u8>>::new())
        .unwrap();

    let file_meta = UnsyncFileMeta {
        filename: "thumb.png".to_string(),
        mime: "image/png".to_string(),
        extra: HashMap::new(),
    };

    // Test functional API - create
    let saved_file = manager
        .import_file_meta_by_create(post.id, file_meta.clone())
        .unwrap();

    manager
        .set_post_thumb(post.id, &Some(saved_file.id))
        .unwrap();
    assert_eq!(saved_file.filename, file_meta.filename);
}

#[test]
fn test_functional_import_file_meta_by_update() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let author = UnsyncAuthor::new("octocat".to_string())
        .sync(&manager)
        .unwrap();

    // Create a post
    let (post, _) = UnsyncPost::new()
        .authors(vec![author.id])
        .sync(&manager, HashMap::<String, Vec<u8>>::new())
        .unwrap();

    let mut file_meta = UnsyncFileMeta {
        filename: "thumb.png".to_string(),
        mime: "image/png".to_string(),
        extra: HashMap::new(),
    };

    // Create initial file
    let saved_file = manager
        .import_file_meta_by_create(post.id, file_meta.clone())
        .unwrap();

    // Update with new extra data
    file_meta.extra.insert("width".to_string(), json!(800));
    let updated_file = manager
        .import_file_meta_by_update(post.id, saved_file.id, file_meta.clone())
        .unwrap();
    assert_eq!(
        manager.get_file_meta(&updated_file.id).unwrap().extra,
        file_meta.extra
    );
}

#[test]
fn test_functional_check_file_meta() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Test check_file_meta when file doesn't exist
    let result = manager
        .check_file_meta(crate::PostId(1), "test.jpg")
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn test_functional_get_file_meta() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author and post
    let author = UnsyncAuthor::new("test".to_string())
        .sync(&manager)
        .unwrap();
    let (post, _) = UnsyncPost::new()
        .authors(vec![author.id])
        .sync(&manager, HashMap::<String, Vec<u8>>::new())
        .unwrap();

    // Create file meta
    let file_meta = UnsyncFileMeta {
        filename: "test.jpg".to_string(),
        mime: "image/jpeg".to_string(),
        extra: HashMap::new(),
    };

    let saved_file = manager
        .import_file_meta_by_create(post.id, file_meta)
        .unwrap();

    // Test get_file_meta functional API
    let retrieved_file = manager.get_file_meta(&saved_file.id).unwrap();
    assert_eq!(retrieved_file.filename, "test.jpg");
    assert_eq!(retrieved_file.mime, "image/jpeg");
}
