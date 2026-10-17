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
