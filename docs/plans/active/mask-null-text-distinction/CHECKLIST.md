# Masking NULL correctness and unmatched rules — Checklist

## Planning
- [x] Evidence and failure scenario recorded (PLAN.md)
- [x] Scope/non-goals explicit
- [x] PostgreSQL/SQLite support matrix explicit (provider-independent)

## Implementation
- [x] `mask_cell` takes `Option<&str>`; both `eq_ignore_ascii_case("null")` checks removed
- [x] `mask_row` null-aware (`&[Option<String>]`)
- [x] `preview_masking` null-aware; preview rendered to `String`
- [x] `mask_transfer_batch` wraps string cells (`Some`) and documents the NULL limitation
- [x] App callers updated (`build_preview`, `masked_csv_export_outcome`)
- [x] No change to `TransferRow` model / `MaskRule` set

## Tests
- [x] Core regression: SQL NULL vs text `null`/`NULL`/`""` for every rule
- [x] Row/preview caller regression: textual `null` is masked
- [x] App caller regression: build_preview / harness input masks textual `null`
- [x] PostgreSQL/SQLite: N/A (pure function, no SQL emitted)

## Review
- [x] Architecture review complete
- [x] Correctness/security review complete
- [x] P0 = 0
- [x] P1 = 0

## Runtime
- [ ] UI → command → backend → DB → introspection → UI — N/A: masking is a pure
      domain transform, not wired to a database call site
- [x] VERIFICATION.md contains actual commands/evidence
- [x] STATUS.md matches state/folder (updated in publishing PR)

## Related issue #503
- [x] Preserve exact case-sensitive rule/header matching (quoted PostgreSQL identifiers may differ by case)
- [x] Preview warns and names rules that match no header
- [x] Regression tests cover unmatched/matched headers and warning contents
