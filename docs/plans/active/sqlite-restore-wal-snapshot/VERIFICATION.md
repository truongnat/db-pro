# Verification — SQLite Restore WAL Snapshot

- `cargo test -p db-pro-infrastructure --lib sqlite_backup::tests`: 9 passed, 0 failed.
- `cargo check -p db-pro-infrastructure`: passed.
- `cargo fmt --all --check`: passed.
- `git diff --check`: passed.
- Regression keeps a WAL-mode source writer open and verifies restore includes a committed row before checkpoint.
