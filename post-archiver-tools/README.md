# post-archiver-tools

Maintenance CLI for post-archiver archives. The binary is called `post-archiver`.

```sh
post-archiver info <archive>               # version and statistics
post-archiver migrate <archive> [--yes]    # upgrade in place to the latest version
post-archiver vacuum <archive> [--dry-run] # remove unreferenced files, then VACUUM the database
```

`<archive>` defaults to `./archive`. Every command ends with a summary and exits non-zero on failure.

### info

Prints the database version, the latest version this tool can migrate to, and row counts.
Tools are pinned to a database version, so an older archive is not an error: only upgrade when the
tools you use support the new version.

### migrate

Upgrades the archive in place, step by step. **This cannot be undone**: back up the database first,
e.g. `sqlite3 <archive>/post-archiver.db ".backup post-archiver.db.bak"`, or migrate a hard-linked
copy (`cp -al <archive> <new-archive>`) to keep the old archive untouched.

It asks for confirmation unless `--yes` is given. Without a terminal (scripts, agents) it refuses to
run unless `--yes` is given. Each database step runs in its own transaction.

Supported steps:
- [x] v0.1 -> v0.2 -> v0.3 (file layout)
- [x] v0.3 -> v0.4 (database)
- [x] v0.4 -> v0.5
- [x] v0.5 -> v0.6: merges duplicated file metas, removes orphan rows, relaxes tag / alias uniqueness

### vacuum

Only works on archives at the latest version. In the post directories (`<chunk>/<index>/`) it:
- removes files without a matching `file_metas` row,
- removes post and chunk directories left empty,
- keeps and reports anything unexpected (nested directories, symlinks, non-numeric names),
- keeps and reports `file_metas` rows whose file is missing, since posts reference them.

Then it runs `VACUUM` on the database. Every removed path is listed; use `--dry-run` to preview.
