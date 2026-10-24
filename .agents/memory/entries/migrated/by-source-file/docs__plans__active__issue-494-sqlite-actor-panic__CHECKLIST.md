---
schema_version: 1
id: "MEM-SRC-d9ba04214b411b76"
title: "Source mirror: docs/plans/active/issue-494-sqlite-actor-panic/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-494-sqlite-actor-panic/CHECKLIST.md"
source_sha256: "d9ba04214b411b76964ff3755235d564e5b072857a3cdba64e5c4c92b0d42a98"
mirror_body_sha256: "d9ba04214b411b76964ff3755235d564e5b072857a3cdba64e5c4c92b0d42a98"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — issue #494

- [x] All handler calls in `run()` wrapped in `catch_unwind(AssertUnwindSafe)`
- [x] Panic → `Err(DbError::Internal(...panic...))` to the responder
- [x] Transaction handlers mapped to `TransactionFailure` (phase Validation, outcome Unknown)
- [x] Actor loop continues after a panic (loop keeps running; `Shutdown` still honored)
- [x] `#[cfg(test)] SqliteCommand::Poison` drives the panic end-to-end
- [x] Test: `dispatch_guarded_converts_panic_into_internal_error`
- [x] `cargo test -p db-pro-infrastructure --lib sqlite` — 36 passed, 0 failed
- [x] `cargo check -p db-pro-infrastructure` — clean
- [x] Test: `actor_survives_panicking_command_and_keeps_serving`
