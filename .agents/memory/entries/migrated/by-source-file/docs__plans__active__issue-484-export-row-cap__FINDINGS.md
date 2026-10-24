---
schema_version: 1
id: "MEM-SRC-c92c46a0365aa148"
title: "Source mirror: docs/plans/active/issue-484-export-row-cap/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-484-export-row-cap/FINDINGS.md"
source_sha256: "c92c46a0365aa1484854422b730293c29b8f72c6b13cb586c02153a97acd4073"
mirror_body_sha256: "c92c46a0365aa1484854422b730293c29b8f72c6b13cb586c02153a97acd4073"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
