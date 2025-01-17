use crate::wrraper;

use core::fmt;
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript")]
use ts_rs::TS;

macro_rules! define_id {
    ($name:ident) => {
        #[cfg_attr(feature = "typescript", derive(TS))]
        #[cfg_attr(feature = "typescript", ts(export))]
        #[derive(
            Deserialize,
            Serialize,
            Debug,
            Clone,
            Copy,
            Hash,
            PartialEq,
            Eq,
        )]
        pub struct $name(pub u32);
        wrraper!($name: u32);

        impl $name {
            pub fn new(id: u32) -> Self {
                Self(id)
            }
            pub fn raw(&self) -> u32 {
                self.0
            }
        }

        impl From<usize> for $name {
            fn from(id: usize) -> Self {
                Self(id as u32)
            }
        }

        impl From<$name> for usize {
            fn from(id: $name) -> usize {
                id.0 as usize
            }
        }

        impl AsRef<u32> for $name {
            fn as_ref(&self) -> &u32 {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_id!(AuthorId);
define_id!(PostId);
define_id!(FileMetaId);
define_id!(PostTagId);
