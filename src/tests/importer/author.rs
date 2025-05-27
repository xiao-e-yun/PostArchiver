use crate::{
    importer::{ImportFileMetaMethod, UnsyncAuthor, UnsyncFileMeta, UnsyncPost},
    manager::PostArchiverManager,
    FileMetaId, Link,
};
use chrono::{Duration, TimeZone, Utc};
use std::collections::HashMap;

#[test]
fn test_import_author() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let author =
        UnsyncAuthor::new("octocat".to_string()).aliases(vec!["github:octocat".to_string()]);

    // Import the author
    let id = manager.import_author(&author).unwrap();

    // Get the author
    let saved_author = manager.get_author(&id).unwrap();
    assert_eq!(saved_author.name, "octocat");

    // Update the author
    let author = author.name("octocatdog".to_string());
    let id = manager.import_author(&author).unwrap();

    // get author
    let author = manager.get_author(&id).unwrap();
    assert_eq!(author.name, "octocatdog");
}

#[test]
fn test_import_author_by_part() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let author = UnsyncAuthor::new("octocat".to_string());

    // Import the author
    let id = manager.import_author_by_create(&author).unwrap();

    // Get the author
    let saved_author = manager.get_author(&id).unwrap();
    assert_eq!(saved_author.name, "octocat");

    // Update the author
    let author = author.name("octocatdog".to_string());
    let id = manager.import_author_by_update(id, &author).unwrap();

    // get author
    let author = manager.get_author(&id).unwrap();
    assert_eq!(author.name, "octocatdog");
}

#[test]
fn test_sync_author() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author and import it
    let updated = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();

    UnsyncAuthor::new("octocat".to_string())
        .aliases(vec!["github:octocat".to_string(), "x:octocat".to_string()])
        .links(vec![Link::new("github", "https://octodex.github.com/")])
        .updated(Some(updated))
        .sync(&manager)
        .unwrap();
}

#[test]
fn test_set_author() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author and import it
    let updated = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();

    let author = UnsyncAuthor::new("octocat".to_string())
        .aliases(vec!["github:octocat".to_string(), "x:octocat".to_string()])
        .links(vec![Link::new("github", "https://octodex.github.com/")])
        .updated(Some(updated))
        .sync(&manager)
        .unwrap();

    // New author
    let name = "octocatdog".to_string();
    manager.set_author_name(&author.id, &name).unwrap();
    assert_eq!(manager.get_author(&author.id).unwrap().name, name);

    let aliases = vec!["x:octocat".to_string(), "stackoverflow:octocat".to_string()];
    manager
        .set_author_aliases_by_merge(&author.id, &aliases)
        .unwrap();
    assert_eq!(manager.get_author_aliases(&author.id).unwrap().len(), 3);

    // new links
    let links = vec![
        Link::new("github", "https://octodex.github.com/"),
        Link::new("example", "https://example.com/"),
    ];
    manager
        .set_author_links_by_merge(author.id, &links)
        .unwrap();
    assert_eq!(manager.get_author(&author.id).unwrap().links.len(), 2);

    let links = vec![Link::new("example", "https://example.com/")];
    manager.set_author_links(author.id, &links).unwrap();
    assert_eq!(manager.get_author(&author.id).unwrap().links.len(), 1);

    // new updated
    let updated = updated + Duration::seconds(1);
    manager.set_author_updated(author.id, &updated).unwrap();
    assert_eq!(manager.get_author(&author.id).unwrap().updated, updated);

    // Create a post and thumb
    // sync UnsyncPost will update the author thumb and updated time
    let (post, _) = UnsyncPost::new(author.id)
        .thumb(Some(UnsyncFileMeta {
            filename: "thumb.png".to_string(),
            mime: "image/png".to_string(),
            extra: HashMap::new(),
            method: ImportFileMetaMethod::None,
        }))
        .sync(&manager)
        .unwrap();

    assert_eq!(
        manager.get_author(&author.id).unwrap().thumb,
        Some(FileMetaId(1))
    );

    // Set the author thumb to None
    manager.set_author_thumb(author.id, None).unwrap();
    assert_eq!(manager.get_author(&author.id).unwrap().thumb, None);

    assert_ne!(manager.get_author(&author.id).unwrap().updated, updated);

    // Set the post updated to old time
    manager.set_post_updated(post.id, &updated).unwrap();
    manager.set_author_updated_by_latest(author.id).unwrap();

    assert_eq!(manager.get_author(&author.id).unwrap().updated, updated);
}
