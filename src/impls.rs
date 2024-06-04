use std::{collections::{HashMap, HashSet}, path::PathBuf};

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
      self.url = rhs.url;
      self.name = rhs.name;
      self.r#type = rhs.r#type;
      self.thumb = rhs.thumb.or(self.thumb.clone());
  }
}

//==============================================================================
// Author
//==============================================================================

impl ArchiveAuthor {
  pub fn extend(&mut self, rhs: Self) {
      let mut posts = HashSet::new();
      posts.extend(self.posts.iter().cloned());
      posts.extend(rhs.posts.iter().cloned());
      let mut posts: Vec<ArchivePostShort> = posts.into_iter().collect();
      posts.sort_by(|a, b| a.updated.cmp(&b.updated));
      posts.reverse();

      self.id = rhs.id;
      self.posts = posts;
      self.name = rhs.name;
      self.r#type = rhs.r#type;
      self.thumb = rhs.thumb.or(self.thumb.clone());
  }
}

impl Into<ArchiveAuthorsItem> for ArchiveAuthor {
  fn into(self) -> ArchiveAuthorsItem {
      ArchiveAuthorsItem {
          id: self.id,
          url: self.url,
          name: self.name,
          thumb: self.thumb,
          r#type: self.r#type,
      }
  }
}
//==============================================================================
// Post
//==============================================================================

impl Into<ArchivePostShort> for ArchivePost {
  fn into(self) -> ArchivePostShort {
      ArchivePostShort {
          id: self.id,
          title: self.title,
          author: self.author,
          r#type: self.r#type,
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