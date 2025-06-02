use crate::{Collection, Post};

use super::macros::as_table;

as_table! {
    Post {
        id: "id",
        source: "source",
        title: "title",
        content: "content" => json,
        thumb: "thumb",
        comments: "comments" => json,
        updated: "updated",
        published: "published",
        platform: "platform",
    }
}

as_table! {
    Collection {
        id: "id",
        name: "name",
        thumb: "thumb",
        description: "description",
    }
}
