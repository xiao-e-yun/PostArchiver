use crate::{file_meta::FileMeta, id::{ AuthorId, FileMetaId, PostId }, utils::get_mime};

#[test]
fn test_guess_mime() {

    let file_meta = FileMeta {
        id: FileMetaId::new(0),
        author: AuthorId::new(0),
        post: PostId::new(0),
        filename: "test.jpg".to_string(),
        mime: String::new(),
        extra: Default::default(),
    };
    let mime = get_mime(&file_meta.filename);
    assert_eq!(mime, "image/jpeg");
}
