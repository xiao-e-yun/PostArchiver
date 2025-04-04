use crate::{
    importer::{ImportFileMetaMethod, UnsyncAuthor, UnsyncFileMeta, UnsyncPost},
    manager::PostArchiverManager,
};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_import_file_meta() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let author = UnsyncAuthor::new("octocat".to_string())
        .alias(vec!["github:octocat".to_string()])
        .sync(&manager)
        .unwrap();

    // Create a post
    let (post, _) = UnsyncPost::new(author.id).sync(&manager).unwrap();

    let mut thumb = UnsyncFileMeta {
        filename: "thumb.png".to_string(),
        mime: "image/png".to_string(),
        extra: HashMap::new(),
        method: ImportFileMetaMethod::Data(vec![1, 2, 3]),
    };

    let (saved_thumb, method) = manager
        .import_file_meta(author.id, post.id, thumb.clone())
        .unwrap();
    manager
        .set_post_thumb(post.id, &Some(saved_thumb.id))
        .unwrap();
    assert_eq!(saved_thumb.filename, thumb.filename);

    match method {
        ImportFileMetaMethod::Data(data) => {
            // Create the directory if it doesn't exist
            std::fs::create_dir_all(manager.path.join(format!("{}/{}", author.id, post.id)))
                .unwrap();
            // Save the file to the archive
            assert_eq!(data, vec![1, 2, 3]);
            std::fs::write(
                manager.path.join(format!(
                    "{}/{}/{}",
                    author.id, post.id, saved_thumb.filename
                )),
                data,
            )
            .unwrap();
        }
        _ => unreachable!(),
    };

    thumb.extra.insert("width".to_string(), json!(800));
    manager
        .import_file_meta(author.id, post.id, thumb.clone())
        .unwrap();
    assert_eq!(
        manager.get_file_meta(&saved_thumb.id).unwrap().extra,
        thumb.extra
    );
}

#[test]
fn test_import_file_meta_by_parts() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let author = UnsyncAuthor::new("octocat".to_string())
        .alias(vec!["github:octocat".to_string()])
        .sync(&manager)
        .unwrap();

    // Create a post
    let (post, _) = UnsyncPost::new(author.id).sync(&manager).unwrap();

    let mut thumb = UnsyncFileMeta {
        filename: "thumb.png".to_string(),
        mime: "image/png".to_string(),
        extra: HashMap::new(),
        method: ImportFileMetaMethod::Data(vec![1, 2, 3]),
    };

    let (saved_thumb, method) = manager
        .import_file_meta_by_create(author.id, post.id, thumb.clone())
        .unwrap();
    manager
        .set_post_thumb(post.id, &Some(saved_thumb.id))
        .unwrap();
    assert_eq!(saved_thumb.filename, thumb.filename);

    match method {
        ImportFileMetaMethod::Data(data) => {
            // Create the directory if it doesn't exist
            std::fs::create_dir_all(manager.path.join(format!("{}/{}", author.id, post.id)))
                .unwrap();
            // Save the file to the archive
            assert_eq!(data, vec![1, 2, 3]);
            std::fs::write(
                manager.path.join(format!(
                    "{}/{}/{}",
                    author.id, post.id, saved_thumb.filename
                )),
                data,
            )
            .unwrap();
        }
        _ => unreachable!(),
    };

    thumb.extra.insert("width".to_string(), json!(800));
    manager
        .import_file_meta_by_update(author.id, post.id, saved_thumb.id, thumb.clone())
        .unwrap();
    assert_eq!(
        manager.get_file_meta(&saved_thumb.id).unwrap().extra,
        thumb.extra
    );
}

#[test]
fn test_import_file_meta_methods() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let author = UnsyncAuthor::new("octocat".to_string())
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
