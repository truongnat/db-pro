---
schema_version: 1
id: "MEM-SRC-0c206ca64e6d240c"
title: "Source mirror: docs/plans/active/issue-483-pgpass-leak/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-483-pgpass-leak/PLAN.md"
source_sha256: "0c206ca64e6d240c474530053611848b4f369babee46ee44ccc923ee762061aa"
mirror_body_sha256: "0c206ca64e6d240c474530053611848b4f369babee46ee44ccc923ee762061aa"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
