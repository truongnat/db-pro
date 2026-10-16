# Plan — Safe Transfer Cleanup and Empty CSV Headers

## Objective

Fix related export/transfer issues #436 and #437 without deleting a prior successful export or publishing a schema-less empty CSV/TSV.

## Target Branch

`fix/export-transfer-preserves-destination`

## Severity

P1 data-loss regression (#436) and P2 empty delimited export schema loss (#437).

## Scope

- Restrict cleanup of CSV/TSV, JSONL, and Excel targets to the current run's temporary output.
- Preserve an existing final destination on cancel/error.
- Write delimited headers during finish when no rows were written.
- Keep JSONL's standard record-only representation: an empty export has zero lines; it has no header-record convention.
