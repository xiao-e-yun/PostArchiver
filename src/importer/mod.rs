pub mod author;
pub mod file_meta;
pub mod post;
pub mod tags;

use std::{
    cell::RefCell,
    collections::HashMap,
    panic,
    path::{Path, PathBuf},
    rc::Rc,
};

use rusqlite::{Connection,  Transaction};

use crate::{
    utils::{DATABASE_NAME, DATABASE_VERSION},
    PostTagId,
};

pub struct PostArchiverImporter<T> {
    pub conn: T,
    pub path: PathBuf,
    tags_cache: Rc<RefCell<HashMap<String, PostTagId>>>,
}

impl PostArchiverImporter<Connection> {
    pub fn create<P>(path: P) -> Result<Self, rusqlite::Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();
        let db_path = path.join(DATABASE_NAME);

        if db_path.exists() {
            panic!("Database already exists");
        }

        let conn = Connection::open(
            &db_path,
        )?;

        // run the template sql
        conn.execute_batch(include_str!("../utils/template.sql"))?;

        // push current version
        conn.execute(
            "INSERT INTO post_archiver_meta (version) VALUES (?)",
            &[DATABASE_VERSION],
        )?;

        let tags_cache = Rc::new(RefCell::new(HashMap::new()));
        Ok(Self {
            conn,
            path,
            tags_cache,
        })
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
        let expect_version = get_compatible_version(DATABASE_VERSION);

        if match_version != expect_version {
            panic!(
                "Database version mismatch \n + current: {}\n + expected: {}",
                version, DATABASE_VERSION
            );
        }

        let tags_cache = Rc::new(RefCell::new(HashMap::new()));
        Ok(Some(Self {
            conn,
            path,
            tags_cache,
        }))
    }
    pub fn open_or_create<P>(path: P) -> Result<Self, rusqlite::Error>
    where
        P: AsRef<Path>,
    {
        Self::open(&path)
            .transpose()
            .unwrap_or_else(|| Self::create(&path))
    }

    pub fn transaction(&mut self) -> Result<PostArchiverImporter<Transaction>, rusqlite::Error> {
        Ok(PostArchiverImporter {
            path: self.path.clone(),
            conn: self.conn.transaction()?,
            tags_cache: self.tags_cache.clone(),
        })
    }
}

impl<'a> PostArchiverImporter<Transaction<'a>> {
    pub fn commit(self) -> Result<(), rusqlite::Error> {
        self.conn.commit()
    }
}

impl<T> PostArchiverImporter<T>
where
    T: ImportConnection,
{
    fn conn(&self) -> &Connection {
        self.conn.connection()
    }
}

pub trait ImportConnection {
    fn connection(&self) -> &Connection;
}

impl ImportConnection for Connection {
    fn connection(&self) -> &Connection {
        self
    }
}

impl<'a> ImportConnection for Transaction<'a> {
    fn connection(&self) -> &Connection {
        self
    }
}
