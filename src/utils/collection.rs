use crate::{Collection, CollectionId};

pub trait CollectionLike: Sized {
    fn id(&self) -> Option<CollectionId> {
        None
    }
    fn raw(&self) -> Option<&str> {
        None
    }
    fn downcast(self) -> CollectionIdOrRaw {
        match self.id() {
            Some(id) => CollectionIdOrRaw::Id(id),
            None => CollectionIdOrRaw::Raw(self.raw().unwrap().to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CollectionIdOrRaw {
    Id(CollectionId),
    Raw(String),
}

impl CollectionLike for CollectionIdOrRaw {
    fn id(&self) -> Option<CollectionId> {
        match self {
            CollectionIdOrRaw::Id(id) => Some(*id),
            CollectionIdOrRaw::Raw(_) => None,
        }
    }

    fn raw(&self) -> Option<&str> {
        match self {
            CollectionIdOrRaw::Id(_) => None,
            CollectionIdOrRaw::Raw(name) => Some(name),
        }
    }
}

impl CollectionLike for Collection {
    fn id(&self) -> Option<CollectionId> {
        Some(self.id)
    }

    fn raw(&self) -> Option<&str> {
        Some(&self.name)
    }
}

impl CollectionLike for CollectionId {
    fn id(&self) -> Option<CollectionId> {
        Some(*self)
    }
}

impl CollectionLike for &str {
    fn raw(&self) -> Option<&str> {
        Some(self)
    }
}
