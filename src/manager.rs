use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use rusqlite::{Connection, Transaction};

use crate::utils::{DATABASE_NAME, VERSION};
use crate::PostTagId;

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
        conn.execute_batch(include_str!("utils/template.sql"))?;

        // push current version
        conn.execute(
            "INSERT INTO post_archiver_meta (version) VALUES (?)",
            &[VERSION],
        )?;

        let cache = Arc::new(PostArchiverManagerCache::new());

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
        let path = path.as_ref().to_path_buf();
        let db_path = path.join(DATABASE_NAME);

        if !db_path.exists() {
            return Ok(None);
        }

        let conn = Connection::open(db_path)?;

        // check version
        let version: String = conn
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
            );
        }

        let cache = Arc::new(PostArchiverManagerCache::new());
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
        conn.execute_batch(include_str!("utils/template.sql"))?;

        // push current version
        conn.execute(
            "INSERT INTO post_archiver_meta (version) VALUES (?)",
            &[VERSION],
        )?;

        let cache = Arc::new(PostArchiverManagerCache::new());

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

impl<'a> PostArchiverManager<Transaction<'a>> {
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
}

#[derive(Debug)]
pub struct PostArchiverManagerCache {
    pub tags: Mutex<HashMap<String, PostTagId>>,
}

impl PostArchiverManagerCache {
    pub fn new() -> Self {
        Self {
            tags: Mutex::new(HashMap::new()),
        }
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

impl<'a> PostArchiverConnection for Transaction<'a> {
    fn connection(&self) -> &Connection {
        self
    }
}
