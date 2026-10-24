---
schema_version: 1
id: "MEM-PLAN-issue-483-pgpass-leak-1cc3d8fba78d"
title: "Plan archive: Issue 483 Pgpass Leak"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-483-pgpass-leak]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "1cc3d8fba78d895419b1eb4ededf2403179bac161e8fd37bae3a64eb86dbe6b0"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-483-pgpass-leak/PLAN.md"
  - "docs/plans/active/issue-483-pgpass-leak/FINDINGS.md"
  - "docs/plans/active/issue-483-pgpass-leak/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-483-pgpass-leak/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# #483 — failed pgpass write leaves credential file in temp dir

## Goal
`PgPassFile::create` (security): when `write_all` fails, the partially written
credential file must be removed. Previously `create` returned `Err` before the `Self`
guard existed, so `Drop` never ran and a file containing the DB password (0600 but
present) stayed in `temp_dir` until reboot.

## Scope
- `crates/infrastructure/src/backup/pg_dump.rs`: extract the write into
  `write_pgpass(sink, path, contents)`; on error it consumes/drops the sink (closes the
  handle, required for unlink on Windows) and `let _ = std::fs::remove_file(path)`
  before returning `DbError::Internal`.
- `create` calls `write_pgpass(file, &path, …)?` — the error path now cleans up; the
  success path is unchanged (guard drops → file removed).
- Tests in `crates/infrastructure/src/backup/tests/pg_dump_tests.rs` (mod already
  `#[cfg(all(test, unix))]`): inject a failing `Write` sink to prove the file is gone
  on error; positive-path companion test.

## Non-goals
- Open-failure path already can't leak (create_new either created the file and opened
  it, or failed); only the post-open write failure was broken.
- Non-unix permission story unchanged (documented residual risk stays as-is).

## Acceptance
- write failure removes the temp file; success leaves it for the guard to clean on drop.


## Source: `FINDINGS.md`

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


## Source: `VERIFICATION.md`

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

