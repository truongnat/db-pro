# #490 — Checklist

- [x] `parse_postgres_csv_line`: id = `csvlog:{hash(raw line)}`
- [x] `read_csvlog_page`: per-line file byte offset appended to id
- [x] Removed now-unused `pid`/`session_line` locals
- [x] Test: shared prefix / different tail → distinct ids
- [x] Test: identical raw line → stable id
- [x] `cargo check -p db-pro-core` — clean
- [x] `cargo test -p db-pro-core --lib domain::audit` — 8 passed, 0 failed
