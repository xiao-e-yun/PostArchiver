# 完整重寫 manager 模塊

## 目標
- 以破壞性更改的方式重寫 manager 模塊，提升代碼質量和可維護性。
- 不必確保向後兼容性，專注於重構和優化現有功能。
- 引入新的設計模式和架構，以提高模塊的靈活性和可擴展性。
- 你不必考慮與其他模塊的兼容性，專注於 manager 模塊的重寫和優化。

## 現存問題分析

### 1. 方法過度集中
目前 `PostArchiverManager<T>` 結構體透過多個 `impl` 塊散佈在數個文件（`post.rs`、`author.rs`、`tag.rs` 等），累積了大量 CRUD 方法（`list_posts`、`find_post`、`get_post`、`add_post`、`remove_post` …）。這使得 Manager 成為一個「上帝物件」，違反單一職責原則。

### 2. 查詢與寫入混雜
讀取（query）與寫入（mutation）操作均直接掛載在同一個 struct 上，缺乏明確的分層設計。

### 3. importer 進一步膨脹 manager
`importer` 模塊也透過 `impl<T> PostArchiverManager<T>` 繼續向 manager 添加方法（如 `import_post`），造成職責更加模糊。

### 4. Cache 與 Manager 緊耦合
`PostArchiverManagerCache`（`DashMap` for tags、collections、platforms）直接內嵌於 `PostArchiverManager`，難以單獨測試或替換。

---

## 模塊定義

### manager 模塊（本次重寫的目標）
系統核心層，職責精確限縮為：
- 資料庫連線管理（`open`、`create`、`transaction`）
- 透過 `bind(id)` 取得 `Binded<'_, Id>` 以操作已知 ID 的實體（修改、刪除、關聯管理）

> `list`、`get`、`find` 由 **query 模塊**實現（暫不處理）。
> `insert` 相關邏輯由 **importer 模塊**涵蓋，manager 不重複提供。

### importer 模塊
高層導入邏輯，負責插入新實體並管理關聯，不再直接 `impl PostArchiverManager`。

### query 模塊
提供 `list`、`get`、`find` 等讀取操作，以及跨實體的複合查詢（如：一次拉取 Post 及其所有關聯 Author、Tag、FileMeta）。

---

## 重寫後的 manager 模塊設計

### 核心：`Binded<'a, T>`

`Binded<'a, Id>` 代表「已綁定特定實體 ID 的操作上下文」，透過 `PostArchiverManager::bind(id)` 取得。
其職責限縮為：**修改或刪除該實體本身**，以及**管理該實體的關聯關係**。

由於 `PostId`、`AuthorId` 等 ID 類型本身即帶有型別資訊，`bind()` 可直接從引數推導，**無需 turbofish**：

```rust
/// 標記某個 ID 類型對應哪個實體的 trait
pub trait BindableId: Sized {}
impl BindableId for PostId {}
impl BindableId for AuthorId {}
// …其他 ID 類型

pub struct Binded<'a, Id: BindableId> {
    manager: &'a PostArchiverManager,
    id: Id,
}

impl PostArchiverManager {
    // 直接傳 post_id: PostId，Rust 自動推導為 Binded<'_, PostId>
    pub fn bind<Id: BindableId>(&self, id: Id) -> Binded<'_, Id> {
        Binded { manager: self, id }
    }
}
```

每個 ID 類型分別對 `Binded<'_, Id>` 實現修改與關聯操作：

```rust
// src/manager/post.rs
impl<'a> Binded<'a, PostId> {
    // 修改實體本身
    pub fn update(&self, data: UpdatePost) -> Result<(), rusqlite::Error> { ... }
    pub fn delete(&self) -> Result<(), rusqlite::Error> { ... }

    // 關聯操作：Authors
    pub fn list_authors(&self) -> Result<Vec<AuthorId>, rusqlite::Error> { ... }
    pub fn add_authors(&self, authors: &[AuthorId]) -> Result<(), rusqlite::Error> { ... }
    pub fn remove_authors(&self, authors: &[AuthorId]) -> Result<(), rusqlite::Error> { ... }

    // 關聯操作：Tags
    pub fn list_tags(&self) -> Result<Vec<TagId>, rusqlite::Error> { ... }
    pub fn add_tags(&self, tags: &[TagId]) -> Result<(), rusqlite::Error> { ... }
    pub fn remove_tags(&self, tags: &[TagId]) -> Result<(), rusqlite::Error> { ... }

    // 關聯操作：FileMeta
    pub fn list_files(&self) -> Result<Vec<FileMetaId>, rusqlite::Error> { ... }
    pub fn add_files(&self, files: &[FileMetaId]) -> Result<(), rusqlite::Error> { ... }
    pub fn remove_files(&self, files: &[FileMetaId]) -> Result<(), rusqlite::Error> { ... }
}

// 其他 ID 類型類似：Binded<'_, AuthorId>、Binded<'_, TagId>、Binded<'_, PlatformId> …
```

### 使用範例

```rust
let mut manager = PostArchiverManager::open_or_create("./archive")?;
let tx = manager.transaction()?;

// 插入由 importer 模塊負責，此處假設已透過 importer 取得 ID
let post_id: PostId = /* importer */ todo!();
let author_id: AuthorId = /* importer */ todo!();
let tag_id: TagId = /* importer */ todo!();

// 關聯與修改：直接傳 ID，型別自動推導，無需 turbofish
tx.bind(post_id).add_authors(&[author_id])?;
tx.bind(post_id).add_tags(&[tag_id])?;
tx.bind(post_id).update(UpdatePost { title: Some("Updated".to_string()), ..Default::default() })?;
tx.bind(post_id).delete()?;

// 列出某篇文章的關聯（關聯 list 仍在 Binded 上）
let authors: Vec<AuthorId> = tx.bind(post_id).list_authors()?;

tx.commit()?;
```

### `PostArchiverManager` 職責

重寫後 `PostArchiverManager` 只負責**連線管理**與**取得 `Binded` 上下文**：
- `list`、`get`、`find` → 由 **query 模塊**提供（暫不處理）
- `insert` → 由 **importer 模塊**提供
- `update`、`delete`、關聯操作 → 透過 `bind(id)` 取得 `Binded` 後操作

```rust
pub struct PostArchiverManager<C = Connection> {
    pub path: PathBuf,
    conn: C,
}

impl PostArchiverManager {
    pub fn create(path: impl AsRef<Path>) -> Result<Self, rusqlite::Error>;
    pub fn open(path: impl AsRef<Path>) -> Result<Option<Self>, rusqlite::Error>;
    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self, rusqlite::Error>;
    pub fn open_in_memory() -> Result<Self, rusqlite::Error>;
    pub fn transaction(&mut self) -> Result<PostArchiverManager<Transaction<'_>>, rusqlite::Error>;
    // ID 型別有型，Rust 自動推導，無需 turbofish
    pub fn bind<Id: BindableId>(&self, id: Id) -> Binded<'_, Id>;
}
```

Cache（`DashMap` for tags/collections/platforms）移出 Manager 結構體，
改由 importer 模塊在需要時自行持有，或以獨立的 `ImportContext` 結構傳入。

---

## 目錄結構規劃

```
src/
  manager/
    mod.rs          ← PostArchiverManager + bind() + transaction()
    binded.rs       ← Binded<'a, Id> 定義 + BindableId trait + PostArchiverConnection trait
    post.rs         ← impl Binded<'_, PostId>
    author.rs       ← impl Binded<'_, AuthorId>
    tag.rs          ← impl Binded<'_, TagId>
    platform.rs     ← impl Binded<'_, PlatformId>
    collection.rs   ← impl Binded<'_, CollectionId>
    file_meta.rs    ← impl Binded<'_, FileMetaId>
  importer/
    mod.rs
    context.rs      ← ImportContext（持有 cache，調用 manager.bind()）
    post.rs
    author.rs
    ...
  query/
    mod.rs
    post.rs         ← 跨實體查詢（Post + Authors + Tags + Files）
    author.rs
```

---

## 實作步驟

1. **精簡 `PostArchiverManager`**
   - 移除 `cache` 欄位（`PostArchiverManagerCache`）
   - 移除所有業務方法（`list_*`、`get_*`、`find_*`、`add_*`、`remove_*` 等），只保留連線管理（`open`、`create`、`transaction`）
   - 新增 `bind(id)` 方法

2. **建立 `binded.rs`：`BindableId` trait 與 `Binded<'a, Id>` struct**
   - 定義 `BindableId` trait，為 `PostId`、`AuthorId`、`TagId`、`PlatformId`、`CollectionId`、`FileMetaId` 各自實作
   - 定義 `Binded<'a, Id: BindableId>` struct，持有 manager 引用與 id

3. **為各 ID 類型實作 `Binded` 操作**
   - 在 `manager/post.rs` 實作 `impl Binded<'_, PostId>`：`update`、`delete`，以及與 Author、Tag、FileMeta 的關聯操作（`list`、`add`、`remove`）
   - 在 `manager/author.rs` 實作 `impl Binded<'_, AuthorId>`：`update`、`delete`，以及 Alias、Platform 的關聯操作
   - 其餘 `TagId`、`PlatformId`、`CollectionId`、`FileMetaId` 依各自關聯表實作對應操作

4. **重寫 importer 模塊**
   - 建立 `importer::ImportContext`，持有 cache（tags/collections/platforms 的 `DashMap`）與 manager 引用
   - 將所有 `import_*` 方法（原本散佈於 `impl PostArchiverManager`）遷移至 `ImportContext`
   - 插入操作直接執行 SQL，關聯操作改用 `self.manager.bind(id).add_xxx(...)`

5. **更新測試**
   - 調整 `src/tests/` 下的測試，以 `ImportContext` 替代原 manager 方法呼叫
   - 驗證 `bind(id)` 的 update / delete / 關聯操作行為正確