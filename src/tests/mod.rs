use crate::file_meta::FileMeta;

#[cfg(feature = "importer")]
mod importer;

#[cfg(feature = "editor")]
mod editor;

#[cfg(feature = "utils")]
mod manager;

#[test]
fn test_file_meta_path() {
    use crate::id::{FileMetaId, PostId};

    let file_meta = FileMeta {
        id: FileMetaId::new(123),
        filename: "test.jpg".to_string(),
        post: PostId::new(789),
        mime: Default::default(),
        extra: Default::default(),
    };

    let path = file_meta.path();
    assert_eq!(path.to_str().unwrap(), "0/123/test.jpg");

    let file_meta = FileMeta {
        id: FileMetaId::new(123),
        filename: "test.png".to_string(),
        post: PostId::new(2789),
        mime: Default::default(),
        extra: Default::default(),
    };

    let path = file_meta.path();
    assert_eq!(path.to_str().unwrap(), "1/741/test.png");
}
