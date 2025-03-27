use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use rusqlite::{Connection, Transaction};

use crate::PostTagId;

use super::{DATABASE_NAME, VERSION};

/// manager for the archive
///
/// Provides common functions to manage the archive
#[cfg(feature = "importer")]
#[derive(Debug)]
pub struct PostArchiverManager<T = Connection> {
    pub path: PathBuf,
    conn: T,
    pub(crate) cache: Arc<PostArchiverManagerCache>,
}

impl PostArchiverManager {
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

        let cache = Arc::new(PostArchiverManagerCache::new());

        Ok(Self { conn, path, cache })
    }
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
    pub fn open_or_create<P>(path: P) -> Result<Self, rusqlite::Error>
    where
        P: AsRef<Path>,
    {
        Self::open(&path)
            .transpose()
            .unwrap_or_else(|| Self::create(&path))
    }

    pub fn transaction(&mut self) -> Result<PostArchiverManager<Transaction>, rusqlite::Error> {
        Ok(PostArchiverManager {
            path: self.path.clone(),
            conn: self.conn.transaction()?,
            cache: self.cache.clone(),
        })
    }
}

impl<'a> PostArchiverManager<Transaction<'a>> {
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
