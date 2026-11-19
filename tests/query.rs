#![cfg(feature = "importer")]

mod common;

use common::{file_content, post, ts, Archive};
use post_archiver::{
    impl_from_query,
    importer::{UnsyncAlias, UnsyncAuthor, UnsyncCollection, UnsyncTag},
    manager::{PostArchiverBindExt, UpdatePost},
    query::{
        author::AuthorSort, post::PostQuery, post::PostSort, Pagination, PostArchiverQueryExt,
        Query,
    },
    Alias, Author, AuthorId, CollectionId, FileMeta, PlatformId, Post, PostId, TagId,
};

/// Four posts with overlapping relations:
///
/// | post | title          | platform | authors | tags            | collection | updated | published |
/// |------|----------------|----------|---------|-----------------|------------|---------|-----------|
/// | p1   | Hello world    | pixiv    | A       | general         | C          | 100     | 1000      |
/// | p2   | 50% off        | pixiv    | A, B    | general, local  |            | 200     | 2000      |
/// | p3   | a_b            | fanbox   | B       |                 |            | 300     | 3000      |
/// | p4   | back\slash     | -        |         |                 |            | 400     | 4000      |
struct Fixture {
    a: Archive,
    p: [PostId; 4],
    alice: AuthorId,
    bob: AuthorId,
    general: TagId,
    local: TagId,
    series: CollectionId,
    pixiv: PlatformId,
    fanbox: PlatformId,
}

fn fixture() -> Fixture {
    let mut a = Archive::new();
    let pixiv = a.m.importer().import_platform("pixiv".into()).unwrap();
    let fanbox = a.m.importer().import_platform("fanbox".into()).unwrap();
    let alice = UnsyncAuthor::new("Alice".into())
        .aliases(vec![UnsyncAlias::new("pixiv".into(), "alice".into())]);
    let bob = UnsyncAuthor::new("Bob".into())
        .aliases(vec![UnsyncAlias::new("pixiv".into(), "bob".into())]);
    let general = UnsyncTag {
        name: "general".into(),
        platform: None,
    };
    let local = UnsyncTag {
        name: "local".into(),
        platform: Some(pixiv),
    };
    let series = UnsyncCollection::new("Series".into(), "series".into());
    let img = a.file("f.png", "f.png", b"PNG");

    let p = [
        a.import(
            post("src/1")
                .title("Hello world".into())
                .platform("pixiv".into())
                .authors(vec![alice.clone()])
                .tags(vec![general.clone()])
                .collections(vec![series.clone()]),
        ),
        a.import(
            post("src/2")
                .title("50% off".into())
                .platform("pixiv".into())
                .authors(vec![alice.clone(), bob.clone()])
                .tags(vec![general.clone(), local.clone()]),
        ),
        a.import(
            post("src/3")
                .title("a_b".into())
                .platform("fanbox".into())
                .authors(vec![bob.clone()]),
        ),
        a.import(
            post("src/4")
                .title("back\\slash".into())
                .content(vec![file_content(&img)]),
        ),
    ];
    // set dates explicitly so the fixture does not depend on importer timestamp handling
    for (i, id) in p.iter().enumerate() {
        let n = i as i64 + 1;
        a.m.bind(*id)
            .update(
                UpdatePost::default()
                    .updated(ts(n * 100))
                    .published(ts(n * 1000)),
            )
            .unwrap();
    }

    let imp = a.m.importer();
    let fixture_ids = (
        imp.lookup_author("alice", pixiv).unwrap().unwrap(),
        imp.lookup_author("bob", pixiv).unwrap().unwrap(),
        imp.lookup_tag("general", None).unwrap().unwrap(),
        imp.lookup_tag("local", Some(pixiv)).unwrap().unwrap(),
        imp.lookup_collection("series").unwrap().unwrap(),
    );
    let (alice, bob, general, local, series) = fixture_ids;
    Fixture {
        a,
        p,
        alice,
        bob,
        general,
        local,
        series,
        pixiv,
        fanbox,
    }
}

impl Fixture {
    /// IDs of posts matching the filters set by `f`, in ascending order.
    fn ids(&self, f: impl FnOnce(&mut PostQuery)) -> Vec<PostId> {
        let mut q = self.a.m.posts().query();
        f(&mut q);
        let mut ids: Vec<PostId> = q
            .query::<Post>()
            .unwrap()
            .into_iter()
            .map(|p| p.id)
            .collect();
        ids.sort_by_key(|id| id.raw());
        ids
    }
}

// ── Filters ───────────────────────────────────────────────────────────────────

#[test]
fn relation_filters_match_all_given_ids() {
    let f = fixture();
    let [p1, p2, p3, _] = f.p;
    assert_eq!(
        f.ids(|q| {
            q.authors.insert(f.alice);
        }),
        [p1, p2]
    );
    assert_eq!(
        f.ids(|q| {
            q.authors.extend([f.alice, f.bob]);
        }),
        [p2]
    );
    assert_eq!(
        f.ids(|q| {
            q.authors.insert(f.bob);
        }),
        [p2, p3]
    );
    assert_eq!(
        f.ids(|q| {
            q.tags.insert(f.general);
        }),
        [p1, p2]
    );
    assert_eq!(
        f.ids(|q| {
            q.tags.extend([f.general, f.local]);
        }),
        [p2]
    );
    assert_eq!(
        f.ids(|q| {
            q.collections.insert(f.series);
        }),
        [p1]
    );
    assert_eq!(
        f.ids(|q| {
            q.authors.insert(f.alice);
            q.tags.insert(f.local);
        }),
        [p2]
    );
}

#[test]
fn id_filters() {
    let f = fixture();
    let [p1, p2, p3, p4] = f.p;
    assert_eq!(
        f.ids(|q| {
            q.ids.insert(p3);
        }),
        [p3]
    );
    assert_eq!(
        f.ids(|q| {
            q.ids.extend([p1, p4]);
        }),
        [p1, p4]
    );
    assert_eq!(
        f.ids(|q| {
            q.platforms.insert(f.pixiv);
        }),
        [p1, p2]
    );
    assert_eq!(
        f.ids(|q| {
            q.platforms.extend([f.pixiv, f.fanbox]);
        }),
        [p1, p2, p3]
    );
}

#[test]
fn text_filters_escape_wildcards() {
    let f = fixture();
    let [p1, p2, p3, p4] = f.p;
    assert_eq!(
        f.ids(|q| {
            q.title.contains("world");
        }),
        [p1]
    );
    assert_eq!(
        f.ids(|q| {
            q.title.starts_with("Hello");
        }),
        [p1]
    );
    assert_eq!(
        f.ids(|q| {
            q.title.ends_with("off");
        }),
        [p2]
    );
    assert_eq!(
        f.ids(|q| {
            q.title.equals("a_b");
        }),
        [p3]
    );
    assert_eq!(
        f.ids(|q| {
            q.title.contains("%");
        }),
        [p2],
        "% is literal"
    );
    assert_eq!(
        f.ids(|q| {
            q.title.contains("_");
        }),
        [p3],
        "_ is literal"
    );
    assert_eq!(
        f.ids(|q| {
            q.title.contains("\\");
        }),
        [p4],
        "\\ is literal"
    );
    assert_eq!(
        f.ids(|q| {
            q.title.like("a%");
        }),
        [p3],
        "like() keeps wildcards"
    );
    assert_eq!(
        f.ids(|q| {
            q.title.contains("");
        }),
        f.p.to_vec(),
        "empty text adds no condition"
    );
    assert_eq!(
        f.ids(|q| {
            q.source.equals("src/2");
        }),
        [p2]
    );
}

#[test]
fn date_filters() {
    let f = fixture();
    let [p1, p2, p3, p4] = f.p;
    assert_eq!(
        f.ids(|q| {
            q.updated.before(ts(200));
        }),
        [p1, p2]
    );
    assert_eq!(
        f.ids(|q| {
            q.updated.after(ts(300));
        }),
        [p3, p4]
    );
    assert_eq!(
        f.ids(|q| {
            q.updated.after(ts(200)).before(ts(300));
        }),
        [p2, p3]
    );
    assert_eq!(
        f.ids(|q| {
            q.updated.equals(ts(400));
        }),
        [p4]
    );
    assert_eq!(
        f.ids(|q| {
            q.published.after(ts(3500));
        }),
        [p4]
    );
}

// ── Order, pagination, count ─────────────────────────────────────────────────

#[test]
fn order_pagination_and_count() {
    let f = fixture();
    let [p1, p2, p3, p4] = f.p;
    let run = |q: PostQuery| {
        q.query::<Post>()
            .unwrap()
            .into_iter()
            .map(|p| p.id)
            .collect::<Vec<_>>()
    };

    let mut q = f.a.m.posts().query();
    q.order.desc(PostSort::Updated);
    assert_eq!(run(q), [p4, p3, p2, p1]);

    // keys apply in call order: the first key decides
    let mut q = f.a.m.posts().query();
    q.ids.extend([p1, p2, p3]);
    q.order.desc(PostSort::Id).asc(PostSort::Title);
    assert_eq!(run(q), [p3, p2, p1]);
    let mut q = f.a.m.posts().query();
    q.ids.extend([p1, p2, p3]);
    q.order.asc(PostSort::Title).desc(PostSort::Id);
    assert_eq!(
        run(q),
        [p2, p1, p3],
        "\"50% off\" < \"Hello world\" < \"a_b\""
    );

    let mut q = f.a.m.posts().query();
    q.order.asc(PostSort::Updated);
    q.pagination = Pagination::default().limit(2).page(1);
    assert_eq!(q.count().unwrap(), 4, "count ignores pagination");
    assert_eq!(run(q), [p3, p4]);

    let mut q = f.a.m.posts().query();
    q.order.asc(PostSort::Updated);
    q.pagination = Pagination::default().offset(3);
    assert_eq!(run(q), [p4], "offset without limit");

    let mut q = f.a.m.posts().query();
    q.order.random();
    assert_eq!(run(q).len(), 4);

    let mut q = f.a.m.posts().query();
    q.authors.insert(f.bob);
    assert_eq!(q.count().unwrap(), 2);
}

// ── get / projections ────────────────────────────────────────────────────────

struct PostCard {
    id: PostId,
    title: String,
    author_posts: Vec<AuthorId>,
    file_metas: Vec<FileMeta>,
}

impl_from_query! {
    PostCard extends Post {
        id: id,
        title: title,
        author_posts: author_posts,
        file_metas: file_metas,
    }
}

#[test]
fn get_full_entity_and_missing_id() {
    let f = fixture();
    let p: Post = f.a.m.posts().get(f.p[0]).unwrap().unwrap();
    assert_eq!(p.title, "Hello world");
    assert!(f.a.m.posts().get::<Post>(PostId(9999)).unwrap().is_none());
}

#[test]
fn get_many_skips_missing_and_duplicates() {
    let f = fixture();
    let mut posts: Vec<Post> =
        f.a.m
            .posts()
            .get_many([f.p[2], f.p[0], f.p[2], PostId(9999)])
            .unwrap();
    posts.sort_by_key(|p| p.id.raw());
    assert_eq!(
        posts.iter().map(|p| p.id).collect::<Vec<_>>(),
        vec![f.p[0], f.p[2]]
    );

    // single id, and a projection
    let cards: Vec<AuthorCard> = f.a.m.authors().get_many([f.bob]).unwrap();
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].author_aliases[0].source, "bob");

    // empty input means no rows, not the whole table
    assert!(f.a.m.posts().get_many::<Post>([]).unwrap().is_empty());
}

#[test]
fn projection_with_relation_columns() {
    let f = fixture();
    let card: PostCard = f.a.m.posts().get(f.p[1]).unwrap().unwrap();
    let mut authors = card.author_posts.clone();
    authors.sort_by_key(|a| a.raw());
    assert_eq!(
        (card.id, card.title.as_str(), authors),
        (f.p[1], "50% off", vec![f.alice, f.bob])
    );
    assert!(card.file_metas.is_empty());

    let card: PostCard = f.a.m.posts().get(f.p[3]).unwrap().unwrap();
    assert_eq!(card.file_metas.len(), 1);
    assert_eq!(
        (
            card.file_metas[0].filename.as_str(),
            card.file_metas[0].mime.as_str()
        ),
        ("f.png", "image/png")
    );

    let mut q = f.a.m.posts().query();
    q.authors.insert(f.bob);
    assert_eq!(q.query::<PostCard>().unwrap().len(), 2);
}

struct AuthorCard {
    id: AuthorId,
    author_aliases: Vec<Alias>,
}

impl_from_query! {
    AuthorCard extends Author {
        id: id,
        author_aliases: author_aliases,
    }
}

#[test]
fn author_projection_with_aliases() {
    let f = fixture();
    let card: AuthorCard = f.a.m.authors().get(f.alice).unwrap().unwrap();
    assert_eq!(card.id, f.alice);
    assert_eq!(card.author_aliases.len(), 1);
    assert_eq!(card.author_aliases[0].source, "alice");
}

// ── Other entities ───────────────────────────────────────────────────────────

#[test]
fn other_entity_queries() {
    let f = fixture();

    let mut q = f.a.m.authors().query();
    q.name.starts_with("Al");
    q.order.asc(AuthorSort::Name);
    assert_eq!(
        q.query::<Author>()
            .unwrap()
            .into_iter()
            .map(|a| a.id)
            .collect::<Vec<_>>(),
        [f.alice]
    );
    assert_eq!(f.a.m.authors().query().count().unwrap(), 2);

    let mut q = f.a.m.tags().query();
    q.platforms.insert(f.pixiv);
    assert_eq!(
        q.query::<post_archiver::Tag>()
            .unwrap()
            .into_iter()
            .map(|t| t.id)
            .collect::<Vec<_>>(),
        [f.local]
    );

    let mut q = f.a.m.collections().query();
    q.name.contains("Ser");
    assert_eq!(q.count().unwrap(), 1);

    let mut q = f.a.m.platforms().query();
    q.name.equals("fanbox");
    assert_eq!(
        q.query::<post_archiver::Platform>().unwrap()[0].id,
        f.fanbox
    );

    let mut q = f.a.m.file_metas().query();
    q.posts.insert(f.p[3]);
    q.mime.starts_with("image/");
    assert_eq!(q.count().unwrap(), 1);

    let bob: AuthorCard = f.a.m.authors().get(f.bob).unwrap().unwrap();
    assert_eq!(bob.author_aliases.len(), 1);
}
