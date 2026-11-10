//! Importing scraped data into an archive.
//!
//! Build `Unsync*` values (posts, authors, tags, collections, files) and hand them to an
//! [`Importer`] from [`PostArchiverManager::importer`]. Entities are matched against what is
//! already in the archive, so importing the same post again updates it in place:
//!
//! | Entity | Matched by |
//! |--------|------------|
//! | post | `source` |
//! | author | any alias `(platform, source)` |
//! | collection | `source` |
//! | tag | `(name, platform)`; `platform: None` is a general tag |
//! | platform | name (case-insensitive) |
//! | file | `(post, filename)` |
//!
//! # Transactions
//!
//! Every `import_*` call is atomic: if it fails, nothing it did is kept.
//! To make several imports all-or-nothing, open a savepoint yourself and
//! [`commit`](Importer::commit) it; dropping it without committing rolls everything back.
//! Savepoints can be nested.
//!
//! # Files
//!
//! - Files are **copied** from [`UnsyncFileMeta::source`] into the post's directory. The source
//!   file is left alone; deleting it after a successful import is up to the caller.
//! - Copies are written as `<filename>.importing` and renamed into place when the outermost
//!   savepoint commits. On rollback they are deleted, and files already in the archive are
//!   untouched.
//! - [`UnsyncFileMeta::filename`] must be a plain file name (no `/`, `\` or `..`); it becomes
//!   the file's name inside the post directory.
//! - Re-importing a file with the same filename replaces its content, MIME type and `extra`.
//!
//! # Example
//!
//! ```
//! use post_archiver::importer::{UnsyncContent, UnsyncFileMeta, UnsyncPost, UnsyncTag};
//! use post_archiver::manager::PostArchiverManager;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let dir = tempdir::TempDir::new("doc")?;
//! # let root = dir.path().join("archive");
//! # let src = dir.path().join("downloaded.png");
//! # std::fs::create_dir(&root)?;
//! # std::fs::write(&src, b"png")?;
//! let mut manager = PostArchiverManager::create(&root)?;
//!
//! let image = UnsyncFileMeta::new("1.png".into(), "image/png".into(), src.clone());
//! let post = UnsyncPost::new(
//!     "pixiv".into(),
//!     "https://example.com/posts/1".into(),
//!     "Hello".into(),
//!     vec![UnsyncContent::Text("hi".into()), UnsyncContent::File(image)],
//! )
//! .tags(vec![UnsyncTag { name: "art".into(), platform: None }]);
//!
//! let mut importer = manager.importer();
//! let mut tx = importer.savepoint()?; // optional: group several imports
//! let id = tx.import_post(post)?;
//! tx.commit()?;
//!
//! std::fs::remove_file(&src)?; // sources are kept; clean up after a successful import
//! assert_eq!(manager.importer().lookup_post("https://example.com/posts/1")?, Some(id));
//! # Ok(())
//! # }
//! ```

pub mod author;

use std::{
    collections::HashSet,
    fs, mem,
    ops::Deref,
    path::{Path, PathBuf},
};

pub use author::*;

pub mod file_meta;
pub use file_meta::*;

pub mod post;
pub use post::*;

pub mod collection;
pub use collection::*;

pub mod tag;
use rusqlite::{Connection, Result, Savepoint};
pub use tag::*;

use crate::manager::PostArchiverManager;

pub mod platform;

impl PostArchiverManager {
    /// Create an importer for importing data into the archive.
    pub fn importer(&mut self) -> Importer<'_, 'static, &mut Connection> {
        Importer {
            conn: &mut self.conn,
            path: self.path.as_path(),
            pending: ImporterPending {
                buffer: HashSet::new(),
                parent: None,
                depth: 0,
            },
        }
    }
}

#[derive(Debug)]
pub struct Importer<'root, 'parent, C> {
    pub path: &'root Path,
    pub conn: C,
    pub pending: ImporterPending<'parent>,
}

impl<C: SavepointExt> Importer<'_, '_, C> {
    pub fn savepoint(&'_ mut self) -> Result<Importer<'_, '_, Savepoint<'_>>> {
        Ok(Importer {
            conn: self.conn.savepoint()?,
            path: self.path,
            pending: ImporterPending {
                buffer: HashSet::new(),
                parent: Some(&mut self.pending.buffer),
                depth: self.pending.depth + 1,
            },
        })
    }
}

pub trait SavepointExt: Deref<Target = Connection> {
    fn savepoint(&mut self) -> Result<Savepoint<'_>>;
}
impl SavepointExt for &mut Connection {
    fn savepoint(&mut self) -> Result<Savepoint<'_>> {
        Savepoint::new(self)
    }
}
impl SavepointExt for Savepoint<'_> {
    fn savepoint(&mut self) -> Result<Savepoint<'_>> {
        self.savepoint()
    }
}

impl Importer<'_, '_, Savepoint<'_>> {
    pub fn commit(self) -> crate::error::Result<()> {
        self.conn.commit()?;
        self.pending.commit()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ImporterPending<'parent> {
    pub buffer: HashSet<PathBuf>,
    pub parent: Option<&'parent mut HashSet<PathBuf>>,
    pub depth: usize,
}

impl ImporterPending<'_> {
    const IMPORTING_EXT: &'static str = "importing";
    pub fn stage(&mut self, src: &Path, dst: PathBuf) -> crate::error::Result<()> {
        let temp = dst.with_added_extension(Self::IMPORTING_EXT);
        fs::copy(src, &temp)?;
        self.buffer.insert(dst);
        Ok(())
    }
    pub fn commit(mut self) -> crate::error::Result<()> {
        if self.depth <= 1 {
            for dst in &self.buffer {
                let src = dst.with_added_extension(Self::IMPORTING_EXT);
                fs::rename(&src, dst)?;
            }
            self.buffer.clear();
        } else {
            let parent = self.parent.as_mut().unwrap();
            parent.extend(mem::take(&mut self.buffer));
        }
        Ok(())
    }
}

impl Drop for ImporterPending<'_> {
    fn drop(&mut self) {
        if self.depth > 1 {
            return;
        }
        for dst in &self.buffer {
            let src = dst.with_added_extension(Self::IMPORTING_EXT);
            let _ = fs::remove_file(&src);
        }
    }
}
