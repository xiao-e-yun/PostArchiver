use crate::FileMeta;

use super::macros::as_table;

as_table!(
    FileMeta {
        id: "id",
        post: "post",
        filename: "filename",
        mime: "mime",
        extra: "extra" => json,
    }
);
