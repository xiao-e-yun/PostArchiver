use rusqlite::OptionalExtension;

use crate::{utils::macros::as_table, Platform, PlatformId};

use super::{PostArchiverConnection, PostArchiverManager};

as_table!(Platform {
    id: "id",
    name: "name",
});

#[derive(Debug, Clone)]
pub enum PlatformIdOrRaw {
    Id(PlatformId),
    Raw(String),
}

pub trait PlatformLike: Sized {
    fn id(&self) -> Option<PlatformId> {
        None
    }
    fn raw(&self) -> Option<&str> {
        None
    }
    fn downcast(self) -> PlatformIdOrRaw {
        match self.id() {
            Some(id) => PlatformIdOrRaw::Id(id),
            None => PlatformIdOrRaw::Raw(self.raw().unwrap().to_string()),
        }
    }
}

impl PlatformLike for PlatformIdOrRaw {
    fn id(&self) -> Option<PlatformId> {
        match self {
            PlatformIdOrRaw::Id(id) => Some(*id),
            PlatformIdOrRaw::Raw(_) => None,
        }
    }

    fn raw(&self) -> Option<&str> {
        match self {
            PlatformIdOrRaw::Id(_) => None,
            PlatformIdOrRaw::Raw(name) => Some(name),
        }
    }
}

impl PlatformLike for Platform {
    fn id(&self) -> Option<PlatformId> {
        Some(self.id)
    }

    fn raw(&self) -> Option<&str> {
        Some(&self.name)
    }
}

impl PlatformLike for PlatformId {
    fn id(&self) -> Option<PlatformId> {
        Some(*self)
    }
}

impl PlatformLike for &str {
    fn raw(&self) -> Option<&str> {
        Some(self)
    }
}

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Retrieve all platforms.
    pub fn list_platforms(&self) -> Result<Vec<Platform>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM platforms")?;
        let platforms = stmt.query_map([], |row| Platform::from_row(row))?;
        platforms.collect()
    }

    /// Get a platform by its id or name.
    pub fn get_platform(
        &self,
        source: &impl PlatformLike,
    ) -> Result<Option<Platform>, rusqlite::Error> {
        match source.id() {
            Some(id) => {
                let query = "SELECT * FROM platforms WHERE id = ?";
                let mut stmt = self.conn().prepare_cached(query)?;
                stmt.query_row([id], |row| Platform::from_row(row))
            }
            None => {
                let name = source.raw().unwrap();
                let query = "SELECT * FROM platforms WHERE name = ?";
                let mut stmt = self.conn().prepare_cached(query)?;
                stmt.query_row([name], |row| Platform::from_row(row))
            }
        }
        .optional()
    }
}
