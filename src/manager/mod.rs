use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use dashmap::DashMap;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde_json::Value;

use crate::TagId;
use crate::{
    utils::{DATABASE_NAME, VERSION},
    CollectionId, PlatformId, PlatformTagId,
};

pub mod author;
pub mod post;
pub mod file_meta;
pub mod tag;
pub mod platform;
pub mod collection;

/// Core manager type for post archive operations with SQLite backend
///
/// # Examples
/// ```no_run
/// use post_archiver::manager::PostArchiverManager;
///    
/// let manager = PostArchiverManager::open_or_create("./data").unwrap();
/// ```
#[derive(Debug)]
pub struct PostArchiverManager<T = Connection> {
    pub path: PathBuf,
    conn: T,
    pub(crate) cache: Arc<PostArchiverManagerCache>,
}

impl PostArchiverManager {
    /// Creates a new archive at the specified path
    ///
    /// # Safety
    /// The path must not already contain a database file.
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
            &[VERSION],
        )?;

        let cache = Arc::new(PostArchiverManagerCache::default());

        Ok(Self { conn, path, cache })
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

    /// Opens an existing archive at the specified path
    /// Does not check the version of the archive.
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
    /// if let Some(manager) = PostArchiverManager::open_uncheck("./archive").unwrap() {
    ///     println!("Archive opened successfully");
    /// }
    /// ```
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

        let cache = Arc::new(PostArchiverManagerCache::default());
        Ok(Some(Self { conn, path, cache }))
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
    /// it will generate a temporary path for the archive files
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

        let cache = Arc::new(PostArchiverManagerCache::default());

        Ok(Self { conn, path, cache })
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
    pub fn transaction(&mut self) -> Result<PostArchiverManager<Transaction>, rusqlite::Error> {
        Ok(PostArchiverManager {
            path: self.path.clone(),
            conn: self.conn.transaction()?,
            cache: self.cache.clone(),
        })
    }
}

impl PostArchiverManager<Transaction<'_>> {
    /// Commits the transaction
    pub fn commit(self) -> Result<(), rusqlite::Error> {
        self.conn.commit()
    }
}

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    pub fn conn(&self) -> &Connection {
        self.conn.connection()
    }
    pub fn get_feature(&self, name: &str) -> Result<i64, rusqlite::Error> {
        self.conn()
            .query_row("SELECT value FROM features WHERE name = ?", [name], |row| row.get(0))
            .optional()
            .transpose()
            .unwrap_or(Ok(0))
    }
    pub fn get_feature_with_extra(&self, name: &str) -> Result<(i64, HashMap<String, Value>), rusqlite::Error> {
        self.conn()
            .query_row(
                "SELECT value, extra FROM features WHERE name = ?",
                [name],
                |row| {
                    let value: i64 = row.get(0)?;
                    let extra: String = row.get(1)?;
                    let extra: HashMap<String, Value> =
                        serde_json::from_str(&extra).unwrap_or_default();
                    Ok((value, extra))
                },
            )
            .optional()
            .transpose()
            .unwrap_or(Ok((0, HashMap::default())))
    }
    pub fn set_feature(&self, name: &str, value: i64) {
        self.conn()
            .execute(
                "INSERT INTO features (name, value) VALUES (?, ?) ON CONFLICT(name) DO UPDATE SET value = ?",
                params![name, value, value],
            )
            .unwrap();
    }
    pub fn set_feature_with_extra(&self, name: &str, value: i64, extra: HashMap<String, Value>) {
        let extra = serde_json::to_string(&extra).unwrap();
        self.conn()
            .execute(
                "INSERT OR REPLACE INTO features (name, value, extra) VALUES (?, ?, ?)",
                params![name, value, extra],
            )
            .unwrap();
    }
}

#[derive(Debug, Default)]
pub struct PostArchiverManagerCache {
    pub tags: DashMap<String, TagId>,
    pub platform_tags: DashMap<(PlatformId, String), PlatformTagId>,
    pub collections: DashMap<String, CollectionId>,
    pub platforms: DashMap<String, PlatformId>,
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
