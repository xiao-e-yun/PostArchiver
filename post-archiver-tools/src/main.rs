pub mod config;
mod v2;
mod v3;
mod v4;
mod v5;
mod v6;
mod vacuum;

use std::{
    io::{self, IsTerminal, Write},
    path::Path,
    process::exit,
};

use clap::Parser;
use config::{Cli, Command, Config};
use log::{error, info, warn};
use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction};

/// Version written by the newest migration. Bump it together with a new `vN` module.
pub const LATEST_VERSION: &str = "0.6.0";
pub const RELEASES_URL: &str = "https://github.com/xiao-e-yun/PostArchiver/releases";
pub const DATABASE_NAME: &str = "post-archiver.db";

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .without_timestamps()
        .init()
        .unwrap();

    let cli = Cli::parse();
    let result = match cli.command {
        Command::Info { path } => info_command(&path),
        Command::Migrate { path, yes } => migrate_command(&path, yes),
        Command::Vacuum { path, dry_run } => vacuum::vacuum_command(&path, dry_run),
    };

    if let Err(e) = result {
        error!("{e}");
        exit(1);
    }
}

pub type CommandResult = Result<(), Box<dyn std::error::Error>>;

/// Detected archive version, without modifying anything on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveVersion {
    /// Pre-database archive (`authors.json`), handled by the v0.1 migration.
    Legacy,
    /// A database without `post_archiver_meta` (v0.2).
    Unversioned,
    Versioned(String),
}

impl ArchiveVersion {
    pub fn label(&self) -> String {
        match self {
            ArchiveVersion::Legacy => "0.1.x".to_string(),
            ArchiveVersion::Unversioned => "0.2.x".to_string(),
            ArchiveVersion::Versioned(v) => v.clone(),
        }
    }

    pub fn is_latest(&self) -> bool {
        matches!(self, ArchiveVersion::Versioned(v) if same_minor(v, LATEST_VERSION))
    }
}

/// Compare `major.minor`, which is what post-archiver checks when opening an archive.
pub fn same_minor(a: &str, b: &str) -> bool {
    let minor = |v: &str| v.splitn(3, '.').take(2).collect::<Vec<_>>().join(".");
    minor(a) == minor(b)
}

pub fn open_read_only(path: &Path) -> rusqlite::Result<Connection> {
    Connection::open_with_flags(
        path.join(DATABASE_NAME),
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
}

pub fn detect_version(path: &Path) -> Result<ArchiveVersion, Box<dyn std::error::Error>> {
    if !path.is_dir() {
        return Err(format!("{} is not a directory", path.display()).into());
    }
    let db = path.join(DATABASE_NAME);
    if !db.exists() {
        if path.join("authors.json").exists() {
            return Ok(ArchiveVersion::Legacy);
        }
        return Err(format!(
            "{} is not a post-archiver archive (no {DATABASE_NAME})",
            path.display()
        )
        .into());
    }

    let conn = open_read_only(path)?;
    let has_meta: bool = conn.query_row(
        "SELECT count() FROM sqlite_master WHERE type = 'table' AND name = 'post_archiver_meta'",
        [],
        |row| row.get(0),
    )?;
    if !has_meta {
        return Ok(ArchiveVersion::Unversioned);
    }
    let version: Option<String> = conn
        .query_row("SELECT version FROM post_archiver_meta", [], |row| {
            row.get(0)
        })
        .optional()?;
    Ok(ArchiveVersion::Versioned(
        version.unwrap_or_else(|| "unknown".to_string()),
    ))
}

fn info_command(path: &Path) -> CommandResult {
    let version = detect_version(path)?;

    info!("Archive:  {}", path.display());
    info!("Version:  {}", version.label());
    info!("Latest:   {LATEST_VERSION}");

    if let ArchiveVersion::Versioned(_) = version {
        let conn = open_read_only(path)?;
        for table in [
            "posts",
            "authors",
            "tags",
            "collections",
            "platforms",
            "file_metas",
        ] {
            let count: rusqlite::Result<i64> =
                conn.query_row(&format!("SELECT count() FROM {table}"), [], |row| {
                    row.get(0)
                });
            if let Ok(count) = count {
                info!("{:<12} {count}", format!("{table}:"));
            }
        }
    }

    if !version.is_latest() {
        warn!("This archive is not on the latest version. Tools are pinned to a database version,");
        warn!("so only upgrade when the tools you use support it: {RELEASES_URL}");
    }
    Ok(())
}

fn migrate_command(path: &Path, yes: bool) -> CommandResult {
    let version = detect_version(path)?;
    if version.is_latest() {
        info!("Already up to date ({})", version.label());
        return Ok(());
    }

    info!("Archive:  {}", path.display());
    info!("Database: {} -> {LATEST_VERSION}", version.label());
    warn!("THIS CANNOT BE UNDONE!");

    if !yes {
        if !io::stdin().is_terminal() {
            return Err("Refusing to migrate without --yes in a non-interactive session".into());
        }
        if !confirm("Continue? [y/N] ")? {
            info!("Aborted, nothing was changed");
            return Ok(());
        }
    }

    let mut config = Config {
        target: path.to_path_buf(),
        updated: false,
    };

    // File-structure migrations
    v2::Bridge::verify_and_upgrade(&mut config);
    v3::Bridge::verify_and_upgrade(&mut config);

    // Database migrations
    let mut conn = Connection::open(path.join(DATABASE_NAME))?;
    v4::Bridge::verify_and_upgrade(&mut conn, &mut config);
    v5::Bridge::verify_and_upgrade(&mut conn, &mut config);
    v6::Bridge::verify_and_upgrade(&mut conn, &mut config);
    drop(conn);

    let after = detect_version(path)?;
    info!("Summary: {} -> {}", version.label(), after.label());
    if !after.is_latest() {
        return Err(format!(
            "Migration finished on {}, expected {LATEST_VERSION}",
            after.label()
        )
        .into());
    }
    Ok(())
}

fn confirm(prompt: &str) -> io::Result<bool> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}

pub trait Migration: Default {
    const VERSION: &'static str;
    fn verify(&mut self, path: &Path) -> bool;
    fn upgrade(&mut self, path: &Path);
    fn verify_and_upgrade(config: &mut Config) {
        let mut bridge = Self::default();

        info!("Checking updates v{}", Self::VERSION);
        if bridge.verify(&config.target) {
            info!("=================================");
            info!("Migrating from v{}", Self::VERSION);
            bridge.upgrade(&config.target);
            config.updated = true;
            info!("=================================");
        }
        info!("");
    }
}

pub trait MigrationDatabase: Default {
    const VERSION: &'static str;
    const SQL: &'static str;
    fn upgrade(&mut self, path: &Path, tx: &mut Transaction<'_>);

    fn verify(&mut self, conn: &Connection) -> bool {
        conn.query_row(
            "SELECT count() FROM post_archiver_meta WHERE version LIKE ? || '%'",
            [Self::VERSION],
            |row| Ok(row.get_unwrap::<_, usize>(0) == 1),
        )
        .unwrap()
    }
    fn verify_and_upgrade(conn: &mut Connection, config: &mut Config) {
        let mut bridge = Self::default();

        info!("Checking updates v{}", Self::VERSION);
        if bridge.verify(conn) {
            info!("=================================");
            info!("Migrating from v{}", Self::VERSION);
            conn.pragma_update(None, "foreign_keys", "off")
                .expect("Failed to disable foreign keys");
            let mut tx = conn.transaction().unwrap();
            tx.execute_batch(Self::SQL)
                .expect("Failed to execute migration SQL");
            bridge.upgrade(&config.target, &mut tx);
            tx.commit().expect("Failed to commit transaction");
            conn.pragma_update(None, "foreign_keys", "on")
                .expect("Failed to enable foreign keys");
            config.updated = true;
            info!("=================================");
        }
        info!("");
    }
}
