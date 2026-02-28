use crate::{AuthorId, CollectionId, FileMetaId, PlatformId, PostId, TagId};

use super::{PostArchiverConnection, PostArchiverManager};

/// Marker trait for ID types that can be bound to a [`Binded`] context.
///
/// Implemented for all strongly-typed ID types in the system.
pub trait BindableId: Copy {}

impl BindableId for PostId {}
impl BindableId for AuthorId {}
impl BindableId for TagId {}
impl BindableId for PlatformId {}
impl BindableId for CollectionId {}
impl BindableId for FileMetaId {}

/// A bound entity context for update, delete, and relation operations.
///
/// Created via [`PostArchiverManager::bind(id)`](PostArchiverManager::bind).
/// The `Id` type parameter is inferred from the argument — no turbofish needed.
///
/// # Examples
/// ```no_run
/// # use post_archiver::manager::PostArchiverManager;
/// # use post_archiver::PostId;
/// # fn example(manager: &PostArchiverManager, post_id: PostId) -> Result<(), rusqlite::Error> {
/// // Binded<'_, PostId, _> — type inferred from post_id
/// manager.bind(post_id).delete()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Binded<'a, Id: BindableId, C: PostArchiverConnection = rusqlite::Connection> {
    manager: &'a PostArchiverManager<C>,
    id: Id,
}

impl<'a, Id: BindableId, C: PostArchiverConnection> Binded<'a, Id, C> {
    pub(crate) fn new(manager: &'a PostArchiverManager<C>, id: Id) -> Self {
        Self { manager, id }
    }

    /// Returns the bound entity ID.
    pub fn id(&self) -> Id {
        self.id
    }

    /// Returns a reference to the underlying manager.
    pub fn manager(&self) -> &'a PostArchiverManager<C> {
        self.manager
    }

    /// Shortcut: returns a reference to the database connection.
    pub(crate) fn conn(&self) -> &rusqlite::Connection {
        self.manager.conn()
    }
}
