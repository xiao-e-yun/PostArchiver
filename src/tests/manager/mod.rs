use std::collections::HashMap;

use chrono::{TimeZone, Utc};
use tempdir::TempDir;

use crate::{manager::PostArchiverManager, Alias, Tag};

#[test]
fn test_open_manager() {
    let mut manager =
        PostArchiverManager::open_in_memory().expect("Failed to open in-memory manager");
    let tx = manager.transaction().expect("Failed to create transaction");
    tx.commit().expect("Failed to commit transaction");

    let temp = TempDir::new("post_archiver_test").expect("Failed to create temporary directory");
    let path = temp.path();

    assert!(PostArchiverManager::open(path)
        .expect("Failed to open manager when not exists")
        .is_none());
    PostArchiverManager::create(path).expect("Failed to create manager");
    assert!(PostArchiverManager::open(path)
        .expect("Failed to open manager")
        .is_some());
    assert!(PostArchiverManager::open_uncheck(path)
        .expect("Failed to open manager without checking")
        .is_some());
}

#[test]
fn test_get_all() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let updated = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();

    // Check when the database is empty
    assert!(manager
        .check_author(&[(&"github", "octocat")])
        .expect("Failed to check author")
        .is_none());
    assert!(manager.check_post("https://example.com").unwrap().is_none());
    assert!(manager
        .check_post_with_updated("https://example.com", &updated)
        .expect("Failed to check post with updated")
        .is_none());
    assert!(manager
        .check_file_meta(crate::PostId(1), "thumb.png")
        .expect("Failed to check file meta")
        .is_none());

    let value = manager
        .get_feature("example")
        .expect("Failed to get feature");
    assert_eq!(value, 0);

    let (value, extra) = manager
        .get_feature_with_extra("example_extra")
        .expect("Failed to get feature with extra");
    assert_eq!(value, 0);
    assert!(extra.is_empty());

    let expamle_query = "
    INSERT INTO features (name, value) VALUES ('example', 1);
    INSERT INTO features (name, value, extra) VALUES ('example_extra', 2, '{\"key\": \"value\"}');

    INSERT INTO platforms (name) VALUES ('example_platform');

    INSERT INTO authors (name) VALUES ('octocat');

    INSERT INTO author_aliases (source, platform, target) VALUES ('github:octocat', 1);


    INSERT INTO posts (title, content, source, platform) VALUES (1, 'Hello World', '[]', 'https://example.com', 1);

    INSERT INTO author_posts (author, post) VALUES (1, 1);

    INSERT INTO tags name VALUES 'hello';
    INSERT INTO platform_tags (platform, name) VALUES ('example_platform', 'example_tag');

    INSERT INTO post_tags (post, tag) VALUES (1, 1);

    INSERT INTO file_metas (filename, author, post, mime) VALUES ('thumb.png', 1, 1, 'image/png');

    UPDATE posts SET thumb = 1 WHERE id = 1;
    ";

    manager.conn().execute_batch(expamle_query).unwrap();

    // Check when the database is not empty
    assert!(manager
        .check_author(&[(&"github", "octocat")])
        .unwrap()
        .is_some());
    assert!(manager.check_post("https://example.com").unwrap().is_some());
    assert!(manager
        .check_post_with_updated("https://example.com", &updated)
        .unwrap()
        .is_some());
    assert!(manager
        .check_post_with_updated(
            "https://example.com",
            &Utc.with_ymd_and_hms(2016, 1, 1, 0, 0, 0).unwrap()
        )
        .unwrap()
        .is_none());
    assert!(manager
        .check_file_meta(crate::PostId(1), "thumb.png")
        .unwrap()
        .is_some());

    // Get values from the database
    let value = manager
        .get_feature("example")
        .expect("Failed to get feature");
    assert_eq!(value, 1);

    let (value, extra) = manager
        .get_feature_with_extra("example_extra")
        .expect("Failed to get feature with extra");
    assert_eq!(value, 2);
    assert_eq!(extra["key"], "value");

    let platform = manager
        .get_platform(&"example_platform")
        .expect("Failed to get platform")
        .expect("Platform not found");

    let author = manager.get_author(&crate::AuthorId(1)).unwrap();
    assert_eq!(author.name, "octocat");

    let author_aliases = manager.get_author_aliases(&author.id).unwrap();
    assert_eq!(
        author_aliases[0],
        Alias {source:"github:octocat".to_string(),target:crate::AuthorId(1), platform: , link: todo!() }
    );

    let post = manager.get_post(&crate::PostId(1)).unwrap();
    assert_eq!(post.title, "Hello World");

    let tags = manager.list_post_tags(&crate::PostId(1)).unwrap();
    assert_eq!(
        tags[0],
        Tag {
            id: crate::TagId(1),
            category: COLLECTION_CATEGORY.to_string(),
            name: "hello".to_string()
        }
    );

    let file_meta = manager.get_file_meta(&crate::FileMetaId(1)).unwrap();
    assert_eq!(file_meta.filename, "thumb.png");
}
