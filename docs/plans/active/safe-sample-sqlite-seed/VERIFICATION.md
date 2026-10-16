# Verification — Safe SQLite Sample Seeding

- `cargo test -p db-pro-native --bin db-pro-native sample_seed -- --nocapture`: 3 passed, 0 failed.
- `cargo check -p db-pro-native`: passed.
- `cargo fmt --all --check`: passed.
- `git diff --check`: passed.
- Focused regressions confirm unknown pre-existing bytes and a valid unmarked SQLite database are preserved, fixture reuse leaves its bytes unchanged, sample config is read-only, and symlinks are rejected without modifying their targets.
- Fixture creation opens its reserved file with `SQLITE_OPEN_NOFOLLOW` and without `SQLITE_OPEN_CREATE`, so SQLite refuses a symlink swapped into the reserved path.
- Legacy connection migration now additionally requires the SQLite driver.
