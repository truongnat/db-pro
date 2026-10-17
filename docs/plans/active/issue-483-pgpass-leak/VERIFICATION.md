# #483 verification

- `failed_pgpass_write_removes_the_partial_file` — injected `Write` impl that always
  errors on `write()`; asserts `DbError::Internal` is returned and the pre-created
  target file no longer exists.
- `successful_pgpass_write_leaves_file_for_guard` — real file round-trip through
  `write_pgpass`; contents verified, fixture removed.
- Existing `pgpass_file_is_owner_only_and_removed_on_drop` still covers the guard's
  drop-removal path.
- `cargo check -p db-pro-infrastructure` — clean.
- `cargo test -p db-pro-infrastructure --lib` — 167 passed / 0 failed, including both new
  tests above and the pre-existing pgpass drop test.
