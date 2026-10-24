---
schema_version: 1
id: "MEM-SRC-ea6b6798075ba81f"
title: "Source mirror: docs/plans/active/issue-482-pg-blocking-pids/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-482-pg-blocking-pids/VERIFICATION.md"
source_sha256: "ea6b6798075ba81f7fc451995f6e874775bd44d29fd77dba884a3167197580a4"
mirror_body_sha256: "ea6b6798075ba81f7fc451995f6e874775bd44d29fd77dba884a3167197580a4"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Verification — #482

- `cargo check -p db-pro-infrastructure` — clean (with the rest of the touched
  workspace crates).
- `cargo test -p db-pro-infrastructure monitoring` — `monitoring_tests` pass.
- Not run against a live PostgreSQL: no runtime fixture available; correctness
  rests on `pg_blocking_pids` being the documented server-side answer
  (https://www.postgresql.org/docs/current/functions-info.html) plus the
  SQL-shape contract tests.
