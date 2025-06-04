//! Test database setup and utilities
//!
//! Provides helper functions for creating isolated test databases
//! and setting up test fixtures.

#[cfg(feature = "utils")]
use crate::manager::PostArchiverManager;

/// Creates an in-memory test database with all tables initialized
#[cfg(feature = "utils")]
pub fn setup_test_db() -> Result<PostArchiverManager, rusqlite::Error> {
    let manager = PostArchiverManager::open_in_memory()?;
    Ok(manager)
}

/// Creates a test database and runs the provided test function
#[cfg(feature = "utils")]
pub fn with_test_db<F, R>(test_fn: F) -> R
where
    F: FnOnce(&PostArchiverManager) -> R,
{
    let manager = setup_test_db().expect("Failed to setup test database");
    test_fn(&manager)
}

/// Creates a test database with sample data and runs the provided test function
#[cfg(feature = "utils")]
pub fn with_populated_test_db<F, R>(test_fn: F) -> R
where
    F: FnOnce(&PostArchiverManager) -> R,
{
    let manager = setup_test_db().expect("Failed to setup test database");

    // Add some sample data for relationship testing
    populate_test_data(&manager).expect("Failed to populate test data");

    test_fn(&manager)
}

/// Populates the test database with sample data
#[cfg(feature = "utils")]
fn populate_test_data(manager: &PostArchiverManager) -> Result<(), rusqlite::Error> {
    use crate::{AuthorId, PlatformId, PostId, TagId};
    use chrono::Utc;

    // Add test platforms
    let _platform1 = manager.add_platform("Test Platform 1".to_string())?;
    let _platform2 = manager.add_platform("Test Platform 2".to_string())?;

    // Add test authors
    let _author1 = manager.add_author("Test Author 1".to_string(), Some(Utc::now()))?;
    let _author2 = manager.add_author("Test Author 2".to_string(), Some(Utc::now()))?;

    // Add test tags
    let _tag1 = manager.add_tag("test-tag-1".to_string(), None)?;
    let _tag2 = manager.add_tag("test-tag-2".to_string(), None)?;

    // Add test posts
    let _post1 = manager.add_post(
        "Test Post 1".to_string(),
        None,
        None,
        Some(Utc::now()),
        Some(Utc::now()),
    )?;

    Ok(())
}
