---
schema_version: 1
id: "MEM-SRC-4929ad0b4b1bdcca"
title: "Source mirror: docs/plans/active/qa-keyboard-correctness-wave/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/qa-keyboard-correctness-wave/VERIFICATION.md"
source_sha256: "4929ad0b4b1bdcca96f35098e46e69b0d7892dc1726517938ddab842374f91f0"
mirror_body_sha256: "4929ad0b4b1bdcca96f35098e46e69b0d7892dc1726517938ddab842374f91f0"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
