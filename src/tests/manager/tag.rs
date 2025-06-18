//! Tag manager tests
//!
//! Tests for tag CRUD operations, platform associations,
//! and tag-post relationships.

use crate::{manager::PostArchiverManager, TagId};
use chrono::Utc;

#[test]
fn test_add_tag() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let name = "test_tag".to_string();
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    let tag_id = manager
        .add_tag(name.clone(), Some(platform_id))
        .expect("Failed to add tag");

    assert!(tag_id.raw() > 0);

    // Verify the tag was added
    let tag = manager
        .get_tag(&tag_id)
        .expect("Failed to get tag")
        .unwrap();

    assert_eq!(tag.name, name);
    assert_eq!(tag.id, tag_id);
    assert_eq!(tag.platform, Some(platform_id));
}

#[test]
fn test_add_tag_no_platform() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let name = "global_tag".to_string();

    let tag_id = manager
        .add_tag(name.clone(), None)
        .expect("Failed to add tag without platform");

    let tag = manager
        .get_tag(&tag_id)
        .expect("Failed to get tag")
        .unwrap();

    assert_eq!(tag.name, name);
    assert_eq!(tag.platform, None);
}

#[test]
fn test_list_tags() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    // Add multiple tags
    let id1 = manager
        .add_tag("tag1".to_string(), Some(platform_id))
        .expect("Failed to add tag 1");
    let id2 = manager
        .add_tag("tag2".to_string(), None)
        .expect("Failed to add tag 2");

    let tags = manager.list_tags().expect("Failed to list tags");

    assert_eq!(tags.len(), 2);

    let ids: Vec<TagId> = tags.iter().map(|t| t.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

#[test]
fn test_get_tag() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let name = "get_test_tag".to_string();
    let tag_id = manager
        .add_tag(name.clone(), None)
        .expect("Failed to add tag");

    let tag = manager
        .get_tag(&tag_id)
        .expect("Failed to get tag")
        .unwrap();

    assert_eq!(tag.id, tag_id);
    assert_eq!(tag.name, name);
}

#[test]
fn test_get_nonexistent_tag() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let nonexistent_id = TagId(999);

    let result = manager
        .get_tag(&nonexistent_id)
        .expect("Failed to query tag");
    assert!(result.is_none());
}

#[test]
fn test_remove_tag() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let tag_id = manager
        .add_tag("to_delete".to_string(), None)
        .expect("Failed to add tag");

    // Verify tag exists
    let tag = manager.get_tag(&tag_id).expect("Failed to get tag");
    assert!(tag.is_some());

    // Remove tag
    manager.remove_tag(&tag_id).expect("Failed to remove tag");

    // Verify tag is gone
    let result = manager.get_tag(&tag_id).expect("Failed to query tag");
    assert!(result.is_none());
}

#[test]
fn test_set_tag_name() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let tag_id = manager
        .add_tag("original_name".to_string(), None)
        .expect("Failed to add tag");

    let new_name = "updated_name".to_string();
    manager
        .set_tag_name(&tag_id, new_name.clone())
        .expect("Failed to update tag name");

    let tag = manager
        .get_tag(&tag_id)
        .expect("Failed to get tag")
        .unwrap();

    assert_eq!(tag.name, new_name);
}

#[test]
fn test_set_tag_platform() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform1_id = manager
        .add_platform("Platform 1".to_string())
        .expect("Failed to add platform 1");
    let platform2_id = manager
        .add_platform("Platform 2".to_string())
        .expect("Failed to add platform 2");

    let tag_id = manager
        .add_tag("test_tag".to_string(), Some(platform1_id))
        .expect("Failed to add tag");

    // Update platform
    manager
        .set_tag_platform(&tag_id, Some(platform2_id))
        .expect("Failed to update tag platform");

    let tag = manager
        .get_tag(&tag_id)
        .expect("Failed to get tag")
        .unwrap();

    assert_eq!(tag.platform, Some(platform2_id));

    // Set platform to None
    manager
        .set_tag_platform(&tag_id, None)
        .expect("Failed to set tag platform to None");

    let tag = manager
        .get_tag(&tag_id)
        .expect("Failed to get tag")
        .unwrap();

    assert_eq!(tag.platform, None);
}

#[test]
fn test_find_tag_by_string() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let tag_id = manager
        .add_tag("findme".to_string(), None)
        .expect("Failed to add tag");

    // Find by string (no platform)
    let found_id = manager.find_tag(&"findme").expect("Failed to find tag");

    assert_eq!(found_id, Some(tag_id));

    // Test not found
    let not_found = manager
        .find_tag(&"nonexistent")
        .expect("Failed to search for nonexistent tag");

    assert_eq!(not_found, None);
}

#[test]
fn test_find_tag_by_name_and_platform() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    let tag_id = manager
        .add_tag("platform_tag".to_string(), Some(platform_id))
        .expect("Failed to add tag");

    // Find by name and platform
    let found_id = manager
        .find_tag(&("platform_tag", platform_id))
        .expect("Failed to find tag");

    assert_eq!(found_id, Some(tag_id));

    // Test not found with different platform
    let other_platform_id = manager
        .add_platform("Other Platform".to_string())
        .expect("Failed to add other platform");

    let not_found = manager
        .find_tag(&("platform_tag", other_platform_id))
        .expect("Failed to search for tag with different platform");

    assert_eq!(not_found, None);
}

#[test]
fn test_find_tag_by_name_and_optional_platform() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = manager
        .add_platform("Test Platform".to_string())
        .expect("Failed to add platform");

    let tag_id = manager
        .add_tag("optional_platform_tag".to_string(), Some(platform_id))
        .expect("Failed to add tag");

    // Find by name and Some(platform)
    let found_id = manager
        .find_tag(&("optional_platform_tag", Some(platform_id)))
        .expect("Failed to find tag");

    assert_eq!(found_id, Some(tag_id));

    // Test with None platform
    let tag_id_none = manager
        .add_tag("no_platform_tag".to_string(), None)
        .expect("Failed to add tag without platform");

    let found_id_none = manager
        .find_tag(&("no_platform_tag", None))
        .expect("Failed to find tag with None platform");

    assert_eq!(found_id_none, Some(tag_id_none));
}

#[test]
fn test_post_tag_relationships() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    // Setup tags
    let tag1_id = manager
        .add_tag("tag1".to_string(), None)
        .expect("Failed to add tag1");
    let tag2_id = manager
        .add_tag("tag2".to_string(), None)
        .expect("Failed to add tag2");

    // Setup post
    let post_id = manager
        .add_post(
            "Test Post".to_string(),
            None,
            None,
            Some(Utc::now()),
            Some(Utc::now()),
        )
        .expect("Failed to add post");

    // Add tags to post
    manager
        .add_post_tags(post_id, &[tag1_id, tag2_id])
        .expect("Failed to add post tags");

    // Test post's tags
    let post_tags = manager
        .list_post_tags(&post_id)
        .expect("Failed to list post tags");

    assert_eq!(post_tags.len(), 2);
    let tag_ids: Vec<TagId> = post_tags.iter().map(|t| t.id).collect();
    assert!(tag_ids.contains(&tag1_id));
    assert!(tag_ids.contains(&tag2_id));

    // Test tag's posts
    let tag1_posts = manager
        .list_tag_posts(&tag1_id)
        .expect("Failed to list tag posts");

    assert_eq!(tag1_posts.len(), 1);
    assert_eq!(tag1_posts[0].id, post_id);
}

#[test]
fn test_post_tags_method() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let tag_id = manager
        .add_tag("method_tag".to_string(), None)
        .expect("Failed to add tag");

    let post_id = manager
        .add_post(
            "Method Test Post".to_string(),
            None,
            None,
            Some(Utc::now()),
            Some(Utc::now()),
        )
        .expect("Failed to add post");

    manager
        .add_post_tags(post_id, &[tag_id])
        .expect("Failed to add post tags");

    let post = manager.get_post(&post_id).expect("Failed to get post");
    let tags = post.tags(&manager).expect("Failed to get post tags");

    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].id, tag_id);
}

#[test]
fn test_tag_posts_method() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let tag_id = manager
        .add_tag("method_tag".to_string(), None)
        .expect("Failed to add tag");

    let post_id = manager
        .add_post(
            "Method Test Post".to_string(),
            None,
            None,
            Some(Utc::now()),
            Some(Utc::now()),
        )
        .expect("Failed to add post");

    manager
        .add_post_tags(post_id, &[tag_id])
        .expect("Failed to add post tags");

    let tag = manager
        .get_tag(&tag_id)
        .expect("Failed to get tag")
        .unwrap();
    let posts = tag.posts(&manager).expect("Failed to get tag posts");

    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].id, post_id);
}

#[test]
fn test_tag_cache_functionality() {
    let manager = PostArchiverManager::open_in_memory().unwrap();
    let platform_id = manager
        .add_platform("Cache Test Platform".to_string())
        .expect("Failed to add platform");

    let tag_name = "cached_tag".to_string();

    // First call should cache the tag
    let tag_id1 = manager
        .add_tag(tag_name.clone(), Some(platform_id))
        .expect("Failed to add tag");

    // Second call should return the same ID from cache
    let tag_id2 = manager
        .add_tag(tag_name.clone(), Some(platform_id))
        .expect("Failed to add tag again");

    assert_eq!(tag_id1, tag_id2);

    // Find should also use cache
    let found_id = manager
        .find_tag(&(tag_name.as_str(), platform_id))
        .expect("Failed to find cached tag");

    assert_eq!(found_id, Some(tag_id1));
}

#[test]
fn test_remove_tag_with_post_associations() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let tag_id = manager
        .add_tag("associated_tag".to_string(), None)
        .expect("Failed to add tag");

    let post_id = manager
        .add_post(
            "Associated Post".to_string(),
            None,
            None,
            Some(Utc::now()),
            Some(Utc::now()),
        )
        .expect("Failed to add post");

    manager
        .add_post_tags(post_id, &[tag_id])
        .expect("Failed to add post tags");

    // Verify association exists
    let post_tags = manager
        .list_post_tags(&post_id)
        .expect("Failed to list post tags");
    assert_eq!(post_tags.len(), 1);

    // Remove tag
    manager.remove_tag(&tag_id).expect("Failed to remove tag");

    // Verify tag is gone
    let result = manager.get_tag(&tag_id).expect("Failed to query tag");
    assert!(result.is_none());

    // Verify associations are also gone
    let post_tags_after = manager
        .list_post_tags(&post_id)
        .expect("Failed to list post tags after tag removal");
    assert_eq!(post_tags_after.len(), 0);
}

#[test]
fn test_unique_tag_names_across_platforms() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let platform1_id = manager
        .add_platform("Platform 1".to_string())
        .expect("Failed to add platform 1");
    let platform2_id = manager
        .add_platform("Platform 2".to_string())
        .expect("Failed to add platform 2");

    let tag_name = "unique_name".to_string();

    // Add tag with platform1
    let tag1_id = manager
        .add_tag(tag_name.clone(), Some(platform1_id))
        .expect("Failed to add tag 1");

    // Try to add tag with same name but different platform - should fail due to UNIQUE constraint
    let result = manager.add_tag(tag_name.clone(), Some(platform2_id));
    assert!(
        result.is_err(),
        "Adding tag with same name should fail due to UNIQUE constraint"
    );

    // Try to add tag with same name but no platform - should also fail
    let result = manager.add_tag(tag_name.clone(), None);
    assert!(
        result.is_err(),
        "Adding tag with same name should fail due to UNIQUE constraint"
    );

    // Verify the original tag still exists and can be found
    let found1 = manager
        .find_tag(&(tag_name.as_str(), platform1_id))
        .expect("Failed to find tag 1");
    assert_eq!(found1, Some(tag1_id));
}

#[test]
fn test_empty_tag_lists() {
    let manager = PostArchiverManager::open_in_memory().unwrap();

    let tag_id = manager
        .add_tag("lonely_tag".to_string(), None)
        .expect("Failed to add tag");

    let post_id = manager
        .add_post(
            "Lonely Post".to_string(),
            None,
            None,
            Some(Utc::now()),
            Some(Utc::now()),
        )
        .expect("Failed to add post");

    // Test empty lists
    let tag_posts = manager
        .list_tag_posts(&tag_id)
        .expect("Failed to list tag posts");
    assert_eq!(tag_posts.len(), 0);

    let post_tags = manager
        .list_post_tags(&post_id)
        .expect("Failed to list post tags");
    assert_eq!(post_tags.len(), 0);
}
