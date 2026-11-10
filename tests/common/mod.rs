//! Shared fixture for integration tests.
//!
//! Every test gets its own temporary directory holding an archive (`archive/`) and a
//! download area (`dl/`) for source files, so tests can run in parallel without touching
//! each other's files.
#![allow(dead_code)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use post_archiver::{
    importer::{UnsyncContent, UnsyncFileMeta, UnsyncPost},
    manager::PostArchiverManager,
    Post, PostId,
};
use tempdir::TempDir;

pub struct Archive {
    dir: TempDir,
    pub m: PostArchiverManager,
}

impl Archive {
    pub fn new() -> Self {
        let dir = TempDir::new("post-archiver-test").unwrap();
        fs::create_dir_all(dir.path().join("archive")).unwrap();
        fs::create_dir_all(dir.path().join("dl")).unwrap();
        let m = PostArchiverManager::create(dir.path().join("archive")).unwrap();
        Self { dir, m }
    }

    pub fn root(&self) -> PathBuf {
        self.dir.path().join("archive")
    }

    /// Path of a source file in the download area.
    pub fn src(&self, name: &str) -> PathBuf {
        self.dir.path().join("dl").join(name)
    }

    /// Write a source file and describe it as an `UnsyncFileMeta` named `filename`.
    pub fn file(&self, src: &str, filename: &str, data: &[u8]) -> UnsyncFileMeta {
        let path = self.src(src);
        fs::write(&path, data).unwrap();
        UnsyncFileMeta::new(filename.into(), mime(filename).into(), path)
    }

    /// Read a file stored in the archive for `post`.
    pub fn read(&self, post: PostId, filename: &str) -> Option<Vec<u8>> {
        fs::read(self.root().join(Post::directory(post)).join(filename)).ok()
    }

    /// Staging files (`*.importing`) left anywhere in the archive.
    pub fn leftovers(&self) -> Vec<PathBuf> {
        fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
            for entry in fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    walk(&path, out);
                } else if path.extension().is_some_and(|e| e == "importing") {
                    out.push(path);
                }
            }
        }
        let mut out = vec![];
        walk(&self.root(), &mut out);
        out
    }

    pub fn import(&mut self, post: UnsyncPost) -> PostId {
        self.m.importer().import_post(post).unwrap()
    }
}

/// A minimal post whose title equals its source.
pub fn post(source: &str) -> UnsyncPost {
    UnsyncPost::new("test".into(), source.into(), source.into(), vec![])
}

pub fn file_content(file: &UnsyncFileMeta) -> UnsyncContent {
    UnsyncContent::File(file.clone())
}

pub fn ts(secs: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(secs, 0).unwrap()
}

fn mime(filename: &str) -> &'static str {
    match filename.rsplit('.').next() {
        Some("png") => "image/png",
        Some("jpg") => "image/jpeg",
        Some("mp4") => "video/mp4",
        _ => "application/octet-stream",
    }
}
