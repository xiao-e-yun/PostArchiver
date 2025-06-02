use crate::{Alias, Author};

use super::macros::as_table;

as_table! {
    Author {
        id: "id",
        name: "name",
        thumb: "thumb",
        updated: "updated",
    }
}

as_table! {
    Alias {
        source: "source",
        platform: "platform",
        target: "target",
        link: "link",
    }
}
