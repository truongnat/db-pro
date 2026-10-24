---
schema_version: 1
id: "MEM-SRC-9f4871a851f6fa05"
title: "Source mirror: docs/plans/active/postgres-autocommit-search-path/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/postgres-autocommit-search-path/CHECKLIST.md"
source_sha256: "9f4871a851f6fa0526cb389e18aa35ce85d6a7f196e4414d66cd0710a94c56da"
mirror_body_sha256: "9f4871a851f6fa0526cb389e18aa35ce85d6a7f196e4414d66cd0710a94c56da"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Postgres autocommit search_path — CHECKLIST

- [x] Reproduce `SET LOCAL` expiry on live PostgreSQL (autocommit)
- [x] Reproduce through the connector: new regression test failed before the fix with
      `relation "scoped_values" does not exist`
- [x] Scope push/pop by connection transaction state
- [x] Preserve #456 with discard-on-abandon + `close_on_drop`
- [x] Unit tests for abandoned vs restored leases
- [x] Integration tests: autocommit run, held transaction, timeout leak
- [x] `cargo +nightly fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace` (2243 passed / 0 failed / 65 ignored)
- [x] Clean-code scan scoped to the changed files — no new findings (one inherited `execute_batch`
      `HUGE_FUNCTION` error also present at HEAD)
- [ ] UAT: tenant picker + unqualified query in the native app against a real tenant database
