use crate::{
    importer::{ImportFileMetaMethod, UnsyncAuthor, UnsyncFileMeta, UnsyncPost},
    manager::PostArchiverManager,
};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[test]
fn test_check_file_meta() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let (author, _) = UnsyncAuthor::new("octocat".to_string())
        .alias(vec!["github:octocat".to_string()])
        .sync(&manager)
        .unwrap();

    // Create a post
    let (post, _) = UnsyncPost::new(author.id)
        .thumb(Some(UnsyncFileMeta {
            filename: "thumb.png".to_string(),
            mime: "image/png".to_string(),
            extra: HashMap::new(),
            method: ImportFileMetaMethod::None,
        }))
        .sync(&manager)
        .unwrap();

    assert!(post.thumb.is_some());

    // Check if the file meta exists
    let thumb = manager.check_file_meta(post.id, "thumb.png").unwrap();
    assert!(thumb.is_some());

    // Check both is equal
    assert_eq!(thumb, post.thumb);
}

#[test]
fn test_import_file_meta() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let (author, _) = UnsyncAuthor::new("octocat".to_string())
        .alias(vec!["github:octocat".to_string()])
        .sync(&manager)
        .unwrap();

    // Create a post
    let (post, _) = UnsyncPost::new(author.id).sync(&manager).unwrap();

    let thumb = UnsyncFileMeta {
        filename: "thumb.png".to_string(),
        mime: "image/png".to_string(),
        extra: HashMap::new(),
        method: ImportFileMetaMethod::Data(vec![1, 2, 3]),
    };

    let (thumb, method) = manager.import_file_meta(author.id, post.id, thumb).unwrap();

    // Archive the file
    match method {
        ImportFileMetaMethod::Data(data) => {
            // Create the directory if it doesn't exist
            std::fs::create_dir_all(manager.path.join(format!("{}/{}", author.id, post.id)))
                .unwrap();
            // Save the file to the archive
            std::fs::write(
                manager
                    .path
                    .join(format!("{}/{}/{}", author.id, post.id, thumb.filename)),
                data,
            )
            .unwrap();
        }
        _ => unreachable!(),
    };

    // Check if the file meta exists
    let thumb = manager.check_file_meta(post.id, "thumb.png").unwrap();
    assert!(thumb.is_some());
}

#[test]
fn test_import_file_meta_by_update() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let (author, _) = UnsyncAuthor::new("octocat".to_string())
        .sync(&manager)
        .unwrap();

    // Create a post
    let (post, _) = UnsyncPost::new(author.id).sync(&manager).unwrap();

    // Create initial file meta with some extra data
    let mut extra = HashMap::new();
    extra.insert("width".to_string(), json!(800));
    extra.insert("height".to_string(), json!(600));

    let thumb = UnsyncFileMeta {
        filename: "thumb.png".to_string(),
        mime: "image/png".to_string(),
        extra,
        method: ImportFileMetaMethod::None,
    };

    let (initial_meta, _) = manager.import_file_meta(author.id, post.id, thumb).unwrap();

    // Update with new extra data
    let mut new_extra = HashMap::new();
    new_extra.insert("alt".to_string(), json!("Updated image"));
    new_extra.insert("height".to_string(), json!(800)); // Override existing height

    let updated_meta = UnsyncFileMeta {
        filename: "thumb.png".to_string(),
        mime: "image/png".to_string(),
        extra: new_extra,
        method: ImportFileMetaMethod::None,
    };

    let (updated_meta, _) = manager
        .import_file_meta_by_update(author.id, post.id, initial_meta.id, updated_meta)
        .unwrap();

    // Verify extra data was merged properly
    assert_eq!(updated_meta.extra.get("width").unwrap(), &json!(800));
    assert_eq!(updated_meta.extra.get("height").unwrap(), &json!(800)); // Updated value
    assert_eq!(
        updated_meta.extra.get("alt").unwrap(),
        &json!("Updated image")
    );
}

#[test]
fn test_import_file_meta_methods() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let (author, _) = UnsyncAuthor::new("test_author".to_string())
        .sync(&manager)
        .unwrap();
    let (post, _) = UnsyncPost::new(author.id).sync(&manager).unwrap();

    // Test URL method
    let url_meta = UnsyncFileMeta {
        filename: "remote.jpg".to_string(),
        mime: "image/jpeg".to_string(),
        extra: HashMap::new(),
        method: ImportFileMetaMethod::Url("https://example.com/image.jpg".to_string()),
    };
    let (_meta, method) = manager
        .import_file_meta(author.id, post.id, url_meta)
        .unwrap();
    assert!(matches!(method, ImportFileMetaMethod::Url(_)));

    // Test File method
    let file_meta = UnsyncFileMeta {
        filename: "local.jpg".to_string(),
        mime: "image/jpeg".to_string(),
        extra: HashMap::new(),
        method: ImportFileMetaMethod::File(PathBuf::from("test.jpg")),
    };
    let (_meta, method) = manager
        .import_file_meta(author.id, post.id, file_meta)
        .unwrap();
    assert!(matches!(method, ImportFileMetaMethod::File(_)));

    // Test None method
    let none_meta = UnsyncFileMeta {
        filename: "phantom.jpg".to_string(),
        mime: "image/jpeg".to_string(),
        extra: HashMap::new(),
        method: ImportFileMetaMethod::None,
    };
    let (_meta, method) = manager
        .import_file_meta(author.id, post.id, none_meta)
        .unwrap();
    assert!(matches!(method, ImportFileMetaMethod::None));
}

#[test]
fn test_unsync_file_meta_equality() {
    let meta1 = UnsyncFileMeta {
        filename: "test.jpg".to_string(),
        mime: "image/jpeg".to_string(),
        extra: HashMap::new(),
        method: ImportFileMetaMethod::None,
    };

    let meta2 = UnsyncFileMeta {
        filename: "test.jpg".to_string(), // Same filename
        mime: "image/png".to_string(),    // Different mime
        extra: {
            // Different extra
            let mut map = HashMap::new();
            map.insert("key".to_string(), json!("value"));
            map
        },
        method: ImportFileMetaMethod::Data(vec![1, 2, 3]), // Different method
    };

    let meta3 = UnsyncFileMeta {
        filename: "different.jpg".to_string(), // Different filename
        mime: "image/jpeg".to_string(),
        extra: HashMap::new(),
        method: ImportFileMetaMethod::None,
    };

    // Test PartialEq
    assert_eq!(meta1, meta2); // Should be equal (same filename)
    assert_ne!(meta1, meta3); // Should not be equal (different filename)

    // Test Hash implementation
    let mut set = HashSet::new();
    set.insert(meta1.clone());
    assert!(set.contains(&meta2)); // Should find meta2 because filename is same as meta1
    assert!(!set.contains(&meta3)); // Should not find meta3 because filename is different
}

#[test]
fn test_import_file_meta_display() {
    let url_method = ImportFileMetaMethod::Url("https://example.com/image.jpg".to_string());
    assert_eq!(url_method.to_string(), "Url(https://example.com/image.jpg)");

    let file_method = ImportFileMetaMethod::File(PathBuf::from("test.jpg"));
    assert_eq!(file_method.to_string(), "File(test.jpg)");

    let data_method = ImportFileMetaMethod::Data(vec![1, 2, 3]);
    assert_eq!(data_method.to_string(), "Data(3 bytes)");

    let none_method = ImportFileMetaMethod::None;
    assert_eq!(none_method.to_string(), "None");
}
