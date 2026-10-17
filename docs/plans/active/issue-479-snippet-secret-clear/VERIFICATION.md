# #479 Verification

Commands run:

- `cargo test -p db-pro-app --lib connection::` — 31 passed, 0 failed.
  New tests (`mapper_tests.rs`):
  - `apply_snippet_clears_cloud_snippet_after_success` — imports
    `postgresql://alice:s3cret@db.example.com:5432/appdb?sslmode=require`;
    asserts all fields populated (incl. `password == "s3cret"`,
    `ssl_mode == UiSslMode::Require`) and `cloud_snippet == ""`.
  - `apply_snippet_keeps_cloud_snippet_on_parse_error` — invalid URI keeps
    the original text so the user can correct it.
- `cargo test -p db-pro-app connection` — 68 passed, 0 failed.
- `cargo check -p db-pro-app` — clean.
