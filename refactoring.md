# query 模塊設計與實現

## 目標
- 為 post-archiver 設計並實現 `query` 模塊，提供所有讀取操作（`list`、`get`、`find`）
- 與已完成的 `manager`（連線管理 + `Binded` 寫入操作）和 `importer`（插入邏輯）形成完整的三層架構
- 支援分頁、排序、篩選，以及跨實體的複合查詢

## 現狀

### 已完成
- **manager 模塊**：`PostArchiverManager<C>` 僅負責連線管理（`open`、`create`、`transaction`）與 `bind(id)` → `Binded<'_, Id, C>` 的寫入操作（`update`、`delete`、關聯增刪）
- **importer 模塊**：透過 `impl<T: PostArchiverConnection> PostArchiverManager<T>` 提供 `import_post`、`import_author`、`import_tag` 等插入邏輯
- **Binded 上的 `value()`**：各 `Binded<'_, XxxId>` 已提供 `value()` 方法以取得單一實體資料

### 缺失
- **無 `list` 操作**：無法列出所有 Post、Author、Tag 等，目前僅在 `tests/helpers.rs` 以裸 SQL 實現
- **無 `find` 操作**：無法按業務鍵（source、name、alias）查詢實體 ID
- **無複合查詢**：無法一次取得 Post 及其所有關聯 Author、Tag、FileMeta
- **無分頁/排序/篩選**：所有查詢都是全表掃描

目前 `tests/helpers.rs`（441 行）中的裸 SQL 查詢函數正好代表了 query 模塊所需的全部操作。

---

## 設計原則

1. **直接 `impl` 在 `PostArchiverManager<C>` 上**
   查詢方法與 importer 一樣，透過 `impl<C: PostArchiverConnection> PostArchiverManager<C>` 來擴展。無需引入額外 trait 或包裝結構 — 使用者直接 `manager.get_post(id)` 即可，API 最簡潔。

2. **讀取專用：所有方法接收 `&self`**
   query 模塊不修改資料，所有方法僅需 `&self`。

3. **與 `Binded::value()` 的關係**
   `Binded::value()` 保留為內部快捷方式（已知 ID 的簡單 SELECT），query 模塊的 `get_xxx` 返回 `Option` 語義（ID 可能不存在）。兩者並存互不衝突。

4. **模塊功能邊界**
   - `get_xxx(id)` → 按主鍵取得完整實體，返回 `Option<T>`
   - `find_xxx(...)` → 按業務鍵查詢 ID，返回 `Option<XxxId>`
   - 複合查詢 → 一次拉取實體及其所有關聯數據

---

## 公共基礎設施

### 分頁

```rust
// src/query/mod.rs

/// 分頁參數
#[derive(Debug, Clone)]
pub struct Pagination {
    /// 偏移量（跳過前 N 筆）
    pub offset: u64,
    /// 每頁最多返回筆數
    pub limit: u64,
}

impl Pagination {
    pub fn new(offset: u64, limit: u64) -> Self {
        Self { offset, limit }
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 50,
        }
    }
}
```

### 排序

```rust
/// 排序方向
#[derive(Debug, Clone, Copy, Default)]
pub enum SortDir {
    #[default]
    Asc,
    Desc,
}

impl SortDir {
    fn as_sql(&self) -> &'static str {
        match self {
            SortDir::Asc => "ASC",
            SortDir::Desc => "DESC",
        }
    }
}
```

### 查詢結果容器

```rust
/// 分頁查詢結果
#[derive(Debug, Clone)]
pub struct PageResult<T> {
    /// 本頁數據
    pub items: Vec<T>,
    /// 符合篩選條件的總筆數（忽略分頁）
    pub total: u64,
}
```

---

## 各實體查詢 API

### Post 查詢

```rust
// src/query/post.rs

/// Post 排序欄位
#[derive(Debug, Clone, Copy, Default)]
pub enum PostSort {
    #[default]
    Updated,
    Published,
    Title,
    Id,
}

/// Post 篩選條件
#[derive(Debug, Clone, Default)]
pub struct PostFilter {
    /// 按平台篩選
    pub platform: Option<PlatformId>,
    /// 標題關鍵字（LIKE %keyword%）
    pub title_contains: Option<String>,
    /// 分頁
    pub pagination: Pagination,
    /// 排序欄位
    pub sort: PostSort,
    /// 排序方向
    pub sort_dir: SortDir,
}

/// 完整的 Post 含所有關聯數據
#[derive(Debug, Clone)]
pub struct PostWithRelations {
    pub post: Post,
    pub authors: Vec<Author>,
    pub tags: Vec<Tag>,
    pub files: Vec<FileMeta>,
    pub collections: Vec<Collection>,
}

impl<C: PostArchiverConnection> PostArchiverManager<C> {
    /// 按主鍵取得 Post
    pub fn get_post(&self, id: PostId) -> Result<Option<Post>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM posts WHERE id = ?")?;
        stmt.query_row([id], Post::from_row).optional()
    }

    /// 按 source 查詢 PostId
    pub fn find_post_by_source(&self, source: &str) -> Result<Option<PostId>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT id FROM posts WHERE source = ?")?;
        stmt.query_row([source], |row| row.get(0)).optional()
    }

    /// 帶篩選/分頁/排序的 Post 列表
    pub fn list_posts(&self, filter: PostFilter) -> Result<PageResult<Post>, rusqlite::Error> { ... }

    /// 取得 Post 及其所有關聯（Authors、Tags、Files、Collections）
    pub fn get_post_with_relations(&self, id: PostId) -> Result<Option<PostWithRelations>, rusqlite::Error> {
        let Some(post) = self.get_post(id)? else { return Ok(None) };
        let authors = self.list_post_authors(id)?;
        let tags = self.list_post_tags(id)?;
        let files = self.list_post_files(id)?;
        let collections = self.list_post_collections(id)?;
        Ok(Some(PostWithRelations { post, authors, tags, files, collections }))
    }

    /// 取得某篇 Post 的所有 Author（完整實體，非僅 ID）
    pub fn list_post_authors(&self, post: PostId) -> Result<Vec<Author>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "SELECT authors.* FROM authors \
             INNER JOIN author_posts ON author_posts.author = authors.id \
             WHERE author_posts.post = ?"
        )?;
        let rows = stmt.query_map([post], Author::from_row)?;
        rows.collect()
    }

    /// 取得某篇 Post 的所有 Tag（完整實體）
    pub fn list_post_tags(&self, post: PostId) -> Result<Vec<Tag>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "SELECT tags.* FROM tags \
             INNER JOIN post_tags ON post_tags.tag = tags.id \
             WHERE post_tags.post = ?"
        )?;
        let rows = stmt.query_map([post], Tag::from_row)?;
        rows.collect()
    }

    /// 取得某篇 Post 的所有 FileMeta（完整實體）
    pub fn list_post_files(&self, post: PostId) -> Result<Vec<FileMeta>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "SELECT * FROM file_metas WHERE post = ?"
        )?;
        let rows = stmt.query_map([post], FileMeta::from_row)?;
        rows.collect()
    }

    /// 取得某篇 Post 的所有 Collection（完整實體）
    pub fn list_post_collections(&self, post: PostId) -> Result<Vec<Collection>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "SELECT collections.* FROM collections \
             INNER JOIN collection_posts ON collection_posts.collection = collections.id \
             WHERE collection_posts.post = ?"
        )?;
        let rows = stmt.query_map([post], Collection::from_row)?;
        rows.collect()
    }
}
```

### Author 查詢

```rust
// src/query/author.rs

/// Author 排序欄位
#[derive(Debug, Clone, Copy, Default)]
pub enum AuthorSort {
    #[default]
    Updated,
    Name,
    Id,
}

/// Author 篩選條件
#[derive(Debug, Clone, Default)]
pub struct AuthorFilter {
    /// 名稱關鍵字（LIKE %keyword%）
    pub name_contains: Option<String>,
    /// 分頁
    pub pagination: Pagination,
    /// 排序
    pub sort: AuthorSort,
    pub sort_dir: SortDir,
}

impl<C: PostArchiverConnection> PostArchiverManager<C> {
    /// 按主鍵取得 Author
    pub fn get_author(&self, id: AuthorId) -> Result<Option<Author>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM authors WHERE id = ?")?;
        stmt.query_row([id], Author::from_row).optional()
    }

    /// 按 alias 查詢 AuthorId（使用 source + platform 作為查詢條件）
    pub fn find_author_by_alias(
        &self,
        source: &str,
        platform: PlatformId,
    ) -> Result<Option<AuthorId>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "SELECT target FROM author_aliases WHERE platform = ? AND source = ?"
        )?;
        stmt.query_row(params![platform, source], |row| row.get(0)).optional()
    }

    /// 帶篩選/分頁/排序的 Author 列表
    pub fn list_authors(&self, filter: AuthorFilter) -> Result<PageResult<Author>, rusqlite::Error> { ... }

    /// 取得某位 Author 的所有 Alias
    pub fn list_author_aliases(&self, author: AuthorId) -> Result<Vec<Alias>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM author_aliases WHERE target = ?")?;
        let rows = stmt.query_map([author], Alias::from_row)?;
        rows.collect()
    }

    /// 取得某位 Author 的所有 Post（完整實體）
    pub fn list_author_posts(&self, author: AuthorId) -> Result<Vec<Post>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "SELECT posts.* FROM posts \
             INNER JOIN author_posts ON author_posts.post = posts.id \
             WHERE author_posts.author = ?"
        )?;
        let rows = stmt.query_map([author], Post::from_row)?;
        rows.collect()
    }
}
```

### Tag 查詢

```rust
// src/query/tag.rs

/// Tag 篩選條件
#[derive(Debug, Clone, Default)]
pub struct TagFilter {
    /// 按平台篩選
    pub platform: Option<Option<PlatformId>>,
    /// 名稱關鍵字
    pub name_contains: Option<String>,
    /// 分頁
    pub pagination: Pagination,
    pub sort_dir: SortDir,
}

impl<C: PostArchiverConnection> PostArchiverManager<C> {
    /// 按主鍵取得 Tag
    pub fn get_tag(&self, id: TagId) -> Result<Option<Tag>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM tags WHERE id = ?")?;
        stmt.query_row([id], Tag::from_row).optional()
    }

    /// 按 name + platform 查詢 TagId
    pub fn find_tag(
        &self,
        name: &str,
        platform: Option<PlatformId>,
    ) -> Result<Option<TagId>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "SELECT id FROM tags WHERE platform IS ? AND name = ?"
        )?;
        stmt.query_row(params![platform, name], |row| row.get(0)).optional()
    }

    /// 帶篩選/分頁的 Tag 列表
    pub fn list_tags(&self, filter: TagFilter) -> Result<PageResult<Tag>, rusqlite::Error> { ... }

    /// 取得某 Tag 關聯的所有 Post（完整實體）
    pub fn list_tag_posts(&self, tag: TagId) -> Result<Vec<Post>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "SELECT posts.* FROM posts \
             INNER JOIN post_tags ON post_tags.post = posts.id \
             WHERE post_tags.tag = ?"
        )?;
        let rows = stmt.query_map([tag], Post::from_row)?;
        rows.collect()
    }
}
```

### Platform 查詢

```rust
// src/query/platform.rs

impl<C: PostArchiverConnection> PostArchiverManager<C> {
    /// 按主鍵取得 Platform
    pub fn get_platform(&self, id: PlatformId) -> Result<Option<Platform>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM platforms WHERE id = ?")?;
        stmt.query_row([id], Platform::from_row).optional()
    }

    /// 按 name 查詢 PlatformId（COLLATE NOCASE）
    pub fn find_platform(&self, name: &str) -> Result<Option<PlatformId>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT id FROM platforms WHERE name = ?")?;
        stmt.query_row([name], |row| row.get(0)).optional()
    }

    /// 列出所有 Platform
    pub fn list_platforms(&self) -> Result<Vec<Platform>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM platforms")?;
        let rows = stmt.query_map([], Platform::from_row)?;
        rows.collect()
    }

    /// 取得某 Platform 下的所有 Post（完整實體）
    pub fn list_platform_posts(&self, platform: PlatformId) -> Result<Vec<Post>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM posts WHERE platform = ?")?;
        let rows = stmt.query_map([platform], Post::from_row)?;
        rows.collect()
    }

    /// 取得某 Platform 下的所有 Tag（完整實體）
    pub fn list_platform_tags(&self, platform: PlatformId) -> Result<Vec<Tag>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM tags WHERE platform = ?")?;
        let rows = stmt.query_map([platform], Tag::from_row)?;
        rows.collect()
    }
}
```

### Collection 查詢

```rust
// src/query/collection.rs

/// Collection 篩選條件
#[derive(Debug, Clone, Default)]
pub struct CollectionFilter {
    /// 名稱關鍵字
    pub name_contains: Option<String>,
    /// 分頁
    pub pagination: Pagination,
    pub sort_dir: SortDir,
}

impl<C: PostArchiverConnection> PostArchiverManager<C> {
    /// 按主鍵取得 Collection
    pub fn get_collection(&self, id: CollectionId) -> Result<Option<Collection>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM collections WHERE id = ?")?;
        stmt.query_row([id], Collection::from_row).optional()
    }

    /// 按 source 查詢 CollectionId
    pub fn find_collection_by_source(&self, source: &str) -> Result<Option<CollectionId>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT id FROM collections WHERE source = ?")?;
        stmt.query_row([source], |row| row.get(0)).optional()
    }

    /// 帶篩選/分頁的 Collection 列表
    pub fn list_collections(&self, filter: CollectionFilter) -> Result<PageResult<Collection>, rusqlite::Error> { ... }

    /// 取得某 Collection 內的所有 Post（完整實體）
    pub fn list_collection_posts(&self, collection: CollectionId) -> Result<Vec<Post>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "SELECT posts.* FROM posts \
             INNER JOIN collection_posts ON collection_posts.post = posts.id \
             WHERE collection_posts.collection = ?"
        )?;
        let rows = stmt.query_map([collection], Post::from_row)?;
        rows.collect()
    }
}
```

### FileMeta 查詢

```rust
// src/query/file_meta.rs

impl<C: PostArchiverConnection> PostArchiverManager<C> {
    /// 按主鍵取得 FileMeta
    pub fn get_file_meta(&self, id: FileMetaId) -> Result<Option<FileMeta>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM file_metas WHERE id = ?")?;
        stmt.query_row([id], FileMeta::from_row).optional()
    }

    /// 按 post + filename 查詢 FileMetaId
    pub fn find_file_meta(
        &self,
        post: PostId,
        filename: &str,
    ) -> Result<Option<FileMetaId>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "SELECT id FROM file_metas WHERE post = ? AND filename = ?"
        )?;
        stmt.query_row(params![post, filename], |row| row.get(0)).optional()
    }
}
```     

---

## `list_xxx` 動態 SQL 建構

帶篩選條件的 `list` 方法需要動態組裝 WHERE / ORDER BY / LIMIT。
以 `list_posts` 為範例展示完整實現模式，其他實體依此類推：

```rust
pub fn list_posts(&self, filter: PostFilter) -> Result<PageResult<Post>, rusqlite::Error> {
    use rusqlite::types::ToSql;

    let mut wheres: Vec<&str> = Vec::new();
    let mut params: Vec<&dyn ToSql> = Vec::new();

    if let Some(ref platform) = filter.platform {
        wheres.push("platform = ?");
        params.push(platform);
    }

    if let Some(ref kw) = filter.title_contains {
        wheres.push("title LIKE '%' || ? || '%'");
        params.push(kw);
    }

    let where_clause = if wheres.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", wheres.join(" AND "))
    };

    let order_col = match filter.sort {
        PostSort::Updated => "updated",
        PostSort::Published => "published",
        PostSort::Title => "title",
        PostSort::Id => "id",
    };

    // Count total
    let count_sql = format!("SELECT COUNT(*) FROM posts {where_clause}");
    let total: u64 = self.conn().query_row(&count_sql, params.as_slice(), |row| row.get(0))?;

    // Fetch page
    let query_sql = format!(
        "SELECT * FROM posts {where_clause} ORDER BY {order_col} {} LIMIT ? OFFSET ?",
        filter.sort_dir.as_sql()
    );
    params.push(&filter.pagination.limit);
    params.push(&filter.pagination.offset);

    let mut stmt = self.conn().prepare(&query_sql)?;
    let rows = stmt.query_map(params.as_slice(), Post::from_row)?;
    let items = rows.collect::<Result<Vec<_>, _>>()?;

    Ok(PageResult { items, total })
}
```

---

## 與 `Binded::value()` 的差異對照

| 場景 | 使用方式 | 說明 |
|------|---------|------|
| 已知 ID 必定存在（剛 insert） | `manager.bind(id).value()` | 返回 `Result<T>`，ID 不存在即報錯 |
| ID 可能不存在 | `manager.get_post(id)` | 返回 `Result<Option<T>>`，安全處理 |
| 按業務鍵查找 | `manager.find_post_by_source(src)` | 返回 `Result<Option<PostId>>` |
| 列表 + 分頁 | `manager.list_posts(filter)` | 返回 `Result<PageResult<Post>>` |
| 取完整關聯 | `manager.get_post_with_relations(id)` | 一次取得 Post + Authors + Tags + Files + Collections |

---

## 目錄結構

```
src/
  query/
    mod.rs          ← Pagination, SortDir, PageResult 公共類型 + re-exports
    post.rs         ← get_post, find_post_by_source, list_posts, get_post_with_relations
                       list_post_authors, list_post_tags, list_post_files, list_post_collections
    author.rs       ← get_author, find_author_by_alias, list_authors
                       list_author_aliases, list_author_posts
    tag.rs          ← get_tag, find_tag, list_tags, list_tag_posts
    platform.rs     ← get_platform, find_platform, list_platforms
                       list_platform_posts, list_platform_tags
    collection.rs   ← get_collection, find_collection_by_source, list_collections
                       list_collection_posts
    file_meta.rs    ← get_file_meta, find_file_meta
```

---

## 使用範例

```rust
use post_archiver::manager::PostArchiverManager;
use post_archiver::query::{PostFilter, PostSort, SortDir, Pagination};

let manager = PostArchiverManager::open_or_create("./archive")?;

// 簡單取得
let post = manager.get_post(PostId::new(1))?;

// 按業務鍵查詢
if let Some(id) = manager.find_post_by_source("https://example.com/post/123")? {
    let post = manager.bind(id).value()?;
    println!("Found: {}", post.title);
}

// 帶篩選的分頁查詢
let result = manager.list_posts(PostFilter {
    platform: Some(PlatformId::new(1)),
    title_contains: Some("Rust".to_string()),
    pagination: Pagination::new(0, 20),
    sort: PostSort::Updated,
    sort_dir: SortDir::Desc,
    ..Default::default()
})?;
println!("Total: {}, This page: {}", result.total, result.items.len());

// 複合查詢：一次取得 Post 及所有關聯
let full = manager.get_post_with_relations(PostId::new(1))?;
if let Some(full) = full {
    println!("Post: {}", full.post.title);
    println!("Authors: {:?}", full.authors);
    println!("Tags: {:?}", full.tags);
    println!("Files: {}", full.files.len());
}

// Author 查詢
let author_id = manager.find_author_by_alias("octocat", PlatformId::new(1))?;
let authors = manager.list_authors(AuthorFilter::default())?;

// 跨實體查詢
let author_posts = manager.list_author_posts(AuthorId::new(1))?;
let tag_posts = manager.list_tag_posts(TagId::new(5))?;
let collection_posts = manager.list_collection_posts(CollectionId::new(2))?;
```

---

## 實作步驟

1. **建立 `src/query/mod.rs`**
   - 定義 `Pagination`、`SortDir`、`PageResult` 公共類型
   - 宣告並 re-export 子模塊
   - 在 `src/lib.rs` 中加入 `#[cfg(feature = "utils")] pub mod query;`

2. **實作 `query/post.rs`**
   - `get_post`、`find_post_by_source`
   - `list_posts`（含 `PostFilter`、`PostSort`、動態 SQL）
   - `get_post_with_relations`（含 `PostWithRelations`）
   - 跨實體：`list_post_authors`、`list_post_tags`、`list_post_files`、`list_post_collections`

3. **實作 `query/author.rs`**
   - `get_author`、`find_author_by_alias`
   - `list_authors`（含 `AuthorFilter`、`AuthorSort`）
   - `list_author_aliases`、`list_author_posts`

4. **實作 `query/tag.rs`**
   - `get_tag`、`find_tag`
   - `list_tags`（含 `TagFilter`）
   - `list_tag_posts`

5. **實作 `query/platform.rs`**
   - `get_platform`、`find_platform`、`list_platforms`
   - `list_platform_posts`、`list_platform_tags`

6. **實作 `query/collection.rs`**
   - `get_collection`、`find_collection_by_source`
   - `list_collections`（含 `CollectionFilter`）
   - `list_collection_posts`

7. **實作 `query/file_meta.rs`**
   - `get_file_meta`、`find_file_meta`

8. **替換 `tests/helpers.rs`**
   - 將 `helpers.rs` 中所有裸 SQL 查詢函數（`get_post`、`find_post`、`list_posts`、`list_post_authors` 等）改為呼叫 query 模塊方法
   - 僅保留 `add_xxx` 和關聯寫入的 helper（這些屬於 insert 邏輯，由 importer 或測試專用）
   - 確保所有既有測試通過

9. **更新 importer**
   - importer 內的 `find` 查詢（如 `find_post_by_source`、`find_author_by_alias`）改為呼叫 query 模塊同名方法，避免重複 SQL