# Checklist — Safe SQLite Sample Seeding

- [x] Remove the shared `/tmp` fixture path from new startup seeding.
- [x] No-overwrite create and validate existing fixtures by app id and marker.
- [x] Reject symlinks and preserve unrecognized existing files.
- [x] Seed the sample connection read-only and migrate the legacy sample entry.
- [x] Add tests for idempotence, arbitrary existing files, and symlink safety.
- [x] Native test suite, native package check, formatting, and diff validation pass.
- [x] Rebase, commit, push, and open draft PR #529 linked to #492.
