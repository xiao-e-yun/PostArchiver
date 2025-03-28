use crate::{
    importer::{ImportFileMetaMethod, UnsyncAuthor, UnsyncFileMeta, UnsyncPost},
    manager::PostArchiverManager,
    Link,
};
use chrono::{Duration, Utc};
use std::collections::HashMap;

#[test]
fn test_check_author() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Check if the author not exists
    let id = manager
        .check_author(&["github:octocat".to_string()])
        .unwrap();

    assert_eq!(id, None);

    // Create an author
    let (author, _) = UnsyncAuthor::new("octocat".to_string())
        .alias(vec!["github:octocat".to_string()])
        .sync(&manager)
        .unwrap();

    // Check if the author exists
    let id = manager
        .check_author(&["github:octocat".to_string()])
        .unwrap();

    assert_eq!(id, Some(author.id));
}

#[test]
fn test_import_author() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let author = UnsyncAuthor::new("octocat".to_string()).alias(vec!["github:octocat".to_string()]);

    // Import the author
    let id = manager.import_author(&author).unwrap();

    // Get the author
    let saved_author = manager.get_author(&id).unwrap();
    assert_eq!(saved_author.name, "octocat");
}

#[test]
fn test_import_author_by_create() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let author = UnsyncAuthor::new("octocat".to_string()).alias(vec!["github:octocat".to_string()]);

    // Import the author
    let id = manager.import_author_by_create(&author).unwrap();

    // Get the author
    let saved_author = manager.get_author(&id).unwrap();
    assert_eq!(saved_author.name, "octocat");
}

#[test]
fn test_import_author_by_update() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let author = UnsyncAuthor::new("octocat".to_string()).alias(vec!["github:octocat".to_string()]);

    // Import the author
    let id = manager.import_author(&author).unwrap();

    // Next time, we can just update the author
    let author =
        UnsyncAuthor::new("octocatdog".to_string()).alias(vec!["github:octocatdog".to_string()]);

    // Update the author
    let id = manager.import_author_by_update(id, &author).unwrap();

    // get author
    let author = manager.get_author(&id).unwrap();
    assert_eq!(author.name, "octocatdog");
}

#[test]
fn test_get_author() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author and import it
    let (author, _) = UnsyncAuthor::new("octocat".to_string())
        .alias(vec!["github:octocat".to_string()])
        .sync(&manager)
        .unwrap();

    // Get the author by their id
    let saved_author = manager.get_author(&author.id).unwrap();
    assert_eq!(saved_author.name, "octocat");
}

#[test]
fn test_get_author_alias() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author and import it
    let (author, _) = UnsyncAuthor::new("octocat".to_string())
        .alias(vec!["github:octocat".to_string(), "x:octocat".to_string()])
        .sync(&manager)
        .unwrap();

    // Get the author by their id
    let alias = manager.get_author_alias(&author.id).unwrap();

    assert_eq!(alias.len(), 2);
}

#[test]
fn test_set_author_alias_by_merge() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author and import it
    let (author, _) = UnsyncAuthor::new("octocat".to_string())
        .alias(vec!["github:octocat".to_string(), "x:octocat".to_string()])
        .sync(&manager)
        .unwrap();

    // new alias
    let alias = vec!["x:octocat".to_string(), "stackoverflow:octocat".to_string()];

    // Merge the author alias
    manager
        .set_author_alias_by_merge(&author.id, &alias)
        .unwrap();

    // Check the author alias
    let alias = manager.get_author_alias(&author.id).unwrap();
    assert_eq!(alias.len(), 3);
}

#[test]
fn test_set_author_name() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author and import it
    let (author, _) = UnsyncAuthor::new("octocat".to_string())
        .sync(&manager)
        .unwrap();

    // Set the author name
    manager.set_author_name(&author.id, "octocatdog").unwrap();

    // Get the author by their id
    let author = manager.get_author(&author.id).unwrap();
    assert_eq!(author.name, "octocatdog");
}

#[test]
fn test_set_author_thumb() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author and import it
    let (author, _) = UnsyncAuthor::new("octocat".to_string())
        .sync(&manager)
        .unwrap();

    // Create a post and thumb
    let _ = UnsyncPost::new(author.id)
        .thumb(Some(UnsyncFileMeta {
            filename: "thumb.png".to_string(),
            mime: "image/png".to_string(),
            extra: HashMap::new(),
            method: ImportFileMetaMethod::None,
        }))
        .sync(&manager)
        .unwrap();

    // Get the thumb id
    let thumb = manager.get_author(&author.id).unwrap().thumb;
    assert!(thumb.is_some());

    // Set the author thumb
    manager.set_author_thumb(author.id, None).unwrap();

    // Get the thumb again
    let thumb = manager.get_author(&author.id).unwrap().thumb;
    assert_eq!(thumb, None);
}

#[test]
fn test_set_author_thumb_by_latest() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author and import it
    let (author, _) = UnsyncAuthor::new("octocat".to_string())
        .sync(&manager)
        .unwrap();

    // Define updated time
    let updated = Utc::now();

    // Create a post and thumb
    let (first_post, _) = UnsyncPost::new(author.id)
        .thumb(Some(UnsyncFileMeta {
            filename: "thumb.png".to_string(),
            mime: "image/png".to_string(),
            extra: HashMap::new(),
            method: ImportFileMetaMethod::None,
        }))
        .updated(updated)
        .sync(&manager)
        .unwrap();

    // Create another post and thumb, to update the author thumb
    let (second_post, _) = UnsyncPost::new(author.id)
        .thumb(Some(UnsyncFileMeta {
            filename: "thumb2.png".to_string(),
            mime: "image/png".to_string(),
            extra: HashMap::new(),
            method: ImportFileMetaMethod::None,
        }))
        .updated(updated + Duration::seconds(1))
        .sync(&manager)
        .unwrap();

    // Get the author thumb
    let thumb = manager.get_author(&author.id).unwrap().thumb;
    assert_eq!(thumb, second_post.thumb);

    // Update the first post updated time
    manager
        .set_post_updated(first_post.id, &(updated + Duration::seconds(2)))
        .unwrap();
    manager.set_author_thumb_by_latest(author.id).unwrap();

    // Get the author thumb
    let thumb = manager.get_author(&author.id).unwrap().thumb;
    assert_eq!(thumb, first_post.thumb);
}

#[test]
fn test_set_author_links() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author and import it
    let links = vec![Link::new("github", "https://octodex.github.com/")];
    let (author, _) = UnsyncAuthor::new("octocat".to_string())
        .links(links.clone())
        .sync(&manager)
        .unwrap();

    assert_eq!(author.links, links);

    // Set the author links
    let links = vec![
        Link::new("example", "https://example.com/"),
        Link::new("example2", "https://example2.com/"),
    ];
    manager.set_author_links(author.id, &links).unwrap();

    // Get the author by their id
    let author = manager.get_author(&author.id).unwrap();
    assert_eq!(author.links, links);
}

#[test]
fn test_set_author_links_by_merge() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author and import it
    let (author, _) = UnsyncAuthor::new("octocat".to_string())
        .links(vec![Link::new("github", "https://octodex.github.com/")])
        .sync(&manager)
        .unwrap();

    assert_eq!(author.links.len(), 1);

    // Set the author links
    manager
        .set_author_links_by_merge(
            author.id,
            &vec![
                Link::new("example", "https://example.com/"),
                Link::new("example2", "https://example2.com/"),
            ],
        )
        .unwrap();

    // Get the author by their id
    let author = manager.get_author(&author.id).unwrap();

    assert_eq!(author.links.len(), 3);
}

#[test]
fn test_set_author_updated() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let updated = Utc::now();

    // Create an author and import it
    let (author, _) = UnsyncAuthor::new("octocat".to_string())
        .updated(Some(updated))
        .sync(&manager)
        .unwrap();

    assert_eq!(author.updated, updated);

    // Set the author updated time to next second
    let updated = updated + Duration::seconds(1);
    manager.set_author_updated(author.id, &updated).unwrap();

    // Get the author by their id
    let author = manager.get_author(&author.id).unwrap();
    assert_eq!(author.updated, updated);
}

#[test]
fn test_set_author_updated_by_latest() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Define updated time
    let updated = Utc::now();

    // Create an author and import it
    let (author, _) = UnsyncAuthor::new("octocat".to_string())
        .sync(&manager)
        .unwrap();

    // Create a post
    let (first_post, _) = UnsyncPost::new(author.id)
        .updated(updated)
        .sync(&manager)
        .unwrap();

    // Create another post, to update the author updated time
    let (second_post, _) = UnsyncPost::new(author.id)
        .updated(updated + Duration::seconds(1))
        .sync(&manager)
        .unwrap();

    // Get the author updated
    let author_updated = manager.get_author(&author.id).unwrap().updated;
    assert_eq!(author_updated, second_post.updated);

    // Update the first post updated time
    let new_updated = updated + Duration::seconds(2);
    manager
        .set_post_updated(first_post.id, &new_updated)
        .unwrap();
    manager.set_author_updated_by_latest(author.id).unwrap();

    // Get the author updated
    let author_updated = manager.get_author(&author.id).unwrap().updated;
    assert_eq!(author_updated, new_updated);
}

#[test]
fn test_sync_author() {
    // Open a manager
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Create an author
    let (author, _) = UnsyncAuthor::new("octocat".to_string())
        .alias(vec!["github:octocat".to_string()])
        .links(vec![Link::new("github", "https://octodex.github.com/")])
        .updated(Some(Utc::now()))
        .sync(&manager)
        .unwrap();

    let archived_author = manager.get_author(&author.id).unwrap();
    assert_eq!(author, archived_author);
}
