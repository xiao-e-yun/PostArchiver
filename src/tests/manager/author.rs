//! Author manager tests
//!
//! Tests for author CRUD operations, alias management,
//! and author-post relationships.

use crate::{
    manager::{PostArchiverManager, UpdateAuthor, UpdatePost},
    tests::helpers,
    AuthorId, PlatformId,
};
use chrono::Utc;

// ── CRUD via helpers ──────────────────────────────────────────

#[test]
fn test_add_author() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let author_id = helpers::add_author(&manager, "Test Author".into(), Some(now));
    assert!(author_id.raw() > 0);

    let author = helpers::get_author(&manager, author_id);
    assert_eq!(author.name, "Test Author");
    assert_eq!(author.id, author_id);
}

#[test]
fn test_list_authors() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let id1 = helpers::add_author(&manager, "Author 1".into(), Some(now));
    let id2 = helpers::add_author(&manager, "Author 2".into(), Some(now));

    let authors = helpers::list_authors(&manager);
    assert_eq!(authors.len(), 2);
    let ids: Vec<AuthorId> = authors.iter().map(|a| a.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

#[test]
fn test_get_author() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let author_id = helpers::add_author(&manager, "Get Test Author".into(), Some(now));

    let author = helpers::get_author(&manager, author_id);
    assert_eq!(author.id, author_id);
    assert_eq!(author.name, "Get Test Author");
}

#[test]
fn test_find_author() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let platform_id = helpers::add_platform(&manager, "Test Platform".into());
    let author_id = helpers::add_author(&manager, "Test Author".into(), Some(now));

    helpers::add_author_aliases(
        &manager,
        author_id,
        vec![("findme".into(), platform_id, None)],
    );

    let found_id = helpers::find_author(&manager, &[("findme", platform_id)]);
    assert_eq!(found_id, Some(author_id));

    let not_found = helpers::find_author(&manager, &[("nonexistent", platform_id)]);
    assert_eq!(not_found, None);
}

#[test]
fn test_empty_find_author() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let result = helpers::find_author(&manager, &[] as &[(&str, PlatformId)]);
    assert_eq!(result, None);
}

// ── Binded: Delete ────────────────────────────────────────────

#[test]
fn test_remove_author() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let author_id = helpers::add_author(&manager, "To Delete".into(), Some(now));

    let _ = helpers::get_author(&manager, author_id);
    manager.bind(author_id).delete().unwrap();

    let authors = helpers::list_authors(&manager);
    assert!(authors.iter().all(|a| a.id != author_id));
}

// ── Binded: Set fields ───────────────────────────────────────

#[test]
fn test_set_author_name() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let author_id = helpers::add_author(&manager, "Original Name".into(), Some(now));

    manager
        .bind(author_id)
        .update(UpdateAuthor::default().name("Updated Name".into()))
        .unwrap();

    let author = helpers::get_author(&manager, author_id);
    assert_eq!(author.name, "Updated Name");
}

#[test]
fn test_set_author_updated() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let author_id = helpers::add_author(&manager, "Test Author".into(), Some(now));

    let new_updated = Utc::now();
    manager
        .bind(author_id)
        .update(UpdateAuthor::default().updated(new_updated))
        .unwrap();

    let author = helpers::get_author(&manager, author_id);
    let diff = (author.updated - new_updated).num_milliseconds().abs();
    assert!(diff < 1000);
}

#[test]
fn test_set_author_thumb() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let author_id = helpers::add_author(&manager, "Test Author".into(), Some(now));
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );
    let file_meta_id = helpers::add_file_meta(
        &manager,
        post_id,
        "thumbnail.jpg".into(),
        "image/jpeg".into(),
        std::collections::HashMap::new(),
    );

    manager
        .bind(author_id)
        .update(UpdateAuthor::default().thumb(Some(file_meta_id)))
        .unwrap();

    let author = helpers::get_author(&manager, author_id);
    assert_eq!(author.thumb, Some(file_meta_id));

    manager
        .bind(author_id)
        .update(UpdateAuthor::default().thumb(None))
        .unwrap();
    let author = helpers::get_author(&manager, author_id);
    assert_eq!(author.thumb, None);
}

#[test]
fn test_set_author_thumb_by_latest() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let author_id = helpers::add_author(&manager, "Test Author".into(), Some(now));
    let post_id = helpers::add_post(
        &manager,
        "Latest Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    let file_meta_id = helpers::add_file_meta(
        &manager,
        post_id,
        "post_thumb.jpg".into(),
        "image/jpeg".into(),
        std::collections::HashMap::new(),
    );

    // Set post thumb and associate author
    manager
        .bind(post_id)
        .update(UpdatePost::default().thumb(Some(file_meta_id)))
        .unwrap();
    helpers::add_post_authors(&manager, post_id, &[author_id]);

    manager
        .bind(author_id)
        .update(UpdateAuthor::default().thumb_by_latest())
        .unwrap();

    let author = helpers::get_author(&manager, author_id);
    assert_eq!(author.thumb, Some(file_meta_id));
}

#[test]
fn test_set_author_updated_by_latest() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let author_id = helpers::add_author(&manager, "Test Author".into(), Some(now));

    let later_time = Utc::now();
    let post_id = helpers::add_post(
        &manager,
        "Latest Post".into(),
        None,
        None,
        Some(later_time),
        Some(later_time),
    );
    helpers::add_post_authors(&manager, post_id, &[author_id]);

    manager
        .bind(author_id)
        .update(UpdateAuthor::default().updated_by_latest())
        .unwrap();

    let author = helpers::get_author(&manager, author_id);
    let diff = (author.updated - later_time).num_milliseconds().abs();
    assert!(diff < 1000);
}

// ── Binded: Alias management ─────────────────────────────────

#[test]
fn test_add_author_aliases() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let author_id = helpers::add_author(&manager, "Test Author".into(), Some(now));
    let platform_id = helpers::add_platform(&manager, "Test Platform".into());

    let aliases = vec![
        (
            "alias1".into(),
            platform_id,
            Some("http://example.com/alias1".into()),
        ),
        ("alias2".into(), platform_id, None),
    ];

    manager.bind(author_id).add_aliases(aliases).unwrap();

    let stored = helpers::list_author_aliases(&manager, author_id);
    assert_eq!(stored.len(), 2);
    let sources: Vec<String> = stored.iter().map(|a| a.source.clone()).collect();
    assert!(sources.contains(&"alias1".to_string()));
    assert!(sources.contains(&"alias2".to_string()));
}

#[test]
fn test_remove_author_aliases() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let platform_id = helpers::add_platform(&manager, "Test Platform".into());
    let author_id = helpers::add_author(&manager, "Test Author".into(), Some(now));

    helpers::add_author_aliases(
        &manager,
        author_id,
        vec![
            ("keep".into(), platform_id, None),
            ("remove".into(), platform_id, None),
        ],
    );

    manager
        .bind(author_id)
        .remove_aliases(&[("remove".into(), platform_id)])
        .unwrap();

    let remaining = helpers::list_author_aliases(&manager, author_id);
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].source, "keep");
}

#[test]
fn test_list_author_aliases() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let platform1 = helpers::add_platform(&manager, "Platform 1".into());
    let platform2 = helpers::add_platform(&manager, "Platform 2".into());
    let author_id = helpers::add_author(&manager, "Test Author".into(), Some(now));

    helpers::add_author_aliases(
        &manager,
        author_id,
        vec![
            (
                "alias1".into(),
                platform1,
                Some("http://example.com/alias1".into()),
            ),
            ("alias2".into(), platform1, None),
            (
                "alias3".into(),
                platform2,
                Some("http://example.com/alias3".into()),
            ),
        ],
    );

    let stored = manager.bind(author_id).list_aliases().unwrap();
    assert_eq!(stored.len(), 3);

    let sources: Vec<String> = stored.iter().map(|a| a.source.clone()).collect();
    assert!(sources.contains(&"alias1".to_string()));
    assert!(sources.contains(&"alias2".to_string()));
    assert!(sources.contains(&"alias3".to_string()));

    for alias in &stored {
        assert_eq!(alias.target, author_id);
    }

    let alias1 = stored.iter().find(|a| a.source == "alias1").unwrap();
    assert_eq!(alias1.link, Some("http://example.com/alias1".to_string()));
    assert_eq!(alias1.platform, platform1);

    let alias2 = stored.iter().find(|a| a.source == "alias2").unwrap();
    assert_eq!(alias2.link, None);
    assert_eq!(alias2.platform, platform1);

    let alias3 = stored.iter().find(|a| a.source == "alias3").unwrap();
    assert_eq!(alias3.link, Some("http://example.com/alias3".to_string()));
    assert_eq!(alias3.platform, platform2);
}

#[test]
fn test_set_author_alias_name() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let platform_id = helpers::add_platform(&manager, "Test Platform".into());
    let author_id = helpers::add_author(&manager, "Test Author".into(), Some(now));

    helpers::add_author_aliases(
        &manager,
        author_id,
        vec![("old_name".into(), platform_id, None)],
    );

    let old_alias = ("old_name".into(), platform_id);
    manager
        .bind(author_id)
        .set_alias_name(&old_alias, "new_name".into())
        .unwrap();

    let aliases = helpers::list_author_aliases(&manager, author_id);
    assert_eq!(aliases.len(), 1);
    assert_eq!(aliases[0].source, "new_name");
    assert_eq!(aliases[0].platform, platform_id);
}

#[test]
fn test_set_author_alias_platform() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let platform1 = helpers::add_platform(&manager, "Platform 1".into());
    let platform2 = helpers::add_platform(&manager, "Platform 2".into());
    let author_id = helpers::add_author(&manager, "Test Author".into(), Some(now));

    helpers::add_author_aliases(
        &manager,
        author_id,
        vec![("test_alias".into(), platform1, None)],
    );

    let old_alias = ("test_alias".into(), platform1);
    manager
        .bind(author_id)
        .set_alias_platform(&old_alias, platform2)
        .unwrap();

    let aliases = helpers::list_author_aliases(&manager, author_id);
    assert_eq!(aliases.len(), 1);
    assert_eq!(aliases[0].source, "test_alias");
    assert_eq!(aliases[0].platform, platform2);
}

#[test]
fn test_set_author_alias_link() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let platform_id = helpers::add_platform(&manager, "Test Platform".into());
    let author_id = helpers::add_author(&manager, "Test Author".into(), Some(now));

    helpers::add_author_aliases(
        &manager,
        author_id,
        vec![("test_alias".into(), platform_id, None)],
    );

    let alias = ("test_alias".into(), platform_id);
    let new_link = Some("http://example.com/new_link".to_string());

    manager
        .bind(author_id)
        .set_alias_link(&alias, new_link.clone())
        .unwrap();

    let aliases = helpers::list_author_aliases(&manager, author_id);
    assert_eq!(aliases[0].link, new_link);

    manager
        .bind(author_id)
        .set_alias_link(&alias, None)
        .unwrap();

    let aliases = helpers::list_author_aliases(&manager, author_id);
    assert_eq!(aliases[0].link, None);
}

// ── Binded: Post relationships ───────────────────────────────

#[test]
fn test_author_post_relationships() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let now = Utc::now();
    let author_id = helpers::add_author(&manager, "Test Author".into(), Some(now));
    let post_id = helpers::add_post(
        &manager,
        "Test Post".into(),
        None,
        None,
        Some(now),
        Some(now),
    );

    helpers::add_post_authors(&manager, post_id, &[author_id]);

    // Test author's posts via Binded
    let post_ids = manager.bind(author_id).list_posts().unwrap();
    assert_eq!(post_ids.len(), 1);
    assert_eq!(post_ids[0], post_id);

    // Test post's authors via helpers
    let post_authors = helpers::list_post_authors(&manager, post_id);
    assert_eq!(post_authors.len(), 1);
    assert_eq!(post_authors[0].id, author_id);
}
