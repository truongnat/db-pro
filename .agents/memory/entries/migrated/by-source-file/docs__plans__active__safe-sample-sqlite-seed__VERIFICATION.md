---
schema_version: 1
id: "MEM-SRC-ee613144509165e5"
title: "Source mirror: docs/plans/active/safe-sample-sqlite-seed/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/safe-sample-sqlite-seed/VERIFICATION.md"
source_sha256: "ee613144509165e56b8f8546e6e8857bdd42ecb5cf6afa62917a09f06a703463"
mirror_body_sha256: "ee613144509165e56b8f8546e6e8857bdd42ecb5cf6afa62917a09f06a703463"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Verification — Safe SQLite Sample Seeding

- `cargo test -p db-pro-native --bin db-pro-native sample_seed -- --nocapture`: 3 passed, 0 failed.
- `cargo check -p db-pro-native`: passed.
- `cargo fmt --all --check`: passed.
- `git diff --check`: passed.
- Focused regressions confirm unknown pre-existing bytes and a valid unmarked SQLite database are preserved, fixture reuse leaves its bytes unchanged, sample config is read-only, and symlinks are rejected without modifying their targets.
- Fixture creation opens its reserved file with `SQLITE_OPEN_NOFOLLOW` and without `SQLITE_OPEN_CREATE`, so SQLite refuses a symlink swapped into the reserved path.
- Legacy connection migration now additionally requires the SQLite driver.
