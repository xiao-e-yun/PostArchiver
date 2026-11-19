#![cfg(feature = "importer")]

mod common;

use std::collections::HashMap;

use common::{file_content, post, ts, Archive};
use post_archiver::{
    importer::{UnsyncAlias, UnsyncAuthor, UnsyncCollection, UnsyncTag},
    manager::{
        PostArchiverBindExt, UpdateAuthor, UpdateCollection, UpdateFileMeta, UpdatePlatform,
        UpdatePost, UpdateTag,
    },
    query::{PostArchiverQueryExt, Query},
    Comment, Content, Post,
};

// ── Field round-trips: set every field once, read it back ─────────────────────

#[test]
fn post_roundtrip() {
    let mut a = Archive::new();
    let pixiv = a.m.importer().import_platform("pixiv".into()).unwrap();
    let img = a.file("1.png", "1.png", b"PNG");
    let id = a.import(post("s1").content(vec![file_content(&img)]));
    let file = a.m.bind(id).list_file_metas().unwrap()[0];
    let content = vec![Content::Text("t".into()), Content::File(file)];
    let comments = vec![Comment {
        user: "u".into(),
        text: "c".into(),
        replies: vec![],
    }];

    a.m.bind(id)
        .update(
            UpdatePost::default()
                .title("T".into())
                .source(Some("s2".into()))
                .platform(Some(pixiv))
                .thumb(None)
                .content(content.clone())
                .comments(comments.clone())
                .published(ts(10))
                .updated(ts(20)),
        )
        .unwrap();

    let p = a.m.bind(id).value().unwrap();
    assert_eq!(
        (p.title.as_str(), p.source.as_deref(), p.platform, p.thumb),
        ("T", Some("s2"), Some(pixiv), None)
    );
    assert_eq!(
        (p.content, p.comments, p.published, p.updated),
        (content, comments, ts(10), ts(20))
    );

    a.m.bind(id)
        .update(UpdatePost::default().updated_by_latest(ts(5)))
        .unwrap();
    assert_eq!(
        a.m.bind(id).value().unwrap().updated,
        ts(20),
        "by_latest keeps the newer time"
    );
    a.m.bind(id)
        .update(UpdatePost::default().updated_by_latest(ts(30)))
        .unwrap();
    assert_eq!(a.m.bind(id).value().unwrap().updated, ts(30));
}

#[test]
fn other_entities_roundtrip() {
    let mut a = Archive::new();
    let pixiv = a.m.importer().import_platform("pixiv".into()).unwrap();
    let img = a.file("1.png", "1.png", b"PNG");
    let id = a.import(
        post("s1")
            .content(vec![file_content(&img)])
            .authors(vec![UnsyncAuthor::new("A".into())])
            .tags(vec![UnsyncTag {
                name: "t".into(),
                platform: None,
            }])
            .collections(vec![UnsyncCollection::new("C".into(), "c".into())]),
    );
    let b = a.m.bind(id);
    let (author, tag, collection, file) = (
        b.list_authors().unwrap()[0],
        b.list_tags().unwrap()[0],
        b.list_collections().unwrap()[0],
        b.list_file_metas().unwrap()[0],
    );

    a.m.bind(author)
        .update(
            UpdateAuthor::default()
                .name("A2".into())
                .thumb(Some(file))
                .updated(ts(7)),
        )
        .unwrap();
    let v = a.m.bind(author).value().unwrap();
    assert_eq!(
        (v.name.as_str(), v.thumb, v.updated),
        ("A2", Some(file), ts(7))
    );

    a.m.bind(tag)
        .update(UpdateTag::default().name("t2".into()).platform(Some(pixiv)))
        .unwrap();
    let v = a.m.bind(tag).value().unwrap();
    assert_eq!((v.name.as_str(), v.platform), ("t2", Some(pixiv)));

    a.m.bind(collection)
        .update(
            UpdateCollection::default()
                .name("C2".into())
                .source(Some("c2".into()))
                .thumb(Some(file)),
        )
        .unwrap();
    let v = a.m.bind(collection).value().unwrap();
    assert_eq!(
        (v.name.as_str(), v.source.as_deref(), v.thumb),
        ("C2", Some("c2"), Some(file))
    );

    a.m.bind(pixiv)
        .update(UpdatePlatform::default().name("Pixiv".into()))
        .unwrap();
    assert_eq!(a.m.bind(pixiv).value().unwrap().name, "Pixiv");

    let extra = HashMap::from([("w".to_string(), serde_json::json!(640))]);
    a.m.bind(file)
        .update(
            UpdateFileMeta::new()
                .mime("image/webp".into())
                .extra(extra.clone()),
        )
        .unwrap();
    let v = a.m.bind(file).value().unwrap();
    assert_eq!((v.mime.as_str(), v.extra), ("image/webp", extra));
    assert_eq!(
        a.m.bind(file).get_path().unwrap(),
        Post::directory(id).join("1.png"),
        "relative to the archive root"
    );
}

// ── Relations ────────────────────────────────────────────────────────────────

#[test]
fn relations_are_visible_from_both_sides() {
    let mut a = Archive::new();
    let pixiv = a.m.importer().import_platform("pixiv".into()).unwrap();
    let p1 = a.import(post("s1").platform("pixiv".into()));
    let p2 = a.import(post("s2"));
    let mut imp = a.m.importer();
    let author = imp.import_author(UnsyncAuthor::new("A".into())).unwrap();
    let tag = imp
        .import_tag(UnsyncTag {
            name: "t".into(),
            platform: Some(pixiv),
        })
        .unwrap();
    let collection = imp
        .import_collection(UnsyncCollection::new("C".into(), "c".into()))
        .unwrap();

    let b = a.m.bind(p1);
    b.add_authors(&[author]).unwrap();
    b.add_tags(&[tag]).unwrap();
    b.add_collections(&[collection]).unwrap();
    a.m.bind(collection).add_posts(&[p2]).unwrap();

    assert_eq!(a.m.bind(author).list_posts().unwrap(), [p1]);
    assert_eq!(a.m.bind(tag).list_posts().unwrap(), [p1]);
    let mut in_collection = a.m.bind(collection).list_posts().unwrap();
    in_collection.sort_by_key(|p| p.raw());
    assert_eq!(in_collection, [p1, p2]);
    assert_eq!(a.m.bind(p2).list_collections().unwrap(), [collection]);
    assert_eq!(a.m.bind(pixiv).list_posts().unwrap(), [p1]);
    assert_eq!(a.m.bind(pixiv).list_tags().unwrap(), [tag]);

    b.remove_authors(&[author]).unwrap();
    b.remove_tags(&[tag]).unwrap();
    b.remove_collections(&[collection]).unwrap();
    a.m.bind(collection).remove_posts(&[p2]).unwrap();
    assert!(b.list_authors().unwrap().is_empty());
    assert!(b.list_tags().unwrap().is_empty());
    assert!(a.m.bind(collection).list_posts().unwrap().is_empty());
}

#[test]
fn aliases_add_edit_remove() {
    let mut a = Archive::new();
    let pixiv = a.m.importer().import_platform("pixiv".into()).unwrap();
    let fanbox = a.m.importer().import_platform("fanbox".into()).unwrap();
    let author =
        a.m.importer()
            .import_author(UnsyncAuthor::new("A".into()))
            .unwrap();
    let b = a.m.bind(author);

    b.add_aliases([("a".to_string(), pixiv, None)]).unwrap();
    b.set_alias_link(&("a".into(), pixiv), Some("https://pixiv/a".into()))
        .unwrap();
    b.set_alias_platform(&("a".into(), pixiv), fanbox).unwrap();
    b.set_alias_name(&("a".into(), fanbox), "a2".into())
        .unwrap();

    let aliases = b.list_aliases().unwrap();
    assert_eq!(aliases.len(), 1);
    let alias = &aliases[0];
    assert_eq!(
        (
            alias.source.as_str(),
            alias.platform,
            alias.link.as_deref(),
            alias.target
        ),
        ("a2", fanbox, Some("https://pixiv/a"), author)
    );

    b.remove_aliases([("a2".to_string(), fanbox)]).unwrap();
    assert!(b.list_aliases().unwrap().is_empty());
}

// ── Deletes ──────────────────────────────────────────────────────────────────

#[test]
fn deleting_a_post_removes_its_relations_and_file_metas() {
    let mut a = Archive::new();
    let img = a.file("1.png", "1.png", b"PNG");
    let id = a.import(
        post("s1")
            .content(vec![file_content(&img)])
            .authors(vec![UnsyncAuthor::new("A".into())
                .aliases(vec![UnsyncAlias::new("unknown".into(), "a".into())])])
            .tags(vec![UnsyncTag {
                name: "t".into(),
                platform: None,
            }]),
    );
    let author = a.m.bind(id).list_authors().unwrap()[0];
    let tag = a.m.bind(id).list_tags().unwrap()[0];

    a.m.bind(id).delete().unwrap();

    assert!(a.m.posts().get::<Post>(id).unwrap().is_none());
    assert!(
        a.m.bind(author).list_posts().unwrap().is_empty(),
        "author_posts row removed"
    );
    assert!(
        a.m.bind(tag).list_posts().unwrap().is_empty(),
        "post_tags row removed"
    );
    assert_eq!(
        a.m.file_metas().query().count().unwrap(),
        0,
        "file_metas removed"
    );
}
