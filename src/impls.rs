use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
    mem,
    path::PathBuf,
};

use crate::structs::*;

//==============================================================================
// List
//==============================================================================
impl ArchiveAuthorsList {
    pub fn from_vector(vec: Vec<ArchiveAuthor>) -> Self {
        let mut vec: Vec<ArchiveAuthorsItem> = vec.into_iter().map(|a| a.into()).collect();
        vec.sort_by(|a, b| a.id.cmp(&b.id));
        ArchiveAuthorsList(vec)
    }
    pub fn extend(&mut self, rhs: Self) {
        let mut authors_map = HashMap::new();

        for author in self.0.iter().cloned() {
            authors_map.insert(author.id.clone(), author);
        }

        for author in rhs.0.iter().cloned() {
            if let Some(old_author) = authors_map.get_mut(&author.id) {
                old_author.extend(author);
            } else {
                authors_map.insert(author.id.clone(), author);
            }
        }

        let mut authors: Vec<ArchiveAuthorsItem> = authors_map.into_values().collect();
        authors.sort_by(|a, b| a.id.cmp(&b.id));
        self.0 = authors;
    }

    pub fn authors(&self) -> Vec<ArchiveAuthorsItem> {
        self.0.clone()
    }
}

impl ArchiveAuthorsItem {
    pub fn extend(&mut self, rhs: Self) {
        self.id = rhs.id;
        self.name = rhs.name;
        self.from = rhs.from;
        self.thumb = rhs.thumb.or(self.thumb.clone());
    }
}

//==============================================================================
// Author
//==============================================================================

impl ArchiveAuthor {
    pub fn extend(&mut self, rhs: Self) {
        self.id = rhs.id;
        self.name = rhs.name;
        self.from = rhs.from;
        self.thumb = rhs.thumb.or(self.thumb.clone());
    }
}

impl Into<ArchiveAuthorsItem> for ArchiveAuthor {
    fn into(self) -> ArchiveAuthorsItem {
        ArchiveAuthorsItem {
            id: self.id,
            name: self.name,
            thumb: self.thumb,
            from: self.from,
        }
    }
}

impl ArchiveAuthorPosts {
    pub fn all(&self) -> Vec<ArchivePostShort> {
        self.0.clone()
    }
    pub fn get(&self, id: &str) -> Option<&ArchivePostShort> {
        self.0.iter().find(|p| p.id == id)
    }
    pub fn extend(&mut self, rhs: &mut Self) {
        let mut posts_map = HashMap::new();

        for post in mem::take(&mut self.0).into_iter() {
            posts_map.insert(post.id.clone(), post);
        }

        for post in mem::take(&mut rhs.0).into_iter() {
            let entry = posts_map.entry(post.id.clone());
            match entry {
                Entry::Occupied(mut occupied) => {
                    let expired = occupied.get().updated < post.updated;
                    if expired {
                        occupied.insert(post);
                    };
                }
                Entry::Vacant(vacant) => {
                    vacant.insert(post);
                }
            }
        }

        let mut posts: Vec<ArchivePostShort> = posts_map.into_values().collect();
        posts.sort_by(|a, b| a.updated.cmp(&b.updated).then(a.id.cmp(&b.id)));
        self.0 = posts;
    }
}
//==============================================================================
// Post
//==============================================================================
impl Hash for ArchivePostShort {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.from.hash(state);
        self.author.hash(state);
    }
}

impl Into<ArchivePostShort> for ArchivePost {
    fn into(self) -> ArchivePostShort {
        let url = PathBuf::from(&self.author).join(&self.id);
        ArchivePostShort {
            url,
            id: self.id,
            title: self.title,
            author: self.author,
            from: self.from,
            updated: self.updated,
            thumb: self.thumb,
        }
    }
}
//==============================================================================
// Utils
//==============================================================================
impl ArchiveFile {
    pub fn filename(&self) -> &PathBuf {
        match self {
            ArchiveFile::Image { filename, .. } => filename,
            ArchiveFile::Video { filename, .. } => filename,
            ArchiveFile::File { filename, .. } => filename,
        }
    }
    pub fn path(&self) -> &PathBuf {
        match self {
            ArchiveFile::Image { path, .. } => path,
            ArchiveFile::Video { path, .. } => path,
            ArchiveFile::File { path, .. } => path,
        }
    }
    pub fn is_image(&self) -> bool {
        matches!(self, ArchiveFile::Image { .. })
    }
}