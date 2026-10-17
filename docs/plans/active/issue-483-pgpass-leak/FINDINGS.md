# #483 findings

- `PgPassFile::create` opened `temp_dir()/db-pro-pgpass-<uuid>` with `create_new` +
  0600, then `file.write_all(contents)` with `?`. On failure it returned `Err` while
  `Self { path }` was never constructed → `Drop::drop` (the only removal) never ran →
  a possibly-partial credential file persisted.
- Unpredictable name + 0600 mitigate, but the file still contains the plaintext
  password; the issue is a real (if narrow) secret-leak window.
- Fix shape: a free `write_pgpass(mut sink: impl Write, path, contents)` — generic sink
  keeps it unit-testable without needing an unwritable-but-creatable path; dropping the
  sink before `remove_file` is required on Windows and harmless on Unix.
- `OpenOptions::open` failure can't leak: `create_new` either creates and returns the
  handle or fails atomically, and the error branch then holds no file handle.
