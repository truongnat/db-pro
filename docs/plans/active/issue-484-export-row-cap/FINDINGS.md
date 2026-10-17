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
