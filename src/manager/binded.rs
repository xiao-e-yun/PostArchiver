use std::ops::Deref;

use rusqlite::Connection;

use crate::{AuthorId, CollectionId, FileMetaId, PlatformId, PostId, TagId};

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
/// Created via [`PostArchiverBindExt::bind`], available on the manager and on any
/// `Deref<Target = Connection>` such as a transaction.
/// The `Id` type parameter is inferred from the argument — no turbofish needed.
///
/// # Examples
/// ```no_run
/// # use post_archiver::manager::{PostArchiverBindExt, PostArchiverManager};
/// # use post_archiver::PostId;
/// # fn example(manager: &PostArchiverManager, post_id: PostId) -> Result<(), post_archiver::error::Error> {
/// // Binded<'_, PostId> — type inferred from post_id
/// manager.bind(post_id).delete()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Binded<'a, Id: BindableId> {
    pub(crate) conn: &'a Connection,
    id: Id,
}

impl<'a, Id: BindableId> Binded<'a, Id> {
    pub fn new(conn: &'a Connection, id: Id) -> Self {
        Self { conn, id }
    }

    /// Returns the bound entity ID.
    pub fn id(&self) -> Id {
        self.id
    }
}

pub trait PostArchiverBindExt {
    fn conn(&self) -> &Connection;

    /// Bind an entity ID to get a [`Binded`] context for update/delete/relation operations.
    ///
    /// The type parameter is inferred from the ID argument — no turbofish needed.
    ///
    /// # Examples
    /// ```no_run
    /// # use post_archiver::manager::{PostArchiverBindExt, PostArchiverManager};
    /// # use post_archiver::PostId;
    /// # fn example(manager: &PostArchiverManager, post_id: PostId) {
    /// let binded = manager.bind(post_id);
    /// // binded is Binded<'_, PostId>
    /// # }
    /// ```
    fn bind<Id: BindableId>(&self, id: Id) -> Binded<'_, Id> {
        Binded::new(self.conn(), id)
    }
}
impl<T> PostArchiverBindExt for T
where
    T: Deref<Target = Connection>,
{
    fn conn(&self) -> &Connection {
        self.deref()
    }
}
