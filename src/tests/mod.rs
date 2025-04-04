use crate::file_meta::FileMeta;

#[cfg(feature = "importer")]
mod importer;

#[cfg(feature = "utils")]
mod manager;

#[test]
fn test_file_meta_path() {
    use crate::id::{AuthorId, FileMetaId, PostId};

    let file_meta = FileMeta {
        id: FileMetaId::new(123),
        filename: "test.jpg".to_string(),
        author: AuthorId::new(456),
        post: PostId::new(789),
        mime: Default::default(),
        extra: Default::default(),
    };

    let path = file_meta.path();
    assert_eq!(path.to_str().unwrap(), "456/789/test.jpg");
}
