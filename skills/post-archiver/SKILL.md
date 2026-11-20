---
name: post-archiver
description: Index of the PostArchiver programs (importers, viewer, editor, maintenance CLI). Use for archiving posts (Fanbox, Pixiv, Patreon, fanbox-dl folders) or anything involving PostArchiver or a `post-archiver.db` archive.
---

# PostArchiver

A simple unified post archive: importers write posts into one archive directory, the other tools read it.
Each program's usage is in its README and `--help` (prefer `--help`, READMEs can lag behind).

# Tools

## post-archiver
Maintenance CLI: `info`, `migrate`, `vacuum`. Every program is pinned to one database version,
so after `migrate`, update every program that opens the archive.
https://github.com/xiao-e-yun/PostArchiver (`tools-v*` releases)

## post-archiver-viewer
Read-only web UI for browsing an archive.
https://github.com/xiao-e-yun/PostArchiverViewer

## post-archiver-editor
Web UI that edits an archive.
https://github.com/xiao-e-yun/PostArchiverEditor

# Importers

- fanbox - https://github.com/xiao-e-yun/FanboxArchive
- pixiv - https://github.com/xiao-e-yun/PixivArchive
- patreon - https://github.com/xiao-e-yun/PatreonArchive
- fanbox-dl - https://github.com/xiao-e-yun/FanboxDLArchive

See [Start.md](Start.md) to set up an archive.
