use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript")]
use ts_rs::TS;

/// A named URL reference that can optionally use a proxy service
///
/// # Safety
/// - Names must not be empty
/// - URLs must be valid url.
///
/// # Examples
/// ```rust
/// use post_archiver::Link;
///
/// let github = Link::new("GitHub", "https://github.com/user");
/// assert_eq!(github.name, "GitHub");
/// assert_eq!(github.url, "https://github.com/user");
/// ```
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

    /// Creates a proxied version of the link, preserving the original URL in the name
    ///
    /// # Examples
    /// ```rust
    /// use post_archiver::Link;
    ///
    /// // Create a link to a restricted resource
    /// let original = Link::new("Resource", "https://internal.example.com");
    ///
    /// // Create a publicly accessible proxied version
    /// let proxied = original.proxy("https://proxy.public.com/internal");
    /// assert_eq!(proxied.name, "Resource [https://internal.example.com]");
    /// assert_eq!(proxied.url, "https://proxy.public.com/internal");
    /// ```
    pub fn proxy(self, url: &str) -> Link {
        let name = format!("{} [{}]", self.name, self.url);
        let url = url.to_string();
        Link { name, url }
    }
}
