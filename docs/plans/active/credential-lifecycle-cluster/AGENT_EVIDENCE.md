# Agent Evidence — Credential Lifecycle Cluster

- Baseline: `beaa1882a6253d902748c27fba53b99d9969b635` (`origin/main` at triage).
- Issue #483 confirmed in `PgPassFile::create`: `write_all` could fail before `PgPassFile { path }` existed, leaving a partial temporary credential file.
- Fix is limited to `crates/infrastructure/src/backup/pg_dump.rs` and its tests. The cleanup guard is constructed before writing; on error the writer is explicitly dropped before the error unwinds and the guard removes the file.
- Regression test uses a real file, writes a partial prefix, injects a subsequent write failure, and asserts the file is removed.
- Issue #479 was investigated and is deferred: the URI remains visible in the plaintext UI field after import, but the claimed draft persistence/backend/log exposure was not substantiated at baseline. It requires a separate UI behavior change and its own UI verification.
- Exact test/check results and the Clippy baseline blocker are recorded in `VERIFICATION.md`.
- No live provider behavior was changed or claimed.
