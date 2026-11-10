# PostArchiver
Types for Archiver
A simple unify post archive.
[Docs.rs](https://docs.rs/post-archiver/latest/post_archiver/)

### Input
* [FanboxArchive](https://github.com/xiao-e-yun/FanboxArchive)
* [PatreonArchive](https://github.com/xiao-e-yun/PatreonArchive)
* [FanboxDLArchive](https://github.com/xiao-e-yun/FanboxDLArchive)
* [PixivArchive](https://github.com/xiao-e-yun/PixivArchive)

### Output
* [PostArchiverViewer](https://github.com/xiao-e-yun/PostArchiverViewer)
* Code by your self

### Edit
* [PostArchiverEditor](https://github.com/xiao-e-yun/PostArchiverEditor)

## Install

### For Rust
```sh
cargo add post-archiver                     # manager + query (default `utils` feature)
cargo add post-archiver -F importer         # + importer for scrapers
cargo add post-archiver --no-default-features  # data types only, no SQLite
```

### For TypeScript
```sh
npm add -D post-archiver
```

Import you need types
```ts
import type { Post } from "post-archiver"
```

## Upgrading an archive
Archives created by older versions are upgraded with [`post-archiver-bridge`](post-archiver-bridge):
```sh
cargo run --release --manifest-path post-archiver-bridge/Cargo.toml -- <old-archive> <new-archive>
```
Pass the same path twice to upgrade in place. See [CHANGELOG.md](CHANGELOG.md) for what changed.

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
