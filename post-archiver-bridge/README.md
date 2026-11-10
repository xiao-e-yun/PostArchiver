# PostArchiverBridge

Migrate old version

Upgrades an archive created by any older version to the current one, step by step.

```sh
post-archiver-bridge <source> <target>
```

`<target>` must not exist; the source is copied there first. Pass the same path twice to
upgrade in place (you will be asked to confirm).

### Support
- [x] v0.1 -> v0.2 -> v0.3 (file layout)
- [x] v0.3 -> v0.4 (database)
- [x] v0.4 -> v0.5
- [x] v0.5 -> v0.6: merges duplicated file metas, removes orphan rows, relaxes tag / alias uniqueness