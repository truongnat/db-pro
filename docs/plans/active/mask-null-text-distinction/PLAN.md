# Masking NULL correctness and unmatched rules — Plan

State: REVIEW
Branch: `fix/mask-null-text-distinction`
Issues: truongnat/db-pro#502, truongnat/db-pro#503
Baseline SHA: `d5aea9568430c5391bb6150a9748332fd2e0d62e`
Parent branch: `main` (clean, synced with `origin/main`)

## Goal

Group the related masking correctness issues #502 and #503 in one PR. `mask_cell`
must only treat a cell as SQL NULL when the caller says so. The preview must also
make configured rules that match no source header visible to the user. A present
text value that happens to be `"null"`, `"NULL"` (any case) or `""` is ordinary
data and must be processed by the configured `ColumnMask` rule. Today the early
`value.eq_ignore_ascii_case("null")` short-circuit returns the literal `"NULL"`
for every rule, bypassing masking.

Invariant: a masked output equal to `NULL` means the source cell was a real SQL
NULL, never that a textual value escaped masking.

## Evidence / baseline (source)

- `crates/core/src/domain/masking.rs:70-73` — `mask_cell` early-returns `"NULL"`
  for any value equal to `null`/`NULL` (case-insensitive), before the rule match.
- `crates/core/src/domain/masking.rs:78-84` — `MaskRule::PreserveNull` repeats the
  same `eq_ignore_ascii_case("null")` check.
- `crates/core/src/domain/masking.rs:123-136` — `mask_row` passes `&str` cells.
- `crates/core/src/domain/masking.rs:162-174` — `mask_transfer_batch` passes
  `TransferRow.cells` (`Vec<String>`).
- `crates/core/src/domain/tests/masking_tests.rs:51` — existing test asserts the
  buggy behavior: `mask_cell("NULL", PartialReveal) == "NULL"`.

### Failure scenario

1. A column `email` is assigned rule `Redact` (or `Hash`/`Fixed`/`PartialReveal`/
   `Synthetic`).
2. A row holds the literal text value `"null"` (e.g. CSV import wrote the string,
   or the user typed `null`), which is valid data, not SQL NULL.
3. `mask_cell` sees `eq_ignore_ascii_case("null")`, returns `"NULL"` verbatim.
4. The redaction never happens. The output is indistinguishable from a genuine
   SQL NULL, so the bypass is silent.

### Data representations (independently verified)

- Query layer: `crates/ui/src/runtime/runtime_query_types.rs:12-20` `UiCell`
  distinguishes `Null` from `Text(String)`; masking is **not** wired to `UiCell`.
- Transfer model: `crates/core/src/domain/transfer.rs:204-207` `TransferRow.cells`
  is `Vec<String>`; the pipeline represents SQL NULL as the empty string
  (`application/table_copy.rs:294`, `application/export_formats.rs:122`,
  `application/json_excel_transfer.rs:146`), so the transfer model cannot
  distinguish SQL NULL from an empty text cell.
- Domain preview: `MaskingPreview.original/masked` are `Vec<Vec<String>>`.

## Scope

1. `crates/core/src/domain/masking.rs`
   - `mask_cell(value: Option<&str>, …)`: `None` = SQL NULL → `"NULL"`;
     `Some(s)` = present text, rule always applied. Remove both
     `eq_ignore_ascii_case("null")` checks.
   - `mask_row(headers, cells: &[Option<String>], …)`: null-aware row masking.
   - `preview_masking(headers, rows: &[Vec<Option<String>>], …)`: null-aware
     preview; `original`/`masked` rendered to `String` (`None` → `"NULL"`).
   - `mask_transfer_batch`: keep `&[TransferRow]`; the transfer model has no NULL
     representation, so each cell is passed as `Some(text)` and a real provider
     NULL keeps its existing empty-text behavior (documented limitation).
2. Update callers:
   - `crates/app/src/tools/masking.rs` (`build_preview` builds `Option` rows).
   - `crates/app/src/tools/transfer_harness_view.rs` (`masked_csv_export_outcome`).
3. Regression tests distinguishing SQL NULL from text `null`/`NULL`/`""` for
   `mask_cell`, `mask_row`, `preview_masking` and the app caller.
4. For #503, preserve exact case-sensitive column matching (PostgreSQL quoted
   identifiers may differ by case) and show a preview warning naming any rule
   whose column matches no header.
5. Keep `docs/plans/STATUS.md` in sync with both issues and verified gates.

## Non-goals

- No change to the `TransferRow` model (`Vec<String>` stays). Changing it would
  ripple through every transfer source/target and require provider runtime
  re-verification; it is not required to close the security bypass.
- No new UI surface for masking; masking remains preview-only (#253).
- No change to `MaskRule` set, `ColumnMask` shape, or the keyed-hash contract.

## Provider matrix

Masking is provider-independent (it operates on already-materialized cell text,
never on a live connection). No provider SQL is emitted.

| Provider | Supported | Required proof |
|---|---|---|
| PostgreSQL | n/a | provider-independent pure function; no SQL emitted |
| SQLite | n/a | provider-independent pure function; no SQL emitted |

## Acceptance criteria

- [x] `mask_cell(Some("null"), <any rule>)` applies the rule, never returns the
      literal input as `"NULL"`.
- [x] `mask_cell(None, <any rule>)` returns `"NULL"`.
- [x] `mask_cell(Some(""), Redact)` returns `"***"`; `Some("")` + `PreserveNull`
      returns `""`.
- [x] Callers (`mask_row`, `preview_masking`, `masked_csv_export_outcome`) mask a
      textual `"null"` cell.
- [x] `cargo test -p db-pro-core masking` and `-p db-pro-app masking` pass.
- [x] `cargo check --workspace`, formatting pass; clippy has inherited failures in untouched `saved_task.rs`.
