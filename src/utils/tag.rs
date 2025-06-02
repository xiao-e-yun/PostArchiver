use crate::{
    manager::platform::PlatformIdOrRaw, PlatformId, PlatformTag, PlatformTagId, Tag, TagId,
};

use super::macros::as_table;

as_table! {
    Tag {
        id: "id",
        name: "name",
    }
}

as_table!(PlatformTag {
    id: "id",
    name: "name",
    platform: "platform",
});

pub trait TagLike: Sized {
    fn id(&self) -> Option<TagId> {
        None
    }
    fn raw(&self) -> Option<&str> {
        None
    }
    fn downcast(self) -> TagIdOrRaw {
        match self.id() {
            Some(id) => TagIdOrRaw::Id(id),
            None => TagIdOrRaw::Raw(self.raw().unwrap().to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TagIdOrRaw {
    Id(TagId),
    Raw(String),
}

impl TagLike for TagIdOrRaw {
    fn id(&self) -> Option<TagId> {
        match self {
            TagIdOrRaw::Id(id) => Some(*id),
            TagIdOrRaw::Raw(_) => None,
        }
    }

    fn raw(&self) -> Option<&str> {
        match self {
            TagIdOrRaw::Id(_) => None,
            TagIdOrRaw::Raw(name) => Some(name),
        }
    }
}

impl TagLike for Tag {
    fn id(&self) -> Option<TagId> {
        Some(self.id)
    }

    fn raw(&self) -> Option<&str> {
        Some(&self.name)
    }
}

impl TagLike for TagId {
    fn id(&self) -> Option<TagId> {
        Some(*self)
    }
}

impl TagLike for &str {
    fn raw(&self) -> Option<&str> {
        Some(self)
    }
}

pub trait PlatformTagLike: Sized {
    fn id(&self) -> Option<PlatformTagId> {
        None
    }
    fn raw(&self) -> Option<(&PlatformIdOrRaw, &str)> {
        None
    }
    fn downcast(self) -> PlatformTagIdOrRaw {
        match self.id() {
            Some(id) => PlatformTagIdOrRaw::Id(id),
            None => {
                let raw = self.raw().unwrap();
                PlatformTagIdOrRaw::Raw((raw.0.clone(), raw.1.to_string()))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum PlatformTagIdOrRaw {
    Id(PlatformTagId),
    Raw((PlatformIdOrRaw, String)),
}

impl PlatformTagLike for PlatformTagIdOrRaw {
    fn id(&self) -> Option<PlatformTagId> {
        match self {
            PlatformTagIdOrRaw::Id(id) => Some(*id),
            PlatformTagIdOrRaw::Raw(_) => None,
        }
    }

    fn raw(&self) -> Option<(&PlatformIdOrRaw, &str)> {
        match self {
            PlatformTagIdOrRaw::Id(_) => None,
            PlatformTagIdOrRaw::Raw(name) => Some((&name.0, &name.1)),
        }
    }
}

impl PlatformTagLike for PlatformTag {
    fn id(&self) -> Option<PlatformTagId> {
        Some(self.id)
    }
}

impl PlatformTagLike for PlatformTagId {
    fn id(&self) -> Option<PlatformTagId> {
        Some(*self)
    }
}

impl PlatformTagLike for (&PlatformIdOrRaw, &str) {
    fn raw(&self) -> Option<(&PlatformIdOrRaw, &str)> {
        Some((&self.0, &self.1))
    }
}

impl<T> From<(T, String)> for PlatformTagIdOrRaw
where
    T: Into<PlatformIdOrRaw>,
{
    fn from(value: (T, String)) -> Self {
        PlatformTagIdOrRaw::Raw((value.0.into(), value.1))
    }
}
