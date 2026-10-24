---
schema_version: 1
id: "MEM-PLAN-issue-484-export-row-cap-6790b5820758"
title: "Plan archive: Issue 484 Export Row Cap"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-484-export-row-cap]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "6790b58207581d9542d781c9bfdc776bffad6ae9579dbf2203bfa52b2e95b4b8"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-484-export-row-cap/PLAN.md"
  - "docs/plans/active/issue-484-export-row-cap/FINDINGS.md"
  - "docs/plans/active/issue-484-export-row-cap/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-484-export-row-cap/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# #484 — Hard cap on exported rows

## Goal
`SELECT * FROM huge_table` export must fail with an explicit error instead of
materializing unbounded per-row buffers and OOMing the process.

## Scope
- `crates/core/src/application/export_formats.rs`: new
  `MAX_EXPORT_ROWS = 500_000` + `ensure_within_export_cap(result)` returning
  `DbError::Validation` naming the cap and suggesting a `LIMIT` clause.
- Guard called at the top of `render_csv` / `render_json` / `render_excel`
  (defense in depth at the point row data is pulled).
- `export_service.rs`: `execute_for_export` enforces the cap right after the
  connector returns, before `result.validate()` walks every row.
- Tests in `application/tests/export_service_tests.rs`.

## Non-goals
- No streaming export architecture (issue's boring option).
- No new SELECT-lacks-LIMIT linter — none exists in safety/policy code, so the
  suggestion lives in the error message text only.

## Acceptance
- Result > 500k rows → `DbError::Validation` naming the cap + LIMIT hint.
- ≤ cap exports unchanged.
- `cargo test -p db-pro-core` + `cargo check -p db-pro-core`.


## Source: `FINDINGS.md`

# #484 Findings

## Evidence (source)
- `export_service.rs` `execute_for_export`: `connector.query` returns the whole
  `QueryResult` (`rows: Vec<Row>` fully materialized by the connector layer —
  out of scope to change without streaming).
- `render_csv` allocates a `Vec<String>` per row (fields) and writes into a
  growing `csv::Writer` buffer; `render_json` collects a `Vec<Map>` of all rows
  then `to_vec_pretty`s it; `render_excel` builds the entire workbook in memory.
  No row bound anywhere → unbounded intermediate + final buffers.

## Failure scenario
`SELECT * FROM huge_table` on export → connector buffers all rows, then
renderers double-buffer → process OOM for sufficiently large tables.

## Fix seam
`execute_for_export` is the single entry every export format funnels through —
cap enforced there first. Renderers also call `ensure_within_export_cap` at
entry so direct/internal callers can't bypass it.

## Notes
- `QueryResult::validate()` iterates all rows (O(n)); the cap check runs before
  it so an oversized result is rejected cheaply.
- No existing "SELECT without LIMIT" warning exists in `domain/safety.rs` /
  `sql_policy`; per assignment the LIMIT hint is embedded in the error message.
- `ConnectionConfig.max_rows` is a display/grid setting, unrelated to exports.


## Source: `VERIFICATION.md`

# #484 Verification

## Pass-after
- `cargo check -p db-pro-core` → Finished OK.
- `cargo test -p db-pro-core export` → 21/21 PASS, including new:
  - `export_cap_allows_exactly_max_rows` (boundary, 500_000 ok)
  - `export_cap_rejects_one_row_over_max` (DbError::Validation, names cap + LIMIT)
  - `csv_render_rejects_result_over_row_cap` (renderer-level defense)
  - `export_csv_rejects_result_over_row_cap` (service-level, full pipeline)
- `cargo test -p db-pro-core` → 487 passed / 0 failed.

## Not done (declared honestly)
- Query itself still fetches all rows from the driver (no streaming) — the cap
  bounds what we buffer/write, per the issue's chosen option.
- No SELECT-lacks-LIMIT linter (doesn't exist; LIMIT hint is in the error text).
- Full `cargo test --workspace` not run; no UI screenshot.
- Issue #484 left OPEN pending owner decision.

