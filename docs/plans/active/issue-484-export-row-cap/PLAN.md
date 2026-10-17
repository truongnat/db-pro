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
