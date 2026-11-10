#![cfg(feature = "importer")]

mod common;

use common::{post, Archive};
use post_archiver::{
    importer::{UnsyncAlias, UnsyncAuthor},
    manager::{PostArchiverBindExt, PostArchiverManager},
};

#[test]
fn tag_uniqueness() {
    let a = Archive::new();
    let insert = |name: &str, platform: Option<i64>| {
        a.m.execute(
            "INSERT INTO tags (name, platform) VALUES (?, ?)",
            (name, platform),
        )
    };
    insert("x", None).unwrap();
    insert("x", Some(0)).unwrap();
    a.m.execute("INSERT INTO platforms (id, name) VALUES (1, 'pixiv')", ())
        .unwrap();
    insert("x", Some(1)).unwrap();

    assert!(
        insert("x", None).is_err(),
        "general tags are unique by name"
    );
    assert!(
        insert("x", Some(1)).is_err(),
        "platform tags are unique per platform"
    );
}

#[test]
fn same_alias_source_on_two_platforms_are_two_authors() {
    let mut a = Archive::new();
    let pixiv = a.m.importer().import_platform("pixiv".into()).unwrap();
    let fanbox = a.m.importer().import_platform("fanbox".into()).unwrap();
    let mut imp = a.m.importer();
    let on_pixiv = imp
        .import_author(
            UnsyncAuthor::new("P".into()).aliases(vec![UnsyncAlias::new(pixiv, "123".into())]),
        )
        .unwrap();
    let on_fanbox = imp
        .import_author(
            UnsyncAuthor::new("F".into()).aliases(vec![UnsyncAlias::new(fanbox, "123".into())]),
        )
        .unwrap();

    assert_ne!(on_pixiv, on_fanbox);
    assert_eq!(a.m.bind(on_pixiv).list_aliases().unwrap().len(), 1);
    assert_eq!(a.m.bind(on_fanbox).list_aliases().unwrap().len(), 1);
}

#[test]
fn file_meta_filename_is_unique_per_post() {
    let mut a = Archive::new();
    let id = a.import(post("s1"));
    let insert = || {
        a.m.execute(
            "INSERT INTO file_metas (post, filename, mime) VALUES (?, 'a.png', 'image/png')",
            (id.raw(),),
        )
    };
    insert().unwrap();
    assert!(insert().is_err());
}

#[test]
fn foreign_keys_are_enforced() {
    let a = Archive::new();
    let on: i64 =
        a.m.query_row("PRAGMA foreign_keys", (), |r| r.get(0))
            .unwrap();
    assert_eq!(
        on, 1,
        "ON DELETE CASCADE / SET NULL in the schema only work with foreign_keys on"
    );
}

#[test]
fn reopen_existing_archive() {
    let mut a = Archive::new();
    let id = a.import(post("s1"));
    let root = a.root();

    let mut reopened = PostArchiverManager::open(&root)
        .unwrap()
        .expect("archive exists");
    assert_eq!(reopened.importer().lookup_post("s1").unwrap(), Some(id));
    assert!(PostArchiverManager::open(root.join("missing"))
        .unwrap()
        .is_none());
    assert!(
        PostArchiverManager::create(&root).is_err(),
        "create refuses an existing archive"
    );
}
