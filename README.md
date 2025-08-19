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

## Install

### For Rust
```sh
cargo add post-archiver
```

### For TypeScript
```sh
npm add -D post-archiver
```

Import you need types
```ts
import type { Post } from "post-archiver"
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
