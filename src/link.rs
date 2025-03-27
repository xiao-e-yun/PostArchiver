use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript")]
use ts_rs::TS;

/// Represents a link to a any url
///
/// # Structure
/// `name` Name of the link  
/// `url` URL of the link
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Link {
    pub name: String,
    pub url: String,
}

impl Link {
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
        }
    }

    /// Create a new link with the given url
    ///
    /// # Example
    /// ```rust
    /// use post_archiver::Link;
    ///
    /// let link = Link::new("Name", "https://example.com");
    /// assert_eq!(link.name, "Name");
    ///
    /// let link = link.proxy("https://proxy.com");
    /// assert_eq!(link.name, "Name [https://example.com]");
    /// assert_eq!(link.url, "https://proxy.com");
    /// ```
    pub fn proxy(self, url: &str) -> Link {
        let name = format!("{} [{}]", self.name, self.url);
        let url = url.to_string();
        Link { name, url }
    }
}
