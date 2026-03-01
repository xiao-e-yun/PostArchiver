use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde_json::Value;

use crate::utils::{DATABASE_NAME, VERSION};

pub mod binded;
pub use binded::*;

pub mod author;
pub use author::{AuthorThumb, AuthorUpdated, UpdateAuthor};

pub mod collection;
pub use collection::{CollectionThumb, UpdateCollection};

pub mod file_meta;
pub use file_meta::{UpdateFileMeta, WritableFileMeta};

pub mod platform;
pub use platform::UpdatePlatform;

pub mod post;
pub use post::{PostUpdated, UpdatePost};

pub mod tag;
pub use tag::UpdateTag;

/// Core manager type for post archive operations with SQLite backend
///
/// Provides database connection management and access to entity operations
/// through the [`Binded`] type via [`bind()`](PostArchiverManager::bind).
///
/// # Examples
/// ```no_run
/// use post_archiver::manager::PostArchiverManager;
///
/// let manager = PostArchiverManager::open_or_create("./data").unwrap();
/// ```
#[derive(Debug)]
pub struct PostArchiverManager<C = Connection> {
    pub path: PathBuf,
    conn: C,
}

impl PostArchiverManager {
    /// Creates a new archive at the specified path
    ///
    /// # Panics
    /// Panics if the path already contains a database file.
    ///
    /// # Examples
    /// ```no_run
    /// use post_archiver::manager::PostArchiverManager;
    ///
    /// let manager = PostArchiverManager::create("./new_archive").unwrap();
    /// ```
    pub fn create<P>(path: P) -> Result<Self, rusqlite::Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();
        let db_path = path.join(DATABASE_NAME);

        if db_path.exists() {
            panic!("Database already exists");
        }

        let conn = Connection::open(&db_path)?;

        // run the template sql
        conn.execute_batch(include_str!("../utils/template.sql"))?;

        // push current version
        conn.execute(
            "INSERT INTO post_archiver_meta (version) VALUES (?)",
            [VERSION],
        )?;

        Ok(Self { conn, path })
    }

    /// Opens an existing archive at the specified path
    ///
    /// # Returns
    /// - `Ok(Some(manager))` if archive exists and version is compatible
    /// - `Ok(None)` if archive doesn't exist
    /// - `Err(_)` on database errors
    ///
    /// # Examples
    /// ```no_run
    /// use post_archiver::manager::PostArchiverManager;
    ///
    /// if let Some(manager) = PostArchiverManager::open("./archive").unwrap() {
    ///     println!("Archive opened successfully");
    /// }
    /// ```
    pub fn open<P>(path: P) -> Result<Option<Self>, rusqlite::Error>
    where
        P: AsRef<Path>,
    {
        let manager = Self::open_uncheck(path);

        // check version
        if let Ok(Some(manager)) = &manager {
            let version: String = manager
                .conn()
                .query_row("SELECT version FROM post_archiver_meta", [], |row| {
                    row.get(0)
                })
                .unwrap_or("unknown".to_string());

            let get_compatible_version =
                |version: &str| version.splitn(3, ".").collect::<Vec<_>>()[0..2].join(".");

            let match_version = match version.as_str() {
                "unknown" => "unknown".to_string(),
                version => get_compatible_version(version),
            };
            let expect_version = get_compatible_version(VERSION);

            if match_version != expect_version {
                panic!(
                    "Database version mismatch \n + current: {}\n + expected: {}",
                    version, VERSION
                )
            }
        }

        manager
    }

    /// Opens an existing archive at the specified path without checking version.
    ///
    /// # Returns
    /// - `Ok(Some(manager))` if archive exists
    /// - `Ok(None)` if archive doesn't exist
    /// - `Err(_)` on database errors
    pub fn open_uncheck<P>(path: P) -> Result<Option<Self>, rusqlite::Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();
        let db_path = path.join(DATABASE_NAME);

        if !db_path.exists() {
            return Ok(None);
        }

        let conn = Connection::open(db_path)?;

        Ok(Some(Self { conn, path }))
    }

    /// Opens an existing archive or creates a new one if it doesn't exist
    ///
    /// # Examples
    /// ```no_run
    /// use post_archiver::manager::PostArchiverManager;
    ///
    /// let manager = PostArchiverManager::open_or_create("./archive").unwrap();
    /// ```
    pub fn open_or_create<P>(path: P) -> Result<Self, rusqlite::Error>
    where
        P: AsRef<Path>,
    {
        Self::open(&path)
            .transpose()
            .unwrap_or_else(|| Self::create(&path))
    }

    /// Creates an in-memory database
    ///
    /// # Examples
    /// ```
    /// use post_archiver::manager::PostArchiverManager;
    ///
    /// let manager = PostArchiverManager::open_in_memory().unwrap();
    /// ```
    pub fn open_in_memory() -> Result<Self, rusqlite::Error> {
        let path = std::env::temp_dir();

        let conn = Connection::open_in_memory()?;

        // run the template sql
        conn.execute_batch(include_str!("../utils/template.sql"))?;

        // push current version
        conn.execute(
            "INSERT INTO post_archiver_meta (version) VALUES (?)",
            [VERSION],
        )?;

        Ok(Self { conn, path })
    }

    /// Starts a new transaction
    ///
    /// # Examples
    /// ```no_run
    /// use post_archiver::manager::PostArchiverManager;
    ///
    /// let mut manager = PostArchiverManager::open_in_memory().unwrap();
    /// let mut tx = manager.transaction().unwrap();
    /// // ... perform operations
    /// tx.commit().unwrap();
    /// ```
    pub fn transaction(&mut self) -> Result<PostArchiverManager<Transaction<'_>>, rusqlite::Error> {
        Ok(PostArchiverManager {
            path: self.path.clone(),
            conn: self.conn.transaction()?,
        })
    }
}

impl PostArchiverManager<Transaction<'_>> {
    /// Commits the transaction
    pub fn commit(self) -> Result<(), rusqlite::Error> {
        self.conn.commit()
    }
}

impl<C> PostArchiverManager<C>
where
    C: PostArchiverConnection,
{
    /// Returns a reference to the underlying database connection
    pub fn conn(&self) -> &Connection {
        self.conn.connection()
    }

    /// Bind an entity ID to get a [`Binded`] context for update/delete/relation operations.
    ///
    /// The type parameter is inferred from the ID argument — no turbofish needed.
    ///
    /// # Examples
    /// ```no_run
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::PostId;
    /// # fn example(manager: &PostArchiverManager, post_id: PostId) {
    /// let binded = manager.bind(post_id);
    /// // binded is Binded<'_, PostId>
    /// # }
    /// ```
    pub fn bind<Id: BindableId>(&self, id: Id) -> Binded<'_, Id, C> {
        Binded::new(self, id)
    }
}

/// Trait for types that can provide a database connection
pub trait PostArchiverConnection {
    fn connection(&self) -> &Connection;
}

impl PostArchiverConnection for Connection {
    fn connection(&self) -> &Connection {
        self
    }
}

impl PostArchiverConnection for Transaction<'_> {
    fn connection(&self) -> &Connection {
        self
    }
}
