//! Tests for strongly-typed ID types
//!
//! Comprehensive tests for all ID types including creation, conversion,
//! serialization, and trait implementations.

use crate::{AuthorId, CollectionId, FileMetaId, PlatformId, PostId, TagId};

#[test]
fn test_id_creation() {
    let author_id = AuthorId::new(42);
    assert_eq!(author_id.raw(), 42);
    assert_eq!(*author_id, 42);

    let post_id = PostId::new(123);
    assert_eq!(post_id.raw(), 123);
    assert_eq!(*post_id, 123);
}

#[test]
fn test_id_from_u32() {
    let author_id = AuthorId::from(100u32);
    assert_eq!(author_id.raw(), 100);

    let tag_id: TagId = 200u32.into();
    assert_eq!(tag_id.raw(), 200);
}

#[test]
fn test_id_from_usize() {
    let platform_id = PlatformId::from(300usize);
    assert_eq!(platform_id.raw(), 300);

    let file_id: FileMetaId = 400usize.into();
    assert_eq!(file_id.raw(), 400);
}

#[test]
fn test_id_to_u32() {
    let collection_id = CollectionId::new(500);
    let value: u32 = collection_id.into();
    assert_eq!(value, 500);
}

#[test]
fn test_id_to_usize() {
    let author_id = AuthorId::new(600);
    let value: usize = author_id.into();
    assert_eq!(value, 600);
}

#[test]
fn test_id_display() {
    let post_id = PostId::new(777);
    assert_eq!(format!("{}", post_id), "777");
    assert_eq!(post_id.to_string(), "777");
}

#[test]
fn test_id_equality() {
    let id1 = TagId::new(888);
    let id2 = TagId::new(888);
    let id3 = TagId::new(999);

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
    assert_ne!(id2, id3);
}

#[test]
fn test_id_hash() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    let platform_id = PlatformId::new(111);

    map.insert(platform_id, "test_platform");
    assert_eq!(map.get(&platform_id), Some(&"test_platform"));
    assert_eq!(map.get(&PlatformId::new(111)), Some(&"test_platform"));
    assert_eq!(map.get(&PlatformId::new(222)), None);
}

#[test]
fn test_id_as_ref() {
    let file_id = FileMetaId::new(333);
    let reference: &u32 = file_id.as_ref();
    assert_eq!(*reference, 333);
}

#[test]
fn test_id_deref() {
    let collection_id = CollectionId::new(444);
    assert_eq!(*collection_id, 444);

    // Test that we can use deref in comparisons
    assert!(*collection_id > 400);
    assert!(*collection_id < 500);
}

#[test]
fn test_id_clone_copy() {
    let original = AuthorId::new(555);
    let cloned = original.clone();
    let copied = original;

    assert_eq!(original, cloned);
    assert_eq!(original, copied);
    assert_eq!(cloned, copied);
}

#[test]
fn test_id_serialization() {
    let post_id = PostId::new(666);

    // Test JSON serialization
    let json = serde_json::to_string(&post_id).expect("Failed to serialize");
    assert_eq!(json, "666");

    // Test JSON deserialization
    let deserialized: PostId = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized, post_id);
}

#[test]
fn test_all_id_types() {
    // Test that all ID types work consistently
    let author_id = AuthorId::new(1);
    let post_id = PostId::new(2);
    let tag_id = TagId::new(3);
    let platform_id = PlatformId::new(4);
    let file_id = FileMetaId::new(5);
    let collection_id = CollectionId::new(6);

    assert_eq!(author_id.raw(), 1);
    assert_eq!(post_id.raw(), 2);
    assert_eq!(tag_id.raw(), 3);
    assert_eq!(platform_id.raw(), 4);
    assert_eq!(file_id.raw(), 5);
    assert_eq!(collection_id.raw(), 6);
}

#[cfg(feature = "utils")]
#[test]
fn test_id_rusqlite_integration() {
    use rusqlite::{params, Connection};

    let conn = Connection::open_in_memory().expect("Failed to create in-memory DB");

    // Create a test table
    conn.execute(
        "CREATE TABLE test_ids (
            author_id INTEGER,
            post_id INTEGER,
            tag_id INTEGER
        )",
        [],
    )
    .expect("Failed to create table");

    let author_id = AuthorId::new(100);
    let post_id = PostId::new(200);
    let tag_id = TagId::new(300);

    // Insert IDs
    conn.execute(
        "INSERT INTO test_ids (author_id, post_id, tag_id) VALUES (?, ?, ?)",
        params![author_id, post_id, tag_id],
    )
    .expect("Failed to insert");

    // Query IDs back
    let mut stmt = conn
        .prepare("SELECT author_id, post_id, tag_id FROM test_ids")
        .expect("Failed to prepare");
    let row = stmt
        .query_row([], |row| {
            Ok((
                row.get::<_, AuthorId>(0)?,
                row.get::<_, PostId>(1)?,
                row.get::<_, TagId>(2)?,
            ))
        })
        .expect("Failed to query");

    assert_eq!(row.0, author_id);
    assert_eq!(row.1, post_id);
    assert_eq!(row.2, tag_id);
}
