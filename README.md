# PostArchiver
Types for Archiver
A simple unify post archive.

### Input
* [FanboxArchive](https://github.com/xiao-e-yun/FanboxArchive)
* Planing

### Output
* [PostArchiverViewer](https://github.com/xiao-e-yun/PostArchiverViewer)
* Code by your self

### Docs
[Docs](docs/intro.md)

## Install

### For Rust
```sh
cargo add post-archiver
```
use it (check `src/structs.rs` to know more)
```rs
use post_archiver::*;
```

### For TypeScript
```sh
npm add -D post-archiver
```

Import you need types
```ts
import type { ArchivePost } from "post-archiver"
```

## Build 
### For Rust
```sh
cargo build
```
### For TypeScript
```sh
cargo test -F=typescript
node gen-types.mjs
```
You will get files in `bindings`
