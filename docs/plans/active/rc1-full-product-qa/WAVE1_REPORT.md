# Wave 1 Report

```text
Wave: W1 — BIGINT precision, preview staged leak, close guard bypass, SQLite metadata
Branch: fix/rc1-p1-precision-staged-state
Head SHA: b724c8c
Confirmed findings (original):
  - QA-P1-01: BIGINT/i64 precision loss across Rust→Tauri→JS IPC — CONFIRMED, FIXED
  - QA-P1-02: Preview tab replacement carries staged mutations to another table — CONFIRMED, FIXED
  - QA-P1-03: Staged Data Grid edits bypass tab dirty/close protection — CONFIRMED, FIXED
  - QA-P1-04: SQLite metadata reports every result column as TEXT — CONFIRMED, FIXED
Additional findings from PR #11 review (all FIXED):
  - P1-1: CellValueDto::Int64 in Tauri DTO layer missing string_i64 serde adapter — IPC boundary still broken
  - P1-2: Action Platform closeTabAction and Command Palette requestCloseMany only checked tab.dirty — Ctrl/Cmd+W bypassed guard
  - P1-3: Preview replacement silently discarded staged work instead of promoting tab — violated "cannot be silently replaced" criteria
  - P1-4: SQLite fix was on dead path (query_mapper) — actual runtime path SqliteActor::handle_execute still hardcoded "TEXT"
  - P2-1: getSortValue cast int64 string value as number — lexical sort instead of numeric
  - P2-2: i64 range validation missing in cell-editor and row-edit-dialog — could accept values outside i64 bounds
Rejected/downgraded findings: none
Files changed: 32
Tests added:
  - Rust: 3 boundary-value tests for i64 serde (0, ±1, i64::MAX, i64::MIN, 2^53-1, 2^53, 2^53+1)
  - Frontend: updated 13 test files to expect string int64 values (regression coverage for type contract)
  - Existing close-guard tests (request-close-tab, staged-changes-store, staged-changes) cover QA-P1-03
  - Existing workspace store tests cover QA-P1-02 preview replacement path
  - Workspace store tests updated for preview promotion logic (staged changes → promote, clean → replace)
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
  - SQLite: column_decltype now reads real declared types via rusqlite `column_decltype` feature; SqliteActor::handle_execute now uses extract_columns() for actual runtime path; runtime editor capability evidence deferred (needs office DB)
P0: 0 open
P1: 0 open (W1 scope)
P2: 0 open (BigInt sort + i64 validation fixed)
Runtime evidence still pending:
  - PG + SQLite row with PK 9007199254740993 displays and mutates exact row (needs office DB)
  - SQLite INTEGER/BLOB/text table editor shows correct capabilities (needs office DB)
  - Preview replacement staged leak manual smoke (needs running app)
  - Close guard with staged edits manual smoke (needs running app)
PR: https://github.com/truongnat/db-pro/pull/11
```

## Fix Summary

### QA-P1-01: BIGINT/i64 precision loss

**Root cause**: Tauri IPC uses JSON serialization. `i64` values serialized as JSON numbers lose precision when parsed by JavaScript (IEEE-754 cannot represent all signed 64-bit integers exactly).

**Fix**: Custom serde module `string_i64` on `CellValue::Int64` and `QueryParam::Int64` enum variants serializes i64 as string on the wire. Frontend `CellValue` type changed from `{ type: "int64"; value: number }` to `{ type: "int64"; value: string }`. All frontend consumers (cell-editor, row-edit-dialog, filter-parser, chart-view) updated to handle string values. Chart view uses `Number()` with `Number.isFinite` guard since charts need approximate values, not lossless.

**Additional fix (P1-1)**: `CellValueDto::Int64` in `crates/tauri-app/src/dto.rs` is a separate type from `CellValue` in core domain. Applied the same `string_i64` serde adapter to the DTO layer — the actual Tauri IPC boundary.

### QA-P1-02: Preview tab staged mutation leak

**Root cause**: Workspace store reuses tab ID when replacing a preview tab with a new resource. Staged changes keyed by tabId therefore leak from the original table to the replacement table.

**Fix**: Both `openTab` and `openDbObject` now check `useStagedChangesStore.getState().getCount(existingPreview.id) > 0` before replacing a preview tab. If the preview has staged changes, it is promoted (preview→false) and the new resource opens as a new tab. If clean, the preview is replaced and grid state is reset via `useTabGridStateStore.getState().resetTab(reusedId)`.

**Additional fix (P1-3)**: Original fix silently discarded staged work via `clearTab()`. Updated to promote the preview tab instead, preserving user's staged mutations.

### QA-P1-03: Staged edits bypass close guard

**Root cause**: `requestCloseTab` and `useTabCloseGuard` only checked `tab.dirty` for unsaved work. Staged data-grid edits live in a separate store (`useStagedChangesStore`) and were not checked.

**Fix**: Created unified `hasUnsavedWork(tabId)` in `request-close-tab.ts` that checks both `tab.dirty` AND `useStagedChangesStore.getState().getCount(id) > 0`. All close paths (single close, close many, confirm dialog) now clear staged changes before closing.

**Additional fix (P1-2)**: Action Platform `closeTabAction` in `workspace.actions.ts` and Command Palette `requestCloseMany` in `register-commands.ts` were separate code paths that still only checked `tab.dirty`. Updated both to check staged changes count, ensuring Ctrl/Cmd+W and "Close Others"/"Close Right" commands respect the guard.

### QA-P1-04: SQLite metadata always TEXT

**Root cause**: `extract_columns()` in `query_mapper.rs` hardcoded `"TEXT"` for all column data types instead of reading the declared type from the SQLite statement.

**Fix**: Enabled `column_decltype` feature on rusqlite dependency. `extract_columns()` now uses `stmt.columns()` → `columns[i].decl_type()` to read the actual declared column type from the prepared statement, falling back to `"TEXT"` only when no declared type is available.

**Additional fix (P1-4)**: Original fix was on a dead code path. `SqliteActor::handle_execute()` in `crates/infrastructure/src/sqlite/actor.rs` is the actual runtime path for SQLite queries. It was building columns inline with hardcoded `"TEXT"` instead of calling `extract_columns()`. Replaced inline column building with `let columns = extract_columns(&stmt);` to wire the fix into the runtime path.

### P2-1: BigInt-safe sorting

**Root cause**: `getSortValue` in `query-tab-content.tsx` cast int64 value as `number` for sorting. After the i64 wire contract change, int64 values are strings, so `"10"` sorts before `"2"` lexically instead of numerically.

**Fix**: Changed `getSortValue` to return `bigint` for int64 cells by parsing the string value with `BigInt()`. Updated the sort comparator to handle bigint comparisons safely without mixing types (bigint vs number throws TypeError in JS). Null values sort to the end.

### P2-2: i64 range validation

**Root cause**: Cell editor and row edit dialog accepted any integer string without validating i64 bounds (-2^63 to 2^63-1).

**Fix**: Added range validation in both `cell-editor.tsx` and `row-edit-dialog.tsx`. After the regex check passes, parse as BigInt and verify the value is within i64 bounds. Display "Integer out of range" error if outside bounds.
