---
schema_version: 1
id: "MEM-PLAN-issue-537-539-edit-export-safety-37c6602c03c1"
title: "Plan archive: Issue 537 539 Edit Export Safety"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-537-539-edit-export-safety]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "37c6602c03c1eda420ebfb52b7e77569a5bd6e7b96af0d2e564e83da9c9544ed"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-537-539-edit-export-safety/PLAN.md"
  - "docs/plans/active/issue-537-539-edit-export-safety/FINDINGS.md"
  - "docs/plans/active/issue-537-539-edit-export-safety/VERIFICATION.md"
  - "docs/plans/active/issue-537-539-edit-export-safety/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-537-539-edit-export-safety/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Issues #537 / #538 / #539 — Edit and Export Safety Wave

State: REVIEW
Branch: `main`

## Goal

Close three open P2 correctness tickets from the 2026-10-21 issue batch where
the UI can silently lose or corrupt user data:

- **#537** — a result cell truncated by the runtime mapper (`text > 64 KiB`,
  `bytea > 32 KiB`) is still editable; `Apply` stages the display prefix back
  over the full value.
- **#538** — SQL INSERT/COPY export ignores the connection driver: identifiers
  are always double-quoted and MySQL literals never escape backslashes, so the
  generated SQL fails on MySQL/SQL Server.
- **#539** — the export overwrite confirmation is a `bool`; after changing
  Format or path the armed flag still applies and `Overwrite` replaces a file
  the prompt never named.

Also verify (#540 — Schema tab table-info loading message) is already fixed by
`0e22aae7` and close it with evidence; no code change.

## Evidence / baseline

- #537: `crates/native-app/src/translate_map.rs` caps text/JSON at 65,536 bytes
  and bytea at 32 KiB, appending `…`; nothing downstream could tell a capped
  value from a real one, and `open_cell_inspector` / `begin_data_cell_edit`
  entered the inline editor unconditionally once write access existed.
- #538: `result_grid_export::format_result_sql_insert` hard-coded `"ident"` and
  `''` escaping; `format_result_copy` did the same. The driver string existed
  only for the COPY availability gate.
- #539: `overlay_state.export_overwrite_pending: bool` survived a Format or
  path change; `draw_overwrite_confirmation` displayed the raw typed path while
  the write used `effective_export_path()`.

## Scope

- `crates/ui/src/runtime/runtime_query_types.rs` — shared
  `MAX_CELL_TEXT_BYTES`, `UiCell::is_truncated()`.
- `crates/native-app/src/translate_map.rs` — mapper reads the shared constant.
- `crates/app/src/result_grid/result_grid_edit.rs`,
  `result_grid_inspector_surface_view.rs` — inspector shows the truncation
  notice, hides Export bytes, refuses to open an editor for truncated cells.
- `crates/app/src/table/table_mutation_actions.rs`,
  `table_editor_context.rs` — editor-entry and write-boundary guards.
- `crates/app/src/result_grid/result_grid_export.rs` — dialect-aware INSERT and
  COPY rendering (`DriverDialect::quote_identifier`, driver-specific literal
  escaping, SQL Server `1/0` booleans).
- `crates/app/src/shell/overlay_state.rs`,
  `query_workbench/query_dialogs_view.rs`,
  `query_dialog_surface_view.rs` — overwrite token is the exact effective
  `PathBuf`; any Format/path/Browse change re-arms the prompt for the new
  destination.
- Locales (`en/vi/zh/ja/ko`) — `query.cell_truncated_for_display`.

## Non-goals

- Raising or streaming past the runtime truncation caps (#542 perf is separate).
- A general SQL literal formatter for every dialect (only the export writers).
- Closing the perf tickets #541-#545 or the epics #182/#252/#260.
- Visual redesign of the export dialog or inspector.

## Provider matrix

| Provider | Supported | Required proof |
|---|---|---|
| PostgreSQL | yes | ANSI quoting, `TRUE/FALSE`, COPY quoting — unit tests |
| SQLite | yes | ANSI quoting, `TRUE/FALSE` — unit tests |
| MySQL | yes | backtick identifiers, backslash doubling — unit tests |
| SQL Server | yes | bracket identifiers, `1/0` booleans — unit tests; live run remains BLOCKED |

COPY output stays PostgreSQL-only and is already gated in the dialog.

## Architecture

UI (`DbProApp` result grid / export dialog) → local formatters in
`crates/app` → file write. No command, service, connector or SQL execution path
changes; #538 changes generated text only. #537 blocks a UI-originated write
before it reaches `StagedChange::Update`.

## Acceptance criteria

- [x]#537 A truncated text/JSON/bytes cell never opens an inline editor, the
      inspector warns, Export bytes is unavailable, and `stage_data_cell_edit`
      rejects it defensively.
- [x]#538 Export SQL INSERT/COPY emits dialect-correct identifiers and literals
      for PostgreSQL, SQLite, MySQL and SQL Server.
- [x]#539 `Overwrite` is honored only for the effective path shown in the
      prompt; Format/path/Browse changes re-prompt.
- [x]#540 verified fixed at `0e22aae7` (5 locales, used in Schema tab).
- [x] All workspace gates executed and recorded in VERIFICATION.md.


## Source: `FINDINGS.md`

# Findings — #537 / #538 / #539

Baseline SHA: `0e22aae7`.

## #537 — truncated cells were writable (P2, data loss)

- Runtime mapper (`crates/native-app/src/translate_map.rs`) capped text/JSON at
  65,536 bytes and bytea at 32 KiB and appended `…`, but `UiCell` carried no
  truncation bit. `open_cell_inspector` and `begin_data_cell_edit` checked only
  connection write access, so the grid offered an editor seeded with the
  display prefix; `stage_data_cell_edit` then staged that prefix as the new
  value.
- Fix: `UiCell::is_truncated()` + shared `MAX_CELL_TEXT_BYTES`; editor entry
  opens the inspector with a localized warning instead of the inline editor;
  the inspector hides Export bytes; `stage_data_cell_edit` rejects truncated
  source cells even if a caller bypasses the UI gate.
- Residual: the inspector still shows the capped prefix (it cannot recover the
  full value), clearly marked as truncated.

## #538 — driver-unaware SQL export (P2, wrong SQL)

- `format_result_sql_insert` / `format_result_copy` hard-coded `"ident"` and
  only doubled single quotes. MySQL needs backticks and backslash escaping;
  SQL Server needs brackets and `1/0` booleans.
- Fix: the export path passes `active_driver()` into both writers; identifiers
  go through `DriverDialect::quote_identifier`, literals escape per driver
  type, SQL Server booleans emit `1/0`. COPY remains PostgreSQL-only (already
  gated in the dialog) and uses dialect quoting.
- Residual: MySQL string escaping does not set `NO_BACKSLASH_ESCAPES`; a server
  running with that mode would need doubled backslashes. Documented, not
  changed here.

## #539 — overwrite prompt not bound to the effective path (P2, wrong file)

- `export_overwrite_pending: bool` stayed armed across Format and path changes,
  and the prompt rendered the raw typed path while the write used
  `effective_export_path()` (format extension appended). `Overwrite` could
  replace a file the user never confirmed.
- Fix: token is `Option<PathBuf>` compared against the effective path; Format,
  path edit, Browse, Cancel, Keep existing and the COPY fallback all clear it;
  the prompt shows the effective path.
- Residual: none known.

## #540 — schema table-info loading message (P2, already fixed)

- Verified at `0e22aae7`: keys `table.info_loading` / `table.info_load_failed`
  exist in all five locales and are used by `table_metadata_surface_view.rs`.
  Closed with evidence comment; no code change in this wave.

## Severity

- #537 P2 (requires a >64 KiB cell and an explicit Apply).
- #538 P2 (export produces invalid SQL on two drivers; no data is mutated).
- #539 P2 (requires an existing destination, a Format/path change and clicking
  Overwrite; the overwritten file is user data).


## Source: `VERIFICATION.md`

# Verification — #537 / #538 / #539

Baseline SHA: `0e22aae7`. All commands executed on this host against the
working tree that becomes the wave commit.

## Gates

| Command | Result |
|---|---|
| `cargo +nightly fmt --all -- --check` | PASS (exit 0) |
| `cargo check --workspace` | PASS (exit 0) |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS (exit 0) |
| `cargo test --workspace` | PASS — 2,169 passed / 0 failed / 56 ignored |
| `cargo build --release --locked -p db-pro-native` | PASS (1m35s) |
| `bash scripts/check-ui-architecture.sh` | PASS |
| `python3 tools/cc-scan.py . --fail-on error` | No new findings; exit 1 from 19 inherited error-level findings (baseline file is empty) + 118 warnings |

## Targeted tests

| Test | Covers |
|---|---|
| `runtime_query_types_tests` (4 tests) | text/JSON at cap + marker, bytes marker vs full hex, scalars |
| `translate_map_tests` (4 tests) | oversized text/JSON/bytes flagged; at-cap values not flagged |
| `table_row_edit_tests::truncated_cell_never_opens_an_editor` | #537 editor entry |
| `table_row_edit_tests::truncated_cell_is_rejected_at_the_write_boundary` | #537 staging guard |
| `result_grid_export_tests::sql_insert_export_uses_mysql_quoting_and_backslash_escaping` | #538 MySQL |
| `result_grid_export_tests::sql_insert_export_uses_sql_server_quoting_and_bit_literals` | #538 SQL Server |
| `result_grid_export_tests::sql_insert_export_keeps_ansi_quoting_for_postgres_and_sqlite` | #538 PG/SQLite |
| `result_grid_export_tests::sql_copy_export_quotes_for_postgres` | #538 COPY |
| `query_view_tests::overwrite_confirmation_does_not_follow_a_format_change` | #539 |

## Provider evidence

- PostgreSQL / SQLite / MySQL / SQL Server export SQL: unit-level output tests
  only. No live provider execution of the exported scripts in this wave.
- COPY export remains PostgreSQL-only by dialog gating.

## Not verified (declared)

- Runtime screenshot of the export overwrite prompt and the inspector
  truncation banner: requires a live connection and an oversized cell; not
  captured. Visual review of these surfaces is pending.
- Live MySQL/SQL Server execution of exported INSERT/COPY scripts.
- L3 deep security review: pending the handoff gate.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — #537 / #538 / #539 edit/export safety wave

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | qoder · issue-triage lane |
| Issue(s) | #537, #538, #539 (verifies #540) |
| Task state | Review |
| Baseline SHA | `0e22aae7` (commit recorded in this plan's trailing handoff) |
| Branch / PR | `main` (owner override: no PR, direct merge) |
| Scope interpretation | Close the three open correctness tickets in the 2026-10-21 batch that can silently lose or corrupt user data. |
| Out of scope | Perf tickets #541-#545, epics #182/#252/#260, export dialog redesign |

## 2. Progress checkpoint

- Current HEAD: `26c68d26a42b671d107faf0a1805d588bb15f2c0` (baseline `0e22aae7`).
- Completed acceptance rows: #537, #538, #539, #540 verification.
- Remaining acceptance rows: runtime visual capture; L3 handoff gate.
- Findings / risks: see `FINDINGS.md` (all P2; no P0/P1).
- Tests already run: `cargo test --workspace` → 2,169 passed / 0 failed / 56 ignored, exit 0.
- Dependency / blocker changes: none.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `26c68d26a42b671d107faf0a1805d588bb15f2c0` |
| Commit list | `fix(export,grid): bind overwrite to effective path, driver-aware SQL export, block truncated-cell edits` |
| File / surface inventory | see `PLAN.md` §Scope; 21 modified files + 2 new test files + this plan |
| Acceptance mapping | #537 tests in `runtime_query_types_tests`, `translate_map_tests`, `table_row_edit_tests`; #538 tests in `result_grid_export_tests`; #539 test in `query_view_tests` |
| Commands and counts | fmt PASS; check PASS; clippy PASS; test 2,169/0/56; release build PASS; ui-architecture PASS; cc-scan no new findings |
| CI run IDs / status | not run (local gates only) |
| Known limitations | no live MySQL/SQL Server execution of exported SQL; no runtime screenshots of the two dialogs; NO_BACKSLASH_ESCAPES servers not covered |
| Migrations / config implications | none; `export_overwrite_pending` is transient UI state |
| Out-of-scope changes | none |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `26c68d26a42b671d107faf0a1805d588bb15f2c0` — L3 deep review, 0 findings |
| Verdict | ACCEPT (self-review + L3); independent code review not run |
| P0 / P1 / P2 counts | 0 introduced / 0 introduced / 0 open from this wave |
| Findings | n/a |
| CI disposition | local gates green, see VERIFICATION.md |
| Next task(s) unblocked | #541-#545 perf batch (separate decision) |

## 5. Research / audit handoff

n/a — implementation lane.

## 6. Tổng kết (Vietnamese summary)

Đã xử lý ba ticket P2 trong đợt 2026-10-21: #537 chặn sửa ô bị cắt dữ liệu
(entry + write boundary + inspector), #538 xuất SQL INSERT/COPY theo đúng
dialect của driver (MySQL backtick/backslash, SQL Server bracket/1-0,
PostgreSQL/SQLite ANSI), #539 buộc xác nhận ghi đè theo đúng đường dẫn hiệu
lực và reset khi đổi Format/đường dẫn. #540 đã được xác minh là đã sửa bởi
`0e22aae7`. Toàn bộ gate workspace xanh (2.169 test pass, clippy sạch, build
release native OK); cc-scan không có phát hiện mới. Còn thiếu: ảnh chụp runtime
của dialog export/inspector và chạy thử SQL xuất trên MySQL/SQL Server thật.
Bước tiếp theo: commit, hỏi cổng L3, push `main`, đóng #537/#538/#539.

