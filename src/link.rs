use serde::{ Deserialize, Serialize };
#[cfg(feature = "typescript")]
use ts_rs::TS;

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

    /// Create a proxy link
    pub fn proxy(self, url: &str) -> Link {
        let name = format!("{} [{}]", self.name, self.url);
        let url = url.to_string();
        Link { name, url }
    }
}