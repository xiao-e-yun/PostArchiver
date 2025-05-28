pub mod config;
mod v1;
mod v2;
mod v3;

use std::{fs, io, path::Path};

use clap::Parser;
use config::Config;
use log::{error, info, warn};
use rusqlite::Connection;

fn main() {
    simple_logger::init().unwrap();
    info!("Starting up");

    let mut config = Config::parse();

    info!("");
    info!("- Configuration -----------------");
    info!("Source: {:?}", config.source);
    info!("Output: {:?}", config.target);
    info!("---------------------------------");
    info!("");

    if !config.source.is_dir() {
        error!("Source does not exist");
        return;
    }

    if config.source == config.target {
        config.overwrite = true;
        warn!("Source and Target are the same");
        warn!("It will overwrite the source");
        info!("");
        loop {
            let mut input = String::new();
            println!("Are you sure you want to continue? (yes/No): ");
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();

            if ["yes", "y"].contains(&input.as_str()) {
                info!("Continuing with overwrite");
                break;
            } else if ["no", "n"].contains(&input.as_str()) || input.is_empty() {
                info!("Exiting without changes");
                return;
            } else {
                warn!("Invalid input, please enter 'yes' or 'no'");
            }
        }
    }

    if !config.overwrite {
        if config.target.exists() {
            error!("Target is already exists");
        } else {
            copy_dir_all(&config.source, &config.target).unwrap();

            // https://stackoverflow.com/a/65192210/15859431
            fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
                fs::create_dir_all(&dst)?;
                for entry in fs::read_dir(src)? {
                    let entry = entry?;
                    let ty = entry.file_type()?;
                    if ty.is_dir() {
                        copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
                    } else {
                        fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
                    }
                }
                Ok(())
            }
        }
    }

    // Major Migrating
    v1::Bridge::verify_and_upgrade(&mut config);

    // Add version check, and others.
    v2::Bridge::verify_and_upgrade(&mut config);

    let mut conn = Connection::open(config.target.join("post-archiver.db")).unwrap();
    // Database Migration
    v3::Bridge::verify_and_upgrade(&mut conn, &mut config);

    if config.updated {
        info!("Successfully updated");
    } else {
        info!("No updates required");
    }
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

    fn verify(&mut self, conn: &Connection) -> bool {
        conn.query_row(
            "SELECT count() FROM post_archiver_meta WHERE version LIKE ? || %",
            [],
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
            conn.execute_batch(Self::SQL)
                .expect("Failed to execute migration SQL");
            config.updated = true;
            info!("=================================");
        }
        info!("");
    }
}
