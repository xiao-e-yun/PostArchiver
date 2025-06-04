use std::hash::Hash;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    PlatformId, TagId,
};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    pub fn import_tag(&self, tag: UnsyncTag) -> Result<TagId, rusqlite::Error> {
        let find_tag = (tag.name.as_str(), tag.platform);
        match self.find_tag(&find_tag)? {
            Some(id) => Ok(id),
            None => self.add_tag(tag.name, tag.platform),
        }
    }

    pub fn import_tags(
        &self,
        tags: impl IntoIterator<Item = UnsyncTag>,
    ) -> Result<Vec<TagId>, rusqlite::Error> {
        tags.into_iter()
            .map(|tag| self.import_tag(tag))
            .collect::<Result<Vec<TagId>, rusqlite::Error>>()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnsyncTag {
    pub name: String,
    pub platform: Option<PlatformId>,
}
