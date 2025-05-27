use chrono::{TimeZone, Utc};

use crate::{manager::PostArchiverManager, Alias, Tag};

#[test]
fn test_open_manager() {
    let mut manager = PostArchiverManager::open_in_memory().unwrap();
    let tx = manager.transaction().unwrap();
    tx.commit().unwrap();
}

#[test]
fn test_get_all() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let updated = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();

    // Check when the database is empty
    assert!(manager
        .check_author(&["github:octocat".to_string()])
        .unwrap()
        .is_none());
    assert!(manager.check_post("https://example.com").unwrap().is_none());
    assert!(manager
        .check_post_with_updated("https://example.com", &updated)
        .unwrap()
        .is_none());
    assert!(manager
        .check_file_meta(crate::PostId(1), "thumb.png")
        .unwrap()
        .is_none());

    let expamle_query = "
    INSERT INTO features (name, value) VALUES ('example', 1);
    INSERT INTO features (name, value, extra) VALUES ('example_extra', 2, '{\"key\": \"value\"}');

    INSERT INTO authors (name) VALUES ('octocat');

    INSERT INTO author_aliases (source, target) VALUES ('github:octocat', 1);

    INSERT INTO posts (author, title, content, source) VALUES (1, 'Hello World', '[]', 'https://example.com');

    INSERT INTO tags (name) VALUES ('hello');

    INSERT INTO post_tags (post, tag) VALUES (1, 1);

    INSERT INTO file_metas (filename, author, post, mime) VALUES ('thumb.png', 1, 1, 'image/png');

    UPDATE posts SET thumb = 1 WHERE id = 1;
    ";

    manager.conn().execute_batch(expamle_query).unwrap();

    // Check when the database is not empty
    assert!(manager
        .check_author(&["github:octocat".to_string()])
        .unwrap()
        .is_some());
    assert!(manager.check_post("https://example.com").unwrap().is_some());
    assert!(manager
        .check_post_with_updated("https://example.com", &updated)
        .unwrap()
        .is_some());
    assert!(manager
        .check_post_with_updated("https://example.com", &Utc::now())
        .unwrap()
        .is_none());
    assert!(manager
        .check_file_meta(crate::PostId(1), "thumb.png")
        .unwrap()
        .is_some());

    // Get values from the database
    let value = manager.get_feature("example");
    assert_eq!(value, 1);

    let (value, extra) = manager.get_feature_with_extra("example_extra");
    assert_eq!(value, 2);
    assert_eq!(extra["key"], "value");

    let value = manager.get_feature("unknown");
    assert_eq!(value, 0);

    let author = manager.get_author(&crate::AuthorId(1)).unwrap();
    assert_eq!(author.name, "octocat");

    let author_aliases = manager.get_author_aliases(&crate::AuthorId(1)).unwrap();
    assert_eq!(
        author_aliases[0],
        Alias {
            source: "github:octocat".to_string(),
            target: crate::AuthorId(1)
        }
    );

    let post = manager.get_post(&crate::PostId(1)).unwrap();
    assert_eq!(post.title, "Hello World");

    let tags = manager.get_post_tags(&crate::PostId(1)).unwrap();
    assert_eq!(
        tags[0],
        Tag {
            id: crate::PostTagId(1),
            name: "hello".to_string()
        }
    );

    let file_meta = manager.get_file_meta(&crate::FileMetaId(1)).unwrap();
    assert_eq!(file_meta.filename, "thumb.png");
}
