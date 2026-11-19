//! Remove files and directories that the database doesn't reference, then `VACUUM` it.
//!
//! Only the post directories (`<chunk>/<index>/`, see `Post::directory`) are touched:
//! - files in a post directory without a matching `file_metas` row are removed,
//! - post and chunk directories left empty are removed,
//! - anything unexpected (nested directories, symlinks, non-numeric names) is reported and kept,
//! - `file_metas` rows whose file is missing are reported and kept, since posts reference them.

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use log::{info, warn};
use post_archiver_latest::Post;
use rusqlite::{Connection, OpenFlags};

use crate::{detect_version, ArchiveVersion, CommandResult, DATABASE_NAME, LATEST_VERSION};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// Files not referenced by any `file_metas` row.
    pub removed_files: Vec<PathBuf>,
    /// Post/chunk directories that are empty (after removing files).
    pub removed_dirs: Vec<PathBuf>,
    /// Entries that don't fit the archive layout; never removed.
    pub skipped: Vec<PathBuf>,
    /// Files referenced by `file_metas` that don't exist.
    pub missing: Vec<PathBuf>,
}

pub fn vacuum_command(path: &Path, dry_run: bool) -> CommandResult {
    let version = detect_version(path)?;
    if !version.is_latest() {
        return Err(format!(
            "Archive is on {}, run `post-archiver migrate` first (vacuum supports {LATEST_VERSION})",
            version.label()
        )
        .into());
    }
    debug_assert!(matches!(version, ArchiveVersion::Versioned(_)));

    let db_path = path.join(DATABASE_NAME);
    let size_before = fs::metadata(&db_path)?.len();

    let report = vacuum(path, dry_run)?;

    let verb = if dry_run { "Would remove" } else { "Removed" };
    for file in &report.removed_files {
        info!("{verb} file: {}", file.display());
    }
    for dir in &report.removed_dirs {
        info!("{verb} empty directory: {}", dir.display());
    }
    for entry in &report.skipped {
        warn!("Skipped unexpected entry: {}", entry.display());
    }
    for file in &report.missing {
        warn!(
            "Missing file referenced by the database: {}",
            file.display()
        );
    }

    let size_after = fs::metadata(&db_path)?.len();
    info!("Summary{}:", if dry_run { " (dry run)" } else { "" });
    info!("  {verb} {} unreferenced files", report.removed_files.len());
    info!("  {verb} {} empty directories", report.removed_dirs.len());
    info!("  Skipped {} unexpected entries", report.skipped.len());
    info!("  Missing {} referenced files", report.missing.len());
    if dry_run {
        info!("  Database not vacuumed ({size_before} bytes)");
    } else {
        info!("  Database vacuumed: {size_before} -> {size_after} bytes");
    }
    Ok(())
}

pub fn vacuum(root: &Path, dry_run: bool) -> Result<Report, Box<dyn std::error::Error>> {
    let db_path = root.join(DATABASE_NAME);
    let flags = if dry_run {
        OpenFlags::SQLITE_OPEN_READ_ONLY
    } else {
        OpenFlags::SQLITE_OPEN_READ_WRITE
    };
    let conn = Connection::open_with_flags(&db_path, flags | OpenFlags::SQLITE_OPEN_NO_MUTEX)?;

    let referenced: HashSet<(u32, String)> = conn
        .prepare("SELECT post, filename FROM file_metas")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<_, _>>()?;

    let mut report = Report::default();

    for chunk_entry in sorted_entries(root)? {
        let chunk_path = chunk_entry.path();
        let Some(chunk) = numeric_name(&chunk_path) else {
            continue; // configs, database files, backups, ...
        };
        if !chunk_entry.file_type()?.is_dir() {
            report.skipped.push(chunk_path);
            continue;
        }

        let mut chunk_kept = 0;
        for post_entry in sorted_entries(&chunk_path)? {
            let post_path = post_entry.path();
            let index = numeric_name(&post_path).filter(|i| *i < Post::POSTS_PER_CHUNK);
            let (Some(index), true) = (index, post_entry.file_type()?.is_dir()) else {
                report.skipped.push(post_path);
                chunk_kept += 1;
                continue;
            };
            let post = chunk * Post::POSTS_PER_CHUNK + index;

            let mut post_kept = 0;
            for file_entry in sorted_entries(&post_path)? {
                let file_path = file_entry.path();
                let is_file = file_entry.file_type()?.is_file();
                let name = file_entry.file_name().to_string_lossy().into_owned();

                if !is_file {
                    report.skipped.push(file_path);
                    post_kept += 1;
                } else if referenced.contains(&(post, name)) {
                    post_kept += 1;
                } else {
                    if !dry_run {
                        fs::remove_file(&file_path)?;
                    }
                    report.removed_files.push(file_path);
                }
            }

            if post_kept == 0 {
                if !dry_run {
                    fs::remove_dir(&post_path)?;
                }
                report.removed_dirs.push(post_path);
            } else {
                chunk_kept += 1;
            }
        }

        if chunk_kept == 0 {
            if !dry_run {
                fs::remove_dir(&chunk_path)?;
            }
            report.removed_dirs.push(chunk_path);
        }
    }

    let mut missing: Vec<PathBuf> = referenced
        .iter()
        .map(|(post, filename)| {
            root.join((post / Post::POSTS_PER_CHUNK).to_string())
                .join((post % Post::POSTS_PER_CHUNK).to_string())
                .join(filename)
        })
        .filter(|path| !path.is_file())
        .collect();
    missing.sort();
    report.missing = missing;

    if !dry_run {
        conn.execute_batch("VACUUM")?;
    }

    Ok(report)
}

fn sorted_entries(dir: &Path) -> std::io::Result<Vec<fs::DirEntry>> {
    let mut entries = fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    Ok(entries)
}

/// Directory names produced by `Post::directory`: plain non-negative integers.
fn numeric_name(path: &Path) -> Option<u32> {
    let name = path.file_name()?.to_str()?;
    if name.is_empty() || !name.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    name.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal archive: a `file_metas` table plus files on disk.
    fn archive(metas: &[(u32, &str)], files: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let conn = Connection::open(dir.path().join(DATABASE_NAME)).unwrap();
        conn.execute_batch(
            "CREATE TABLE post_archiver_meta (version TEXT NOT NULL);
             INSERT INTO post_archiver_meta VALUES ('0.6.0');
             CREATE TABLE file_metas (id INTEGER PRIMARY KEY, post INTEGER NOT NULL, filename TEXT NOT NULL);",
        )
        .unwrap();
        for (post, filename) in metas {
            conn.execute(
                "INSERT INTO file_metas (post, filename) VALUES (?, ?)",
                rusqlite::params![post, filename],
            )
            .unwrap();
        }
        for file in files {
            let path = dir.path().join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"x").unwrap();
        }
        dir
    }

    fn rel(root: &Path, paths: &[PathBuf]) -> Vec<String> {
        paths
            .iter()
            .map(|p| p.strip_prefix(root).unwrap().to_string_lossy().into_owned())
            .collect()
    }

    fn fixture() -> tempfile::TempDir {
        archive(
            // post 1 -> 0/1, post 2049 -> 1/1, post 3 -> 0/3 (file missing)
            &[(1, "a.png"), (2049, "b.png"), (3, "gone.png")],
            &[
                "0/1/a.png",
                "0/1/stray.png",  // unreferenced, its post dir stays (a.png)
                "0/2/orphan.png", // unreferenced, post dir becomes empty
                "1/1/b.png",
                "5/7/lonely.png", // unreferenced, post and chunk dirs become empty
                "configs/keep.json", // not part of the post layout
                "0/notes/x.txt",  // unexpected entry inside a chunk
            ],
        )
    }

    #[test]
    fn dry_run_reports_without_touching_anything() {
        let dir = fixture();
        let root = dir.path();
        let report = vacuum(root, true).unwrap();

        assert_eq!(
            rel(root, &report.removed_files),
            ["0/1/stray.png", "0/2/orphan.png", "5/7/lonely.png"]
        );
        assert_eq!(rel(root, &report.removed_dirs), ["0/2", "5/7", "5"]);
        assert_eq!(rel(root, &report.skipped), ["0/notes"]);
        assert_eq!(rel(root, &report.missing), ["0/3/gone.png"]);

        for file in ["0/1/stray.png", "0/2/orphan.png", "5/7/lonely.png"] {
            assert!(root.join(file).exists(), "dry run removed {file}");
        }
    }

    #[test]
    fn removes_unreferenced_files_and_empty_dirs_only() {
        let dir = fixture();
        let root = dir.path();
        let dry = vacuum(root, true).unwrap();
        let report = vacuum(root, false).unwrap();
        assert_eq!(report, dry, "dry run and real run must report the same");

        for gone in ["0/1/stray.png", "0/2", "5"] {
            assert!(!root.join(gone).exists(), "{gone} should be removed");
        }
        for kept in [
            "0/1/a.png",
            "1/1/b.png",
            "configs/keep.json",
            "0/notes/x.txt",
            DATABASE_NAME,
        ] {
            assert!(root.join(kept).exists(), "{kept} should be kept");
        }

        // Nothing left to clean up.
        let again = vacuum(root, false).unwrap();
        assert!(again.removed_files.is_empty() && again.removed_dirs.is_empty());
    }
}
