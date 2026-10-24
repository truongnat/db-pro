---
schema_version: 1
id: "MEM-SRC-cd985a8d5d5407f4"
title: "Source mirror: docs/plans/active/issue-496-497-fk-dropdown/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-496-497-fk-dropdown/VERIFICATION.md"
source_sha256: "cd985a8d5d5407f4ce40d94a3c6de9802ce94c06ea4fa624f73e484b18262734"
mirror_body_sha256: "cd985a8d5d5407f4ce40d94a3c6de9802ce94c06ea4fa624f73e484b18262734"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Verification — #496 + #497

- `cargo check -p db-pro-app`: clean.
- `cargo test -p db-pro-app --lib`: 591 passed, 0 failed.
  - `app::table_fk_lookup_tests::composite_foreign_key_gets_no_value_dropdown` — new (1→2 and 2→1 shapes).
  - `app::table_fk_lookup_tests::foreign_key_lookup_sql_is_bounded_and_ordered` — new:
    `ORDER BY "id"` + `LIMIT 500`.
  - `app::table_fk_lookup_tests::foreign_key_lookup_reports_truncated_options` — new.
  - `app::table_fk_lookup_tests::foreign_key_lookup_edits_and_opens_referenced_row` — existing
    single-column path still dispatches lookup + navigates the referenced row.
