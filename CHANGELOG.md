# Changelog

## 0.6.0

### Database (run `post-archiver-bridge` on 0.5 archives)

- `tags.name` is no longer globally unique. General tags (`platform IS NULL`) are unique by name,
  platform tags by `(platform, name)`.
- `author_aliases.source` is no longer globally unique, only per platform. New index
  `author_aliases_target_idx` on `target`, used to list an author's aliases and by the cascade
  when an author is deleted.
- `file_metas` is unique by `(post, filename)`. The bridge merges existing duplicates and points
  `posts.content` and every `thumb` at the kept row.
- Foreign keys are now enforced, so `ON DELETE CASCADE` / `SET NULL` take effect. Deleting a post
  removes its file metas and relation rows. The bridge removes orphan rows left by older versions.
- The database is opened in WAL mode. Back up `post-archiver.db-wal` / `-shm` together with the
  database, or close every connection first.

### Features

- `utils` is enabled by default. `importer` implies `utils`.

### Breaking API changes

| 0.5 | 0.6 |
|-----|-----|
| `PostArchiverManager<C>`, `manager.transaction()` / `commit()` | `PostArchiverManager` (no type parameter); group imports with `manager.importer().savepoint()` and `commit()` |
| `manager.bind(id)` | same, through `PostArchiverBindExt`, which also works on any `Deref<Target = Connection>` |
| `get_post(id)`, `get_author(id)`, ... | `manager.posts().get(id)`, `manager.authors().get(id)`, ... (`PostArchiverQueryExt`); any `FromQuery` type works, e.g. `get::<MyCard>(id)` |
| `find_post`, `find_author_by_alias`, `find_tag`, ... | `importer.lookup_post`, `lookup_author`, `lookup_tag`, ... |
| `list_post_tags(post)`, `add_post_tags(post, ..)`, ... | `manager.bind(post).list_tags()`, `.add_tags(..)`, ... |
| `import_post(post, update_relation) -> (PostId, Vec<AuthorId>, Vec<CollectionId>)` | `import_post(post) -> PostId` |
| `import_posts`, `import_tags`, `import_collections` | call the single-item versions inside a savepoint |
| `UnsyncPost::new(PlatformId, ..)` | `UnsyncPost::new(String, ..)`: the platform is imported by name |
| `UnsyncFileMeta { data: FileMetaContent, .. }` | `UnsyncFileMeta { source: PathBuf, .. }`: the file is copied in; deleting the source is up to the caller |
| `UpdateFileMeta::content()` / `path()` | removed; re-import the file with the same filename to replace its content |
| `Paginate` / `Countable` / `Sortable` wrappers, `with_total()` | `q.pagination`, `q.order` fields and `q.count()` on the query builder |
| `#[macro_export] as_table!` | crate-private; use `impl_from_query!` to define projections (it no longer needs `rusqlite` in your dependencies) |

### Fixes

- A new post's `updated` time is the one given by the importer, not the import time.
- Importing a file that already exists updates its MIME type as well as `extra`.
- Files are staged as `<filename>.importing` and only renamed into place when the outermost
  savepoint commits; rolled-back imports leave no files behind and never touch existing ones.
