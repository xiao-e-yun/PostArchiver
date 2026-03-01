# Query Builder API — 實作計畫

## 目標

以 **builder pattern** 取代 `XxxFilter` 結構，在 `PostArchiverManager` 上提供
`.posts()`、`.authors()`、`.tags()`、`.platforms()`、`.collections()` 五個入口，
各自回傳對應的查詢 builder。

透過 **Typestate**（`NoRelations`/`WithRelations`、`NoTotal`/`WithTotal`）在編譯期決定
`.query()` 的回傳型別（`Vec<T>` 或 `PageResult<T>`）。

---

## 設計決策

| 問題 | 決策 |
|------|------|
| `.tags([id1, id2])` 語意 | **AND** — 同時擁有所有指定 tag 的 post（每個 tag 用 EXISTS subquery 實作） |
| `.query()` 回傳型別 | **Typestate** — `R × T` 共 4 種組合，編譯期決定 |
| 預設 `.query()` | 回傳 `Vec<T>`；需加 `.with_total()` 才有 `PageResult<T>` |
| 排序 | 透過 `.sort(field, dir)` 方法提供 |
| Platform | 資料量小，`PlatformQuery` 簡化，不加 typestate |
| FileMeta | 不需 builder，只提供 `get_file_meta` / `find_file_meta` 直接方法 |

---

## API 使用範例

```rust
// Vec<Post>
let posts = manager.posts()
    .platform(platform_id)
    .tags(&[tag_a, tag_b])       // AND：同時有 tag_a 且 tag_b
    .pagination(20, 1)
    .sort(PostSort::Updated, SortDir::Desc)
    .query()?;

// PageResult<Post>（含 total）
let page = manager.posts()
    .pagination(20, 2)
    .with_total()
    .query()?;
// page.total, page.items

// Vec<PostWithRelations>
let full = manager.posts()
    .id(post_id)
    .relations()
    .query()?;

// PageResult<PostWithRelations>
let full_page = manager.posts()
    .author(author_id)
    .pagination(10, 1)
    .relations()
    .with_total()
    .query()?;

// Author 查詢
let authors = manager.authors()
    .name_contains("foo")
    .sort(AuthorSort::Name, SortDir::Asc)
    .with_total()
    .query()?;

// Tag 查詢
let tags = manager.tags()
    .platform(Some(platform_id))
    .query()?;

// Platform 查詢
let platforms = manager.platforms().query()?;

// Collection 查詢
let collections = manager.collections()
    .name_contains("series")
    .pagination(10, 1)
    .query()?;
```

---

## 公共基礎設施（`src/query/mod.rs`）

```rust
/// 分頁參數（page 從 1 開始，SQL offset = (page - 1) * limit）
pub struct Pagination {
    pub limit: u64,
    pub page: u64,
}

/// 排序方向
pub enum SortDir { Asc, Desc }

/// 分頁查詢結果
pub struct PageResult<T> {
    pub items: Vec<T>,
    pub total: u64,
}

/// Typestate 標記
pub struct NoRelations;
pub struct WithRelations;
pub struct NoTotal;
pub struct WithTotal;
```

### `QueryExecute` trait

4 種 typestate 組合各自 `impl QueryExecute`：

| Typestate | `Output` |
|-----------|----------|
| `NoRelations, NoTotal` | `Vec<Post>` |
| `WithRelations, NoTotal` | `Vec<PostWithRelations>` |
| `NoRelations, WithTotal` | `PageResult<Post>` |
| `WithRelations, WithTotal` | `PageResult<PostWithRelations>` |

---

## 各模塊設計

### `src/query/post.rs`

```rust
pub struct PostQuery<'a, C, R = NoRelations, T = NoTotal> {
    manager: &'a PostArchiverManager<C>,
    platforms: Vec<PlatformId>,     // OR filter（IN (...)）
    tags: Vec<TagId>,               // AND filter（每個 tag 一個 EXISTS subquery）
    author: Option<AuthorId>,
    ids: Vec<PostId>,               // IN (?)
    limit: u64,
    page: u64,
    sort: PostSort,
    sort_dir: SortDir,
    _r: PhantomData<R>,
    _t: PhantomData<T>,
}
```

**Builder 方法**（皆回傳 `Self`）：

| 方法 | 說明 |
|------|------|
| `.platform(id)` | 加入單個 platform 篩選 |
| `.platforms(ids)` | 加入多個 platform 篩選（OR） |
| `.tags(ids)` | 加入 tag AND 篩選 |
| `.author(id)` | 篩選特定作者的 post |
| `.id(id)` | 篩選單個 post id |
| `.ids(ids)` | 篩選多個 post id |
| `.pagination(limit, page)` | 設定分頁 |
| `.sort(field, dir)` | 設定排序 |

**型別轉換方法**：

| 方法 | 回傳 |
|------|------|
| `.relations()` | `PostQuery<'a, C, WithRelations, T>` |
| `.with_total()` | `PostQuery<'a, C, R, WithTotal>` |

**直接方法（非 builder）**：

```rust
impl<C: PostArchiverConnection> PostArchiverManager<C> {
    pub fn get_post(&self, id: PostId) -> Result<Option<Post>, rusqlite::Error>;
    pub fn find_post_by_source(&self, source: &str) -> Result<Option<PostId>, rusqlite::Error>;
    pub fn get_post_with_relations(&self, id: PostId) -> Result<Option<PostWithRelations>, rusqlite::Error>;
    pub fn list_post_authors(&self, post: PostId) -> Result<Vec<Author>, rusqlite::Error>;
    pub fn list_post_tags(&self, post: PostId) -> Result<Vec<Tag>, rusqlite::Error>;
    pub fn list_post_files(&self, post: PostId) -> Result<Vec<FileMeta>, rusqlite::Error>;
    pub fn list_post_collections(&self, post: PostId) -> Result<Vec<Collection>, rusqlite::Error>;
}
```

**排序欄位**：

```rust
pub enum PostSort { Updated, Published, Title, Id }

pub struct PostWithRelations {
    pub post: Post,
    pub authors: Vec<Author>,
    pub tags: Vec<Tag>,
    pub files: Vec<FileMeta>,
    pub collections: Vec<Collection>,
}
```

**Tags AND SQL 模式**：

```sql
-- 每個 tag 追加一個 EXISTS subquery
AND EXISTS (SELECT 1 FROM post_tags WHERE post = posts.id AND tag = ?)
```

---

### `src/query/author.rs`

```rust
pub struct AuthorQuery<'a, C, R = NoRelations, T = NoTotal> {
    manager: &'a PostArchiverManager<C>,
    name_contains: Option<String>,
    limit: u64,
    page: u64,
    sort: AuthorSort,
    sort_dir: SortDir,
    _r: PhantomData<R>,
    _t: PhantomData<T>,
}

pub enum AuthorSort { Updated, Name, Id }

pub struct AuthorWithRelations {
    pub author: Author,
    pub aliases: Vec<Alias>,
    pub posts: Vec<Post>,
}
```

**直接方法**：

```rust
pub fn get_author(&self, id: AuthorId) -> Result<Option<Author>, rusqlite::Error>;
pub fn find_author_by_alias(&self, source: &str, platform: PlatformId) -> Result<Option<AuthorId>, rusqlite::Error>;
pub fn list_author_aliases(&self, author: AuthorId) -> Result<Vec<Alias>, rusqlite::Error>;
pub fn list_author_posts(&self, author: AuthorId) -> Result<Vec<Post>, rusqlite::Error>;
```

---

### `src/query/tag.rs`

```rust
pub struct TagQuery<'a, C, T = NoTotal> {
    manager: &'a PostArchiverManager<C>,
    platforms: Vec<Option<PlatformId>>,  // None 表示無平台的 tag
    name_contains: Option<String>,
    limit: u64,
    page: u64,
    sort_dir: SortDir,
    _t: PhantomData<T>,
}
```

**直接方法**：

```rust
pub fn get_tag(&self, id: TagId) -> Result<Option<Tag>, rusqlite::Error>;
pub fn find_tag(&self, name: &str, platform: Option<PlatformId>) -> Result<Option<TagId>, rusqlite::Error>;
pub fn list_tag_posts(&self, tag: TagId) -> Result<Vec<Post>, rusqlite::Error>;
```

---

### `src/query/platform.rs`

Platform 資料量小，簡化設計：

```rust
pub struct PlatformQuery<'a, C> {
    manager: &'a PostArchiverManager<C>,
}

impl PlatformQuery<'_, C> {
    pub fn query(self) -> Result<Vec<Platform>, rusqlite::Error>;
}
```

**直接方法**：

```rust
pub fn get_platform(&self, id: PlatformId) -> Result<Option<Platform>, rusqlite::Error>;
pub fn find_platform(&self, name: &str) -> Result<Option<PlatformId>, rusqlite::Error>;
pub fn list_platform_posts(&self, platform: PlatformId) -> Result<Vec<Post>, rusqlite::Error>;
pub fn list_platform_tags(&self, platform: PlatformId) -> Result<Vec<Tag>, rusqlite::Error>;
```

---

### `src/query/collection.rs`

```rust
pub struct CollectionQuery<'a, C, T = NoTotal> {
    manager: &'a PostArchiverManager<C>,
    name_contains: Option<String>,
    limit: u64,
    page: u64,
    sort_dir: SortDir,
    _t: PhantomData<T>,
}
```

**直接方法**：

```rust
pub fn get_collection(&self, id: CollectionId) -> Result<Option<Collection>, rusqlite::Error>;
pub fn find_collection_by_source(&self, source: &str) -> Result<Option<CollectionId>, rusqlite::Error>;
pub fn list_collection_posts(&self, collection: CollectionId) -> Result<Vec<Post>, rusqlite::Error>;
```

---

### `src/query/file_meta.rs`

無 builder，只提供點查詢：

```rust
pub fn get_file_meta(&self, id: FileMetaId) -> Result<Option<FileMeta>, rusqlite::Error>;
pub fn find_file_meta(&self, post: PostId, filename: &str) -> Result<Option<FileMetaId>, rusqlite::Error>;
```

---

## 目錄結構

```
src/query/
  mod.rs          ← Pagination, SortDir, PageResult, Typestate markers, QueryExecute trait
  post.rs         ← PostQuery, PostSort, PostWithRelations
                     get_post, find_post_by_source, get_post_with_relations
                     list_post_authors, list_post_tags, list_post_files, list_post_collections
  author.rs       ← AuthorQuery, AuthorSort, AuthorWithRelations
                     get_author, find_author_by_alias
                     list_author_aliases, list_author_posts
  tag.rs          ← TagQuery
                     get_tag, find_tag, list_tag_posts
  platform.rs     ← PlatformQuery
                     get_platform, find_platform
                     list_platform_posts, list_platform_tags
  collection.rs   ← CollectionQuery
                     get_collection, find_collection_by_source, list_collection_posts
  file_meta.rs    ← get_file_meta, find_file_meta
```

---

## 實作步驟

1. **建立 `src/query/mod.rs`**
   - 定義 `Pagination`、`SortDir`、`PageResult`、Typestate 標記
   - 宣告並 re-export 子模塊
   - 在 `src/lib.rs` 加入 `#[cfg(feature = "utils")] pub mod query;`

2. **實作 `query/post.rs`**
   - `PostQuery` builder + 4 種 typestate `impl QueryExecute`
   - `PostSort`、`PostWithRelations`
   - 直接方法：`get_post`、`find_post_by_source`、`get_post_with_relations`
   - 跨實體：`list_post_authors`、`list_post_tags`、`list_post_files`、`list_post_collections`

3. **實作 `query/author.rs`**
   - `AuthorQuery` builder + typestate
   - `AuthorSort`、`AuthorWithRelations`
   - 直接方法：`get_author`、`find_author_by_alias`、`list_author_aliases`、`list_author_posts`

4. **實作 `query/tag.rs`**
   - `TagQuery` builder + typestate
   - 直接方法：`get_tag`、`find_tag`、`list_tag_posts`

5. **實作 `query/platform.rs`**
   - `PlatformQuery`（簡化，無 typestate）
   - 直接方法：`get_platform`、`find_platform`、`list_platform_posts`、`list_platform_tags`

6. **實作 `query/collection.rs`**
   - `CollectionQuery` builder + typestate
   - 直接方法：`get_collection`、`find_collection_by_source`、`list_collection_posts`

7. **實作 `query/file_meta.rs`**
   - 直接方法：`get_file_meta`、`find_file_meta`

8. **更新 `src/importer/`**
   - `importer/post.rs`、`importer/author.rs` 等中重複的 `find` SQL 改為呼叫 query 模塊同名方法

9. **更新 `src/tests/helpers.rs`**
   - 所有裸 SQL 的 `get_xxx`、`find_xxx`、`list_xxx` 改為呼叫 query 模塊方法
   - 只保留 `add_xxx` 等寫入 helper

10. **驗證**
    - `cargo build -F=utils` — 確認編譯通過
    - `cargo test -F=importer` — 確認所有既有測試通過
    - 手動確認 4 種 typestate 組合的 `.query()` 回傳型別正確
