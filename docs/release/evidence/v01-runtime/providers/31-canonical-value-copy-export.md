# Canonical provider values survive copy and export (#61)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Tree: `main @ bb934ef` (clean at the start of the change); fix commit recorded in `LEDGER.md`
- Issue: **#61** ([Gate 5][C2] Render/copy/export canonical provider values without precision or
  timezone loss) — parent workstream **#24**, parent gate **#21**; the A1/A2 contracts it consumes are
  #51 and #52
- Predecessor in this session: #56 (`docs/release/evidence/v01-runtime/providers/30-pg-temporal-decoder-classes.md`)

## 1. Measured behaviour before the change

Three frontend serialization paths were measured on the tree; each is reached from the result grid
context menu.

| # | Path | Location (before) | Measured defect |
|---|---|---|---|
| D1 | query-view **export to file** (CSV/TSV) | `crates/ui/src/query_view.rs:1363-1383` | cells written raw: a value containing the delimiter, a quote or a newline broke the record layout; `Json`/`Text`/`Bytes` all went through the same `v.clone()` arm |
| D2 | **copy as JSON** (row / selected rows) | `crates/ui/src/result_grid_view.rs:1821-1840` | `Number` was parsed as `i64` and then `f64`, so `9007199254740993` copied as `9007199254740992.0` and `12345678901234567890.12345` as `1.2345678901234567e+19` — the exact digits did not survive the copy |
| D3 | **copy row / rows / with headers** (tab-separated) | `crates/ui/src/result_grid_view.rs:1561-1595,1616-1635` | joined `cell_text()` with a literal `\t` and `\n`, so a cell containing a tab or a newline produced extra fields/rows; `Null` was copied as the literal text `NULL`, indistinguishable from the string `"NULL"` |

The display path (`crate::cell_text`) was **not** a defect and is unchanged: it is shared with the
grid's filter/sort comparison (`crates/ui/src/result_grid.rs:89,108`) and is pinned by
`app::tests::cell_text_keeps_null_and_json_visible`.

## 2. The fix

All three paths now share one escaping rule (`escape_delimited_field`) and one cell renderer
(`format_cell_delimited`), both in `crates/ui/src/result_grid_view.rs`:

| Change | Effect |
|---|---|
| `escape_delimited_field(value, delimiter)` | quotes a field that contains the delimiter, a double quote, `\n` or `\r`, doubling embedded quotes — CSV quoting rules, applied to whatever delimiter is in use |
| `format_cell_delimited(cell, delimiter)` | `Null` → empty field, `Boolean`/`Number` → verbatim (exact digits), `Text`/`Json`/`Bytes` → escaped |
| `format_result_delimited(result, delimiter)` | the header + every row as the export writes them; `query_view::export_result` now calls it instead of re-implementing the join |
| `copied_row_text(...)` | the tab-separated clipboard paths use the same renderer, so a tab or newline inside a value can no longer add a field or a row |
| `cell_to_json_value` | an integer inside ±2^53 → JSON number; otherwise a JSON number only when the decimal text is exactly the shortest form of the parsed `f64`; anything else keeps its exact digits as a JSON string |

The JSON rule follows the same contract the graded export service already applies to decimals
(`crates/core/src/application/export_service.rs:216` — "Keep decimal text exact instead of converting
through f64"): a value whose text cannot survive being read back as a JSON number is emitted as a JSON
string instead of being rounded. `serde_json` renders an `f64`-backed integer as `100.0`, which is why
the ±2^53 integer branch exists — `100` must stay `100`.

## 3. Verification

`cargo test -p db-pro-ui --lib` → **364 passed / 0 failed / 0 ignored** (363 before; 3 new tests added
in the same run that fixed them, 1 pre-existing test updated — the assertions themselves are unchanged,
only the location of the assertions changed):

| Test | What it pins |
|---|---|
| `app::tests::test_delimited_export_keeps_field_count_for_awkward_values` | exact CSV and TSV text for a row whose text cell holds `,`, `"` and `\n`, next to an integer past 2^53 |
| `app::tests::test_copy_as_json_keeps_exact_numeric_digits` | `9007199254740993` and `42.50` and `12345678901234567890.12345` stay exact; `1.5` stays a JSON number; the copied text of `100` is `100` |
| `app::tests::test_export_result_writes_escaped_delimited_text` | drives `DbProApp::export_result` for real: the file on disk is `note,amount\n"line, one\ntwo",42.50\n,9007199254740993\n` (NULL empty, exact digits, newline quoted) and the status message/dialog state are correct |
| `app::tests::test_format_cell_csv_and_cell_to_json` (pre-existing) | unchanged expectations still hold |

Provider-value matrix as covered by the tests above plus the #56 decoder tests:

| Value class | Copy (tab) | Copy as CSV | Copy as JSON | Export file | Display |
|---|---|---|---|---|---|
| NULL | empty field | empty field | `null` | empty field | `NULL` |
| BIGINT past 2^53 | exact digits | exact digits | exact string | exact digits | exact digits |
| DECIMAL with trailing zeroes | exact digits | exact digits | exact string | exact digits | exact digits |
| float | exact digits | exact digits | JSON number when exact | exact digits | exact digits |
| TEXT with `,`/`"`/newline | quoted | quoted | JSON string | quoted | raw (display) |
| JSON | escaped | escaped | parsed as JSON | escaped | raw (display) |
| BYTEA | escape-safe `\x…` text as delivered by the runtime | same | JSON string | same | `\x…` |
| TIMESTAMP / TIMESTAMPTZ | canonical decoder strings, no `Date` parsing anywhere in these paths | same | same | same | same |

TIMESTAMP/TIMESTAMPTZ are strings end to end in the UI layer — there is no `Date`/`chrono` parse in any
copy or export path (`grep -n "chrono\|Date" crates/ui/src/result_grid_view.rs crates/ui/src/query_view.rs`
matches only unrelated identifiers), so the frontend cannot shift an instant, and #56 fixed the
producer side.

Gate line for this change: `cargo fmt --all -- --check`, `cargo check --workspace`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
`cargo build --release --locked -p db-pro-native`, `bash .skills/perf-audit/scripts/perf-scan.sh`
(recorded numbers in `LEDGER.md`).

## 4. Not claimed here

- Dedicated DTO variants for the temporal classes (#52/#54, see evidence file 30).
- Type-aware editability and the mutation policy (#62) — untouched here.
- CSV exported from the **graded** `ExportService` (`crates/core/src/application/export_service.rs`) was
  already correct and is out of scope: the issue is about the frontend surfaces.
