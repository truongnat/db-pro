# Masking NULL correctness and unmatched rules — Findings

Record only evidence-backed findings.

## P0

None.

## P1

### F1 — `mask_cell` treats textual `"null"`/`"NULL"` as SQL NULL and bypasses masking

- Evidence: `crates/core/src/domain/masking.rs:70-73` (pre-fix) and `:78-84`
  (pre-fix) short-circuit `value.eq_ignore_ascii_case("null")` to `"NULL"` before
  the rule match. Reproduced by the pre-fix test
  `crates/core/src/domain/tests/masking_tests.rs:51`
  (`mask_cell("NULL", PartialReveal) == "NULL"`).
- Failure scenario: a `Redact`/`Hash`/`Fixed`/`PartialReveal`/`Synthetic` column
  holds the literal text `null`/`NULL`; the rule is skipped and the raw token is
  emitted as `"NULL"`, indistinguishable from a real SQL NULL.
- Severity: P1 — security-boundary correctness defect in the masking API.
  Scope note: the bypass needs an exact case-insensitive `null` token, and the
  masking API is currently reached only from preview/demo harnesses
  (`crates/app/src/tools/masking.rs`, `transfer_harness_view.rs`), not a live
  export path — recorded honestly rather than overstated.
- Fix: `mask_cell` now takes `Option<&str>`; `None` → `"NULL"`,
  `Some(s)` → rule applied. Both `eq_ignore_ascii_case("null")` checks removed.
- Regression: `sql_null_is_rendered_not_masked`,
  `literal_null_text_is_masked_by_every_rule`, `preserve_null_keeps_text_and_renders_nulls`,
  `mask_row_distinguishes_sql_null_from_text_null`,
  `preview_masking_masks_textual_null`, `transfer_batch_masks_textual_null_cells`.

## P2

### F4 — configured mask rule can silently match no header (#503)

- Evidence: `mask_row` intentionally uses exact equality between configured
  `ColumnMask.column` and header names. A case-insensitive comparison could
  incorrectly treat quoted PostgreSQL identifiers such as `email` and `Email` as
  the same column.
- Fix: retain exact matching, expose `unmatched_rule_columns`, and append a
  warning with rule names to `MaskingPreview.message`; that message is rendered
  in the masking preview UI. No case semantics are widened.
- Regression: `unmatched_rule_columns_are_case_sensitive` and
  `preview_masking_warns_on_unmatched_rule`.

### F2 — Transfer rows cannot express SQL NULL distinct from empty text

- Evidence: `crates/core/src/domain/transfer.rs:204-207` stores `Vec<String>`;
  SQL NULL is encoded as the empty string across the pipeline
  (`application/table_copy.rs:294`, `application/export_formats.rs:122`,
  `application/json_excel_transfer.rs:146`).
- Consequence: `mask_transfer_batch` treats every cell as present text
  (`Some`). A provider NULL that reaches masking as `""` keeps its prior empty
  behavior (rule applies) rather than rendering `"NULL"`. This is pre-existing
  pipeline behavior, not a regression introduced here.
- Decision: not fixed in this change. Changing `TransferRow.cells` to
  `Vec<Option<String>>` would ripple through every transfer source/target and
  require provider runtime re-verification — out of scope for a P1 masking fix.

### F3 — Preview display conflates real NULL with text `"NULL"`

- Evidence: `MaskingPreview.original/masked` are `Vec<Vec<String>>`
  (`crates/core/src/domain/masking.rs`); `preview_masking` renders `None` as
  `"NULL"` for display.
- Consequence: in the preview *original* column a real SQL NULL and a text
  `"NULL"` look identical. The masked column is still unambiguous to programmatic
  callers because the rule is applied to text.

## Rejected / downgraded findings

- Issue text claim "rò giá trị thật" (leaks the real value): the value emitted is
  the literal token `"null"`, not arbitrary sensitive data. The verifiable defect
  is the rule bypass plus ambiguous NULL output, not a general data leak. Severity
  retained at P1 because it is the security masking boundary and the output is
  silently wrong.
- A repo-wide search found no `crates/ui` consumer of `mask_cell`/`mask_row`;
  masking is not wired to `UiCell`. The #503 warning is surfaced by the preview
  message rendered in the app UI.

## Runtime gaps

- PostgreSQL: N/A — pure text transform, emits no SQL.
- SQLite: N/A — pure text transform, emits no SQL.
- UI: masking is preview-only (#253); no `UI → command → DB → introspection → UI`
  lifecycle exists for it.
