# Wave 1 Report

```text
Wave: W1 — BIGINT precision, preview staged leak, close guard bypass, SQLite metadata
Branch: fix/rc1-p1-precision-staged-state
Head SHA: (pending commit)
Confirmed findings:
  - QA-P1-01: BIGINT/i64 precision loss across Rust→Tauri→JS IPC — CONFIRMED, FIXED
  - QA-P1-02: Preview tab replacement carries staged mutations to another table — CONFIRMED, FIXED
  - QA-P1-03: Staged Data Grid edits bypass tab dirty/close protection — CONFIRMED, FIXED
  - QA-P1-04: SQLite metadata reports every result column as TEXT — CONFIRMED, FIXED
Rejected/downgraded findings: none
Files changed: 27
Tests added:
  - Rust: 3 boundary-value tests for i64 serde (0, ±1, i64::MAX, i64::MIN, 2^53-1, 2^53, 2^53+1)
  - Frontend: updated 13 test files to expect string int64 values (regression coverage for type contract)
  - Existing close-guard tests (request-close-tab, staged-changes-store, staged-changes) cover QA-P1-03
  - Existing workspace store tests cover QA-P1-02 preview replacement path
Frontend gates:
  - typecheck: PASS
  - lint: PASS
  - format:check: PASS
  - check:tokens: PASS
  - test: 1335/1335 PASS (108 files)
  - build: PASS
Rust gates:
  - cargo fmt --all -- --check: PASS
  - cargo check --workspace: PASS
  - cargo clippy --workspace --all-targets: PASS
  - cargo test --workspace: PASS (24 tests including 3 new i64 boundary tests)
Provider evidence:
  - PostgreSQL: i64 serde round-trip verified at unit level; runtime row-level evidence deferred (needs office DB)
  - SQLite: column_decltype now reads real declared types via rusqlite `column_decltype` feature; runtime editor capability evidence deferred (needs office DB)
P0: 0 open
P1: 0 open (W1 scope)
P2 introduced: 0
Runtime evidence still pending:
  - PG + SQLite row with PK 9007199254740993 displays and mutates exact row (needs office DB)
  - SQLite INTEGER/BLOB/text table editor shows correct capabilities (needs office DB)
  - Preview replacement staged leak manual smoke (needs running app)
  - Close guard with staged edits manual smoke (needs running app)
PR: (pending)
```

## Fix Summary

### QA-P1-01: BIGINT/i64 precision loss

**Root cause**: Tauri IPC uses JSON serialization. `i64` values serialized as JSON numbers lose precision when parsed by JavaScript (IEEE-754 cannot represent all signed 64-bit integers exactly).

**Fix**: Custom serde module `string_i64` on `CellValue::Int64` and `QueryParam::Int64` enum variants serializes i64 as string on the wire. Frontend `CellValue` type changed from `{ type: "int64"; value: number }` to `{ type: "int64"; value: string }`. All frontend consumers (cell-editor, row-edit-dialog, filter-parser, chart-view) updated to handle string values. Chart view uses `Number()` with `Number.isFinite` guard since charts need approximate values, not lossless.

### QA-P1-02: Preview tab staged mutation leak

**Root cause**: Workspace store reuses tab ID when replacing a preview tab with a new resource. Staged changes keyed by tabId therefore leak from the original table to the replacement table.

**Fix**: Both `openTab` and `openDbObject` now call `useStagedChangesStore.getState().clearTab(replacedPreviewId)` when replacing a preview tab, ensuring staged mutations are discarded before the tab identity changes.

### QA-P1-03: Staged edits bypass close guard

**Root cause**: `requestCloseTab` and `useTabCloseGuard` only checked `tab.dirty` for unsaved work. Staged data-grid edits live in a separate store (`useStagedChangesStore`) and were not checked.

**Fix**: Created unified `hasUnsavedWork(tabId)` in `request-close-tab.ts` that checks both `tab.dirty` AND `useStagedChangesStore.getState().getCount(id) > 0`. All close paths (single close, close many, confirm dialog) now clear staged changes before closing.

### QA-P1-04: SQLite metadata always TEXT

**Root cause**: `extract_columns()` in `query_mapper.rs` hardcoded `"TEXT"` for all column data types instead of reading the declared type from the SQLite statement.

**Fix**: Enabled `column_decltype` feature on rusqlite dependency. `extract_columns()` now uses `stmt.columns()` → `columns[i].decl_type()` to read the actual declared column type from the prepared statement, falling back to `"TEXT"` only when no declared type is available.
