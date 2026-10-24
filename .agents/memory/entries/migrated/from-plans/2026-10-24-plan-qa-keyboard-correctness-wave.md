---
schema_version: 1
id: "MEM-PLAN-qa-keyboard-correctness-wave-900cf8397e84"
title: "Plan archive: Qa Keyboard Correctness Wave"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, qa-keyboard-correctness-wave]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "900cf8397e842fe406eb0823ac4f6e01341c7b167839614fbdc82754979b3ca3"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/qa-keyboard-correctness-wave/PLAN.md"
  - "docs/plans/active/qa-keyboard-correctness-wave/FINDINGS.md"
  - "docs/plans/active/qa-keyboard-correctness-wave/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/qa-keyboard-correctness-wave/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# QA Keyboard Operability and Correctness Wave

State: REVIEW. Fixes, focused tests, workspace gates and hidden native runtime evidence are
recorded in `VERIFICATION.md`; changes remain uncommitted on `main` and await independent review.

Baseline revision: `6dc484af6f0665c08d94b82141bf23ece6da6b6c` (2026-10-21, `main`).

## Goal

Act on the QA audit findings confirmed in this session, with priority on the owner's report that
keyboard control "almost does not work": restore shell-wide Tab traversal, keep global chords alive
while the SQL editor is focused, and make keyboard-focusable custom controls visibly focused.
Alongside keyboard work, fix two confirmed P1 correctness defects (stale schema-diff completion and
SQLite DDL fragment injection) and the confirmed P2 polish items.

## Scope

In scope:

- `crates/app/src/shell/chrome/shell_topbar_view.rs` — window-drag hit target must not be a focus stop.
- `crates/app/src/query_workbench/events_query_dispatch.rs` — app-owned editor focus must not suppress global chords.
- `crates/app/src/files/files_surface_view.rs`, `crates/app/src/tools/tools_hub_surface_view.rs` — visible focus ring on custom clickable cards/tabs.
- `crates/app/src/runtime/runtime_event_handlers.rs`, `crates/app/src/app/mod.rs` — stale schema reply must not complete or leave pending a newer diff request.
- `crates/core/src/application/migration_planner.rs` — untrusted SQLite declared type/default fragments must never reach emitted DDL.
- P2 items: missing `table.no_dependencies` locale key in all five catalogs, PostgreSQL-only COPY export gating, table-switch grid view reset, SQLite EXPLAIN keyword parsing, Quick Open duplicate action rows.

Out of scope / provider impact:

- PostgreSQL: N/A for the two keyboard defects; the migration-planner validation applies to all
  providers but the injection vector was SQLite `PRAGMA table_info` metadata.
- SQLite: DDL validation covered by new unit tests; no live SQLite run is claimed.
- Remaining QA P2s (English hard-coding in tools hub/files tabs/scope chips, palette clipping,
  export TOCTOU/symlink/non-UTF8, fmt gate blind to `include!` modules,
  `parse_sqlite_explain_json` complexity) are recorded in `FINDINGS.md` but not fixed here.

## Design

- Focus contract: Tab/Shift+Tab traverse focusable widgets in registration order. A widget is
  focusable only if it can be activated from the keyboard and is painted with
  `paint_focus_ring` (`crate::components::interact`). The topbar drag hit target therefore uses
  `Sense::CLICK | Sense::DRAG` (no `FOCUSABLE`).
- Chord contract: only egui `TextEdit` surfaces swallow global shortcuts. The custom `SqlEditor`
  owns its own key handling; holding focus there must not disable palette/sidebar/new-connection
  chords.
- Diff-request contract: a schema-loaded event completes `pending_diff_after_refresh` only when
  `schema_request` matches its `RequestId`; a matching failure (or worker-unavailable dispatch)
  cancels the pending diff instead of leaving it armed.
- DDL safety contract: `column_clause` validates declared type and default with
  `safe_sql_fragment` (quote/bracket/paren scanning, top-level `,`/`;`/comment/newline rejection,
  128/512 byte caps). Unsupported operations carry an empty SQL string even if a caller ignores
  `provider_supported`.

## Acceptance criteria

- Regression tests: `tab_focus_traverses_topbar_into_the_workspace`,
  `global_chords_fire_while_the_sql_editor_is_focused`, 4 runtime diff tests, 4 migration planner
  safety tests, plus the P2 tests — all pass.
- `cargo +nightly fmt --all -- --check`, `cargo check --workspace`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
  `cargo build --release --locked -p db-pro-native` executed and recorded.
- Hidden native runtime evidence (Xvfb, no real-desktop window) for focus rings, Quick Open,
  SQLite export dialog and dependency empty state at 1280×800 / 1440×900 / 1920×1080, light + dark.

## Plan files

- `CHECKLIST.md` — ordered implementation and verification steps.
- `FINDINGS.md` — evidence-backed P0/P1/P2 findings and dispositions.
- `VERIFICATION.md` — commands actually executed, outputs, and runtime evidence actually collected.

## Tổng kết bằng tiếng Việt

Kế hoạch này xử lý các lỗi QA đã xác nhận, ưu tiên điều khiển bằng bàn phím: Tab bị kẹt ở nhãn
topbar và toàn bộ phím tắt toàn cục bị vô hiệu khi con trỏ ở SQL editor. Đồng thời sửa hai lỗi P1
đúng đắn (phản hồi schema cũ hoàn tất diff mới; chèn SQL qua kiểu cột mặc định của SQLite) và các
P2 đã xác nhận. Trạng thái hiện tại là IMPLEMENTING — bằng chứng runtime native sẽ được ghi riêng
trong `VERIFICATION.md`.


## Source: `FINDINGS.md`

# Findings

Baseline: `6dc484af6f0665c08d94b82141bf23ece6da6b6c`. Evidence is source/test evidence unless a
runtime capture is explicitly cited. No live-provider claim is made here.

## P1 — Keyboard

### P1-K1 Tab traversal trapped on the topbar connection label

- Evidence: `shell_topbar_view.rs::handle_window_drag` upgraded the label `Response` with
  `Sense::click_and_drag()`, which includes `FOCUSABLE`, making the non-activatable label a focus
  stop that swallowed subsequent Tab presses.
- Failure: pressing Tab from the shell topbar cycles into the connection-name drag target and stops;
  the rest of the shell is unreachable by keyboard.
- Fix: `Sense::CLICK | Sense::DRAG` (non-focusable). The click/double-click drag path is unchanged.
- Regression: `tab_focus_traverses_topbar_into_the_workspace` (>= 10 distinct focus stops, no stall,
  reaches workspace content).

### P1-K2 Global chords dead while the SQL editor is focused

- Evidence: `events_query_dispatch.rs` computed
  `text_input_has_focus = egui_text_focused || self.query.editor.query_editor_focused`, so the
  app-owned `SqlEditor` focus was treated like a raw egui `TextEdit`, suppressing every global chord.
- Failure: after clicking into the SQL editor, palette/sidebar/new-connection shortcuts stop working.
- Fix: chord suppression now keys only on real egui text input focus.
- Regression: `global_chords_fire_while_the_sql_editor_is_focused` (mod+p opens Quick Open with the
  editor stand-in focused).
- Runtime evidence: `captures/keyboard-editor-quick-open.png` and
  `keyboard-editor-quick-open-filtered.png` (palette opens and filters with the editor focused).

### P1-K4 Chord events missed when key-up lands in the same frame

- Evidence: `settings_model::match_shortcut_token` matched `input.key_pressed(key)` against
  `input.modifiers`, i.e. the frame-final modifier state. A fast tap delivers
  `Key{pressed, ctrl}` + `Key{released}` + `ModifiersChanged(NONE)` in one frame, so the key was down
  but the modifier state had already collapsed to `NONE`.
- Failure: chords are timing-dependent — they intermittently do nothing under load or fast tapping
  (reproduced reliably with `xdotool` while the SQL editor held focus: `Ctrl+P` was a no-op and the
  typed filter text landed in the SQL buffer).
- Fix: match the modifiers carried by the key event (`input.events.iter().any(event_matches_shortcut)`).
- Regression: `chord_matches_when_key_release_lands_in_the_same_frame`.
- Runtime evidence: `captures/keyboard-editor-new-query.png` (Ctrl+T created and focused Query 2
  while the editor had focus) plus the P1-K2 captures above.

### P2-K3 Missing visible focus on keyboard-activatable custom controls

- Evidence: files tab strip and tools hub cards use `Sense::click()` (focusable; Space/Enter produce
  a fake primary click) but painted no focus indicator.
- Fix: paint `paint_focus_ring` when `response.has_focus()`.
- Runtime evidence: `captures/tools-focus*.png` and `captures/files-tab-focus*.png` (6 per surface,
  dark+light at 1280×800 / 1440×900 / 1920×1080); rings visible in inspected crops.

## P1 — Correctness

### P1-A Stale schema reply can complete or leave armed a newer diff request

- Evidence: `on_schema_loaded` set `pending_diff_after_refresh = false` for any reply, and
  `handle_schema_request_failure` did not cancel the pending diff; worker-unavailable dispatch in
  `app/mod.rs` also left it armed.
- Failure: a diff requested after a refresh can be consumed by an older in-flight schema reply (wrong
  or empty comparison), or stays pending forever after a failed refresh.
- Fix: gate on `schema.explorer.schema_request == Some(request_id)`; cancel pending diff on matching
  failure and on dispatch failure.
- Regression: four `runtime_event_handlers::tests` cases (stale reply no-op, matching reply completes
  once, unrelated failure keeps pending, matching failure cancels).

### P1-B SQLite metadata can inject SQL into migration DDL

- Evidence: SQLite `PRAGMA table_info` returns declared type and default verbatim; the migration
  planner interpolated them into `CREATE TABLE`/`ADD COLUMN`/`ALTER … TYPE` text.
- Failure: a table created with e.g. `INTEGER); DROP TABLE users; --` produces attacker-controlled
  DDL in the migration preview/apply path.
- Fix: `column_clause` and `add_type_mismatches` validate fragments with `safe_sql_fragment`
  (quote/bracket/paren scanning, top-level comma/semicolon/comment/newline rejection, length caps);
  unsupported operations carry an empty SQL string and an explicit reason.
- Regression: `sqlite_declared_type_cannot_smuggle_a_second_statement`,
  `sqlite_default_cannot_smuggle_a_second_statement`,
  `type_parameterized_names_and_quoted_defaults_remain_supported`,
  `unsafe_source_type_is_not_emitted_for_alter_column`.

## P2 — Fixed

| ID | Finding | Fix |
|---|---|---|
| P2-1 | `table.no_dependencies` missing from all locale catalogs; dependency empty state untranslated | Added to en/vi/ja/ko/zh |
| P2-2 | Export dialog offered PostgreSQL-only `COPY … FROM stdin` script on SQLite | `ExportFormat::offered(postgres)` + coercion to CSV |
| P2-3 | Table switch kept previous grid filter/sort/manual row identity | `reset_for_table_switch()` called on open paths |
| P2-4 | SQLite EXPLAIN parsed `SCAN CONSTANT ROW`/`USE TEMP B-TREE` keywords as relations | Token-aware `sqlite_node` |
| P2-5 | Quick Open listed duplicate Agent/ER diagram actions | Dedupe by action, preserving group order |
| P2-6 | No visible focus ring on keyboard-activatable files tabs/tools cards | `paint_focus_ring` on focus; ring confirmed in dark+light captures at all three sizes |

## Confirmed but deferred (not fixed in this wave)

- Hard-coded English in tools hub, files tabs and scope chips.
- Quick Open label clipping at narrow widths.
- Export TOCTOU/symlink/non-UTF8 path hardening.
- fmt gate blind to `include!` modules.
- `parse_sqlite_explain_json` complexity.


## Source: `VERIFICATION.md`

# Verification

Baseline: `6dc484af6f0665c08d94b82141bf23ece6da6b6c`, working tree changes uncommitted. This file
records commands actually executed and evidence actually observed. Anything not run is marked
NOT RUN.

## Focused tests (executed)

| Command | Result |
|---|---|
| `cargo test -p db_pro_core --lib -- migration_planner` | 10 passed / 0 failed |
| `cargo test -p db_pro_core --lib -- explain_plan` | 11 passed / 0 failed |
| `cargo test -p db-pro-app --lib -- runtime_event_handlers::tests` | 4 passed / 0 failed |
| `cargo test -p db-pro-app --lib -- table_switch_clears_client_side_view_state` | 1 passed / 0 failed |
| `cargo test -p db-pro-app --lib -- copy_format_is_only_offered_for_postgres` | 1 passed / 0 failed |
| `cargo test -p db-pro-app --lib -- quick_open_lists_each_action_once` | 1 passed / 0 failed |
| `cargo test -p db-pro-app --lib -- tab_focus_traverses_topbar_into_the_workspace` | 1 passed / 0 failed |
| `cargo test -p db-pro-app --lib -- global_chords_fire_while_the_sql_editor_is_focused` | 1 passed / 0 failed |
| `cargo test -p db-pro-app --lib -- chord_matches_when_key_release_lands_in_the_same_frame` | 1 passed / 0 failed (added after the runtime gap below) |

## Workspace gates (executed 2026-10-21, final working tree)

| Command | Result |
|---|---|
| `cargo +nightly fmt --all -- --check` | PASS — exit 0, no diff |
| `cargo check --workspace` | PASS — `Finished dev profile … in 11.39s` |
| `cargo check -p db-pro-native --features capture` | PASS — `Finished … in 0.99s` |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS — exit 0, `Finished … in 14.41s` |
| `cargo test --workspace` | PASS — 27 suites: **2,154 passed / 0 failed / 56 ignored** |
| `cargo build --release --locked -p db-pro-native` | PASS — exit 0, `Finished release profile … in 1m 37s` |
| `bash scripts/check-ui-architecture.sh` | PASS — `UI architecture boundary: PASS` |
| `python3 tools/cc-scan.py . --fail-on error` | See note — 19 inherited errors, 0 new in changed files |
| `cargo test -p db_pro_core --lib` / focused suites | PASS (see focused table) |

The 56 ignored tests are the known live-provider (`pg_integration`) and SSH fixture cases; they
were not run and are not counted as passing.

### Clean-code scan (honest result)

`cc-scan` reports 19 error-level findings repo-wide because `.clean-code-baseline.json` is empty;
those findings exist at the baseline revision as well. Scanning only the changed Rust files gives
**3 errors before and 3 errors after** (same findings: `query_dialogs_view::export_result_to_disk`
and two in `explain_plan::parse_sqlite_explain_json`), so this wave adds **no new error-level
finding**. Two new warning-level heuristics remain and are reviewed, not auto-fixed:
`capture::open_workspace_capture` is a flat 15-branch env dispatch (the Tools/Files cases were moved
into a named helper; hard limit is 16) and `migration_planner::safe_sql_fragment` is an 11-branch
linear scanner (nesting reduced from 7 to 2 by extracting `skip_quoted`). Regenerating the baseline
to silence the inherited errors was deliberately not done.

## Runtime evidence (hidden Xvfb, capture feature)

Setup: `xvfb-run -a -s "-screen 0 2000x1200x24"` with `LIBGL_ALWAYS_SOFTWARE=1`,
`WINIT_UNIX_BACKEND=x11`, unset `WAYLAND_DISPLAY`; debug binary built with `--features capture`;
`DB_PRO_CAPTURE_SETTLE_FRAMES=12`; no window ever appeared on the real desktop. Because egui drops
widget focus while the viewport is unfocused, the focus captures needed the X window focused
(`xdotool windowfocus`) before the screenshot; the runner retries until the window is viewable and
verifies `xdotool getwindowfocus` equals the app window. All 12 focus runs reported
`focus=<wid> actual=<wid>`. After the scanner-driven capture refactor, the debug binary was
rebuilt with `--features capture` and the 1920×1080 Tools and Files focus captures were re-taken
through the new helper; both rendered byte-identical PNGs.

48 PNGs in `captures/` (33 matrix captures + 15 keyboard captures); every matrix file's framebuffer
size was checked with `magick identify` and matches its name exactly (no host height clamping):

| Surface | Theme(s) | Sizes | Count |
|---|---|---|---|
| Quick Open (Schema scope, filtered) | dark, light | 1280×800, 1440×900, 1920×1080 | 6 |
| Quick Open (`agent` query, Schema/Commands/All scopes) | dark | same | 9 |
| Table dependencies empty state | dark | same | 3 |
| SQLite export dialog | dark | same | 3 |
| Tools hub card focus ring | dark, light | same | 6 |
| Files tab focus ring | dark, light | same | 6 |
| Keyboard interaction (Tab, Ctrl+P, Ctrl+T) | dark | 1440×900 | 15 |

Observations from inspected captures:

- Tools hub: bright focus ring drawn around the focused "Schema & Data Compare" card in both
  themes at all three sizes (`tools-focus*.png`).
- Files panel: focus ring drawn around the focused `Search` tab in both themes at all three sizes
  (`files-tab-focus*.png`), with the fixture folder `/tmp/dbpro-evidence-workspace` loaded.
- Quick Open lists each action once; the previously duplicated "Open Agent" row is gone. "Agent"
  and "Ask Agent" remain as distinct actions, so an `agent` query in the `all` scope returns both.
- SQLite export dialog offers the CSV-family formats and SQL script, not the PostgreSQL-only
  `COPY … FROM stdin` script (`export-sqlite*.png`).
- Dependencies empty state renders the localized title and the new `table.no_dependencies_hint`
  body (`dependencies-empty*.png`).

Keyboard interaction (Xvfb + `xdotool`, debug build, 1440×900, 15 PNGs):

- Focus is real, not just painted: `xdotool getwindowfocus` equals the app window in every run; Tab×3/6/9 moves a visible focus ring across the window control, plugin/settings button and topbar search (`keyboard-tab-*.png`).
- Quick Open: `Ctrl+P` opens the palette, typing `agent` filters it to Agent / Ask Agent (dedupe verified), Down+Enter activates the row, `Ctrl+P` reopens and Escape closes it (`keyboard-quick-open*.png`).
- Editor-focused chords (P1-K2/P1-K4): Query 1 is opened by clicking its tab; the custom SQL editor is focused by clicking inside it and typing `select 1` (caret and dirty dot visible in `keyboard-editor-typed.png`). With that focus, `Ctrl+P` opens Quick Open and typing `agent` filters it (`keyboard-editor-quick-open*.png`), Escape returns to the editor, and `Ctrl+T` creates and focuses Query 2 (`keyboard-editor-new-query.png`).
- The first editor-focused run failed: `Ctrl+P` did nothing and `agent` landed in the SQL buffer. Temporary instrumentation showed `text_edit_focused=false`, no blocking overlay, and resolved tokens `mod+p`/`mod+t`, but `input.modifiers=NONE` at handler time while the Key event still carried `ctrl` — the frame-final modifier state had already collapsed. The chord matcher now reads event modifiers (P1-K4), the regression test was added, and the keyboard captures were re-taken with the rebuilt binary. Every keyboard capture cited above is from the post-fix run.

## Not verified / limitations

- No run on a real desktop with a physical keyboard; all keyboard/pointer evidence is Xvfb +
  `xdotool`, and widget focus requires the viewport to hold OS focus.
- The ignored 56 tests were not executed (they need live PostgreSQL/SSH fixtures); no new
  live-provider claim is made.
- COPY gating on PostgreSQL is covered by unit tests only; the runtime export capture was taken
  with the SQLite sample connection active.
- Independent review of this wave has not been performed.

## Tổng kết bằng tiếng Việt

Các cổng kiểm tra thực tế đã chạy và đều đạt: fmt, check, clippy `-D warnings`, toàn bộ workspace
test và build release `db-pro-native`. Bằng chứng runtime ẩn trên Xvfb gồm 48 ảnh PNG: 33 ảnh ma
trận đúng kích thước 1280×800 / 1440×900 / 1920×1080 (focus ring của thẻ Tools và tab Files hiển thị
rõ ở cả nền sáng/tối, Quick Open không còn dòng trùng, hộp thoại export SQLite không còn tuỳ chọn
COPY, trạng thái rỗng Dependencies đã được bản địa hoá) và 15 ảnh bàn phím. Kiểm thử bàn phím phát
hiện thêm lỗi P1-K4: phím tắt phụ thuộc thời điểm vì bộ so khớp đọc trạng thái modifier cuối frame;
đã sửa sang modifier của chính sự kiện phím, thêm test hồi quy và chụp lại toàn bộ ảnh sau khi sửa.
Với SQL editor đang focus, Ctrl+P mở Quick Open và lọc được, Ctrl+T tạo Query 2. Giới hạn: chưa thử
trên desktop thật, chưa chạy 56 test cần fixture provider, và chưa có review độc lập.

