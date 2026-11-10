#![cfg(feature = "importer")]

mod common;

use common::{file_content, post, ts, Archive};
use post_archiver::{
    importer::{
        UnsyncAlias, UnsyncAuthor, UnsyncCollection, UnsyncContent, UnsyncFileMeta, UnsyncTag,
    },
    manager::PostArchiverBindExt,
    query::{PostArchiverQueryExt, Query},
    Author, Collection, Comment, Content, Platform, PlatformId, Post, Tag,
};

fn author(name: &str, platform: PlatformId, source: &str) -> UnsyncAuthor {
    UnsyncAuthor::new(name.into()).aliases(vec![UnsyncAlias::new(platform, source.into())])
}

// ── Posts ─────────────────────────────────────────────────────────────────────

#[test]
fn import_post_with_everything() {
    let mut a = Archive::new();
    let pixiv = a.m.importer().import_platform("pixiv".into()).unwrap();
    let img = a.file("1.png", "1.png", b"PNG");
    let comment = Comment {
        user: "u".into(),
        text: "hi".into(),
        replies: vec![],
    };

    let id = a.import(
        post("https://pixiv/1")
            .platform("pixiv".into())
            .title("Title".into())
            .content(vec![
                UnsyncContent::Text("hello".into()),
                file_content(&img),
            ])
            .comments(vec![comment.clone()])
            .published(ts(100))
            .updated(ts(200))
            .tags(vec![
                UnsyncTag {
                    name: "general".into(),
                    platform: None,
                },
                UnsyncTag {
                    name: "local".into(),
                    platform: Some(pixiv),
                },
            ])
            .authors(vec![author("Alice", pixiv, "alice")])
            .collections(vec![UnsyncCollection::new(
                "Series".into(),
                "https://pixiv/s/1".into(),
            )]),
    );

    let p: Post = a.m.posts().get(id).unwrap().unwrap();
    let file = a.m.bind(id).list_file_metas().unwrap()[0];
    assert_eq!(p.source.as_deref(), Some("https://pixiv/1"));
    assert_eq!(p.title, "Title");
    assert_eq!(p.platform, Some(pixiv));
    assert_eq!(
        p.content,
        vec![Content::Text("hello".into()), Content::File(file)]
    );
    assert_eq!(p.thumb, Some(file), "first image becomes the thumb");
    assert_eq!(p.comments, vec![comment]);
    assert_eq!((p.published, p.updated), (ts(100), ts(200)));
    assert_eq!(a.read(id, "1.png").as_deref(), Some(&b"PNG"[..]));

    let b = a.m.bind(id);
    let tags: Vec<Tag> = b
        .list_tags()
        .unwrap()
        .into_iter()
        .map(|t| a.m.tags().get(t).unwrap().unwrap())
        .collect();
    let mut tag_names: Vec<_> = tags.iter().map(|t| (t.name.as_str(), t.platform)).collect();
    tag_names.sort_by_key(|(name, _)| *name);
    assert_eq!(tag_names, [("general", None), ("local", Some(pixiv))]);

    let authors = b.list_authors().unwrap();
    let alice: Author = a.m.authors().get(authors[0]).unwrap().unwrap();
    assert_eq!((authors.len(), alice.name.as_str()), (1, "Alice"));

    let collections = b.list_collections().unwrap();
    let series: Collection = a.m.collections().get(collections[0]).unwrap().unwrap();
    assert_eq!(series.name, "Series");

    let platform: Platform = a.m.platforms().get(pixiv).unwrap().unwrap();
    assert_eq!(platform.name, "pixiv");
}

#[test]
fn reimport_updates_in_place() {
    let mut a = Archive::new();
    let first = a.import(post("s1").title("old".into()).updated(ts(200)));
    let second = a.import(post("s1").title("new".into()).updated(ts(100)));

    assert_eq!(first, second);
    assert_eq!(a.m.posts().query().count().unwrap(), 1);
    let p: Post = a.m.posts().get(first).unwrap().unwrap();
    assert_eq!(p.title, "new");
    assert_eq!(p.updated, ts(200), "updated never moves backwards");
}

#[test]
fn explicit_thumb_wins_and_non_images_are_not_thumbs() {
    let mut a = Archive::new();
    let cover = a.file("cover.jpg", "cover.jpg", b"JPG");
    let img = a.file("1.png", "1.png", b"PNG");
    let video = a.file("v.mp4", "v.mp4", b"MP4");

    let with_cover = a.import(
        post("s1")
            .thumb(Some(cover))
            .content(vec![file_content(&img)]),
    );
    let video_only = a.import(post("s2").content(vec![file_content(&video)]));

    let p: Post = a.m.posts().get(with_cover).unwrap().unwrap();
    let cover_meta =
        a.m.file_metas()
            .get::<post_archiver::FileMeta>(p.thumb.unwrap())
            .unwrap()
            .unwrap();
    assert_eq!(cover_meta.filename, "cover.jpg");
    let p: Post = a.m.posts().get(video_only).unwrap().unwrap();
    assert_eq!(p.thumb, None);
}

#[test]
fn lookups() {
    let mut a = Archive::new();
    let pixiv = a.m.importer().import_platform("pixiv".into()).unwrap();
    let id = a.import(
        post("s1")
            .updated(ts(200))
            .tags(vec![UnsyncTag {
                name: "t".into(),
                platform: Some(pixiv),
            }])
            .collections(vec![UnsyncCollection::new("c".into(), "c-src".into())]),
    );

    let imp = a.m.importer();
    assert_eq!(imp.lookup_post("s1").unwrap(), Some(id));
    assert_eq!(imp.lookup_post("missing").unwrap(), None);
    assert_eq!(
        imp.lookup_post_with_updated("s1", &ts(200)).unwrap(),
        Some(id)
    );
    assert_eq!(imp.lookup_post_with_updated("s1", &ts(201)).unwrap(), None);
    assert!(imp.lookup_tag("t", Some(pixiv)).unwrap().is_some());
    assert_eq!(
        imp.lookup_tag("t", None).unwrap(),
        None,
        "platform tag is not a general tag"
    );
    assert!(imp.lookup_collection("c-src").unwrap().is_some());
    assert_eq!(imp.lookup_platform("pixiv").unwrap(), Some(pixiv));
}

// ── Authors ───────────────────────────────────────────────────────────────────

#[test]
fn authors_are_merged_by_alias_and_aliases_are_updated() {
    let mut a = Archive::new();
    let pixiv = a.m.importer().import_platform("pixiv".into()).unwrap();
    let fanbox = a.m.importer().import_platform("fanbox".into()).unwrap();

    let first =
        a.m.importer()
            .import_author(author("Alice", pixiv, "alice"))
            .unwrap();
    // same pixiv alias plus a new fanbox alias, new name and a new link
    let second =
        a.m.importer()
            .import_author(UnsyncAuthor::new("Alice2".into()).aliases(vec![
                UnsyncAlias::new(pixiv, "alice".into()).link("https://pixiv/alice"),
                UnsyncAlias::new(fanbox, "alice-fb".into()),
            ]))
            .unwrap();

    assert_eq!(first, second);
    let alice: Author = a.m.authors().get(first).unwrap().unwrap();
    assert_eq!(alice.name, "Alice2");
    let mut aliases = a.m.bind(first).list_aliases().unwrap();
    aliases.sort_by(|x, y| x.source.cmp(&y.source));
    assert_eq!(aliases.len(), 2);
    assert_eq!(
        aliases[0].link.as_deref(),
        Some("https://pixiv/alice"),
        "existing alias link is updated"
    );
}

#[test]
fn author_and_collection_follow_latest_post() {
    let mut a = Archive::new();
    let older = a.file("old.png", "old.png", b"OLD");
    let newer = a.file("new.png", "new.png", b"NEW");
    let alice = author("Alice", PlatformId(0), "alice");
    let series = UnsyncCollection::new("Series".into(), "series".into());

    let new_post = a.import(
        post("new")
            .updated(ts(200))
            .thumb(Some(newer))
            .authors(vec![alice.clone()])
            .collections(vec![series.clone()]),
    );
    // importing an older post afterwards must not steal the thumb
    a.import(
        post("old")
            .updated(ts(100))
            .thumb(Some(older))
            .authors(vec![alice])
            .collections(vec![series]),
    );

    let new_thumb = a.m.posts().get::<Post>(new_post).unwrap().unwrap().thumb;
    let author_id = a.m.bind(new_post).list_authors().unwrap()[0];
    let collection_id = a.m.bind(new_post).list_collections().unwrap()[0];
    let author: Author = a.m.authors().get(author_id).unwrap().unwrap();
    let collection: Collection = a.m.collections().get(collection_id).unwrap().unwrap();
    assert_eq!((author.thumb, author.updated), (new_thumb, ts(200)));
    assert_eq!(collection.thumb, new_thumb);
}

// ── Files ─────────────────────────────────────────────────────────────────────

#[test]
fn files_are_copied_and_sources_are_kept() {
    let mut a = Archive::new();
    let img = a.file("1.png", "1.png", b"PNG");
    let id = a.import(post("s1").content(vec![file_content(&img)]));

    assert_eq!(a.read(id, "1.png").as_deref(), Some(&b"PNG"[..]));
    assert_eq!(std::fs::read(a.src("1.png")).unwrap(), b"PNG");
    assert!(a.leftovers().is_empty());
}

#[test]
fn same_file_as_thumb_and_content() {
    let mut a = Archive::new();
    let img = a.file("1.png", "1.png", b"PNG");
    let id = a.import(
        post("s1")
            .thumb(Some(img.clone()))
            .content(vec![file_content(&img)]),
    );

    assert_eq!(a.m.bind(id).list_file_metas().unwrap().len(), 1);
    assert_eq!(a.read(id, "1.png").as_deref(), Some(&b"PNG"[..]));
    assert_eq!(std::fs::read(a.src("1.png")).unwrap(), b"PNG");
}

#[test]
fn rewriting_a_source_path_does_not_touch_the_archive() {
    let mut a = Archive::new();
    let img = a.file("download.tmp", "1.png", b"FIRST");
    let id = a.import(post("s1").content(vec![file_content(&img)]));

    std::fs::write(a.src("download.tmp"), b"SECOND").unwrap();
    assert_eq!(a.read(id, "1.png").as_deref(), Some(&b"FIRST"[..]));
}

#[test]
fn reimport_replaces_file_content() {
    let mut a = Archive::new();
    let v1 = a.file("v1.png", "1.png", b"V1");
    let v2 = a.file("v2.png", "1.png", b"V2");
    let id = a.import(post("s1").content(vec![file_content(&v1)]));
    a.import(post("s1").content(vec![file_content(&v2)]));

    assert_eq!(a.m.bind(id).list_file_metas().unwrap().len(), 1);
    assert_eq!(a.read(id, "1.png").as_deref(), Some(&b"V2"[..]));
}

/// A post whose second file is missing, so the import fails *after* `img` was staged.
fn failing_post(
    a: &Archive,
    source: &str,
    img: &UnsyncFileMeta,
) -> post_archiver::importer::UnsyncPost {
    let mut missing = a.file("missing.png", "missing.png", b"");
    missing.source = a.src("does-not-exist.png");
    post(source).content(vec![file_content(img), file_content(&missing)])
}

#[test]
fn failed_import_rolls_back_everything() {
    let mut a = Archive::new();
    let img = a.file("1.png", "1.png", b"DATA");
    let bad = failing_post(&a, "s1", &img);
    assert!(a.m.importer().import_post(bad).is_err());

    assert_eq!(a.m.importer().lookup_post("s1").unwrap(), None);
    assert_eq!(a.m.file_metas().query().count().unwrap(), 0);
    assert!(a.leftovers().is_empty());
    assert_eq!(
        std::fs::read(a.src("1.png")).unwrap(),
        b"DATA",
        "source survives for a retry"
    );
}

#[test]
fn missing_source_file_fails_the_import() {
    let mut a = Archive::new();
    let mut img = a.file("1.png", "1.png", b"PNG");
    img.source = a.src("does-not-exist.png");
    assert!(a
        .m
        .importer()
        .import_post(post("s1").content(vec![file_content(&img)]))
        .is_err());
    assert_eq!(a.m.importer().lookup_post("s1").unwrap(), None);
}

#[test]
fn outer_savepoint_commits_or_discards_everything() {
    let mut a = Archive::new();
    let img = a.file("1.png", "1.png", b"PNG");

    {
        let mut imp = a.m.importer();
        let mut sp = imp.savepoint().unwrap();
        sp.import_post(post("dropped").content(vec![file_content(&img)]))
            .unwrap();
        // dropped without commit
    }
    assert_eq!(a.m.importer().lookup_post("dropped").unwrap(), None);
    assert!(a.leftovers().is_empty());

    let root = a.root();
    let mut imp = a.m.importer();
    let mut sp = imp.savepoint().unwrap();
    let id = sp
        .import_post(post("kept").content(vec![file_content(&img)]))
        .unwrap();
    let target = root.join(Post::directory(id)).join("1.png");
    assert!(
        !target.exists(),
        "files appear only after the outermost commit"
    );
    sp.commit().unwrap();
    assert_eq!(a.read(id, "1.png").as_deref(), Some(&b"PNG"[..]));
}

#[test]
fn nested_rollback_keeps_files_staged_by_the_parent() {
    let mut a = Archive::new();
    let img = a.file("1.png", "1.png", b"PNG");

    // the retry stages the same file again (same bytes, as a real retry would)
    let retry = failing_post(&a, "s1", &img);
    let mut imp = a.m.importer();
    let mut sp = imp.savepoint().unwrap();
    let id = sp
        .import_post(post("s1").content(vec![file_content(&img)]))
        .unwrap();
    // retry the same post inside the same transaction, and fail
    assert!(sp.import_post(retry).is_err());
    sp.commit().unwrap();

    assert_eq!(a.read(id, "1.png").as_deref(), Some(&b"PNG"[..]));
    assert!(a.leftovers().is_empty());
}
