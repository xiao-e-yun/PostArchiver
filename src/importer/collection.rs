use std::{hash::Hash, ops::Deref};

use rusqlite::{params, Connection, OptionalExtension};

use crate::{
    error::Result,
    importer::Importer,
    manager::{PostArchiverBindExt, UpdateCollection},
    CollectionId,
};

impl<C: Deref<Target = Connection>> Importer<'_, '_, C> {
    pub fn import_collection(&mut self, collection: UnsyncCollection) -> Result<CollectionId> {
        let id = self.lookup_collection(&collection.source)?;
        Ok(match id {
            Some(id) => {
                let b = self.conn.bind(id);
                b.update(UpdateCollection::default().name(collection.name))?;
                id
            }
            None => {
                let mut ins_stmt = self.conn.prepare_cached(
                    "INSERT INTO collections (name, source) VALUES (?, ?) RETURNING id",
                )?;
                let id: CollectionId = ins_stmt
                    .query_row(params![collection.name, collection.source], |row| {
                        row.get(0)
                    })?;
                id
            }
        })
    }

    pub fn lookup_collection(&self, source: &str) -> Result<Option<CollectionId>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT id FROM collections WHERE source = ?")?;
        let query = stmt.query_row(params![source], |row| row.get(0));
        Ok(query.optional()?)
    }
}

/// Represents a collection that is not yet synced to the database.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnsyncCollection {
    pub name: String,
    pub source: String,
}

impl UnsyncCollection {
    pub fn new(name: String, source: String) -> Self {
        Self { name, source }
    }
    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }
    pub fn source(mut self, source: String) -> Self {
        self.source = source;
        self
    }
}
