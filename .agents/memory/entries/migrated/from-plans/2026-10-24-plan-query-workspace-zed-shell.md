---
schema_version: 1
id: "MEM-PLAN-query-workspace-zed-shell-45d5022b83b2"
title: "Plan archive: Query Workspace Zed Shell"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, query-workspace-zed-shell]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "45d5022b83b23103927b213e0fd174b11a17357a3eea1ec8ab507b4e61fc382c"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/completed/query-workspace-zed-shell/PLAN.md"
  - "docs/plans/completed/query-workspace-zed-shell/FINDINGS.md"
  - "docs/plans/completed/query-workspace-zed-shell/VERIFICATION.md"
  - "docs/plans/completed/query-workspace-zed-shell/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/completed/query-workspace-zed-shell/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Query workspace Zed-like shell (UI05)

State: COMPLETED

## Goal

Make the Query workspace feel editor-first like Zed: minimal chrome, no form
header, output as a dock, overlays for find/completion, contextual status.

Advances closed issue #291 (UI05 Query Workspace).

## Scope

Layout / interaction / UI composition only:

- `crates/ui/src/query_view.rs` — composition, context chips, status strip, dock
- `crates/ui/src/query_editor_panel.rs` — editor fill height; find overlay
- `crates/ui/src/events_query.rs` — open dock on result/error
- `crates/ui/src/app.rs` / `app_state.rs` — dock/params presentation state
- `crates/ui/src/navigation_view.rs` — avoid duplicate shell output on Query tab

## Non-goals

- SQL editor engine rewrite
- Query execution / provider architecture changes
- Explorer / Files / ER / Agent / Schema redesign
- Pixel-perfect Zed clone

## Architecture

Keep `QueryDocument` → `UiCommand` → runtime → `UiEvent` unchanged.
Add only presentation state: dock open/height/maximized, params panel open,
context picker open.

## Provider matrix

| Surface | PostgreSQL | SQLite |
|---|---|---|
| Layout / chrome | n/a (UI) | n/a (UI) |
| Run / Stop / txn / params | unchanged | unchanged |

## Acceptance

- New untitled query: editor + thin context + status; no permanent Result pane
- Connection/schema are caption chips, not permanent ComboBoxes
- Run/Stop primary; secondary actions in More
- Output dock opens on result/error; close returns height to editor
- Floating completion only (no inline completion card)
- Find is an editor overlay (not full-width row)
- Parameters / transaction chrome only when relevant
- Runtime screenshots (dark required; light sanity) at 1280×800 / 1440×900


## Source: `FINDINGS.md`

# Findings — query-workspace-zed-shell

## Baseline (pre-change)

`draw_query` stacks: header (ComboBoxes + Run + Builder + More) → transaction →
search row → builder collapsing header → editor → floating completion **and**
inline completion card → snippets → diagnostics list → parameters panel →
output tabs → output pane.

Editor height reserves 140–240px for output even when empty.

Shell also has `draw_output_panel` (`bottom_panel_open`) — a second, thinner
output path that does not auto-open on query complete.

## Design decisions

1. Reuse `bottom_panel_open` / `bottom_panel_height` for the Query dock so
   persistence and statusbar toggle stay consistent.
2. Skip shell `draw_output_panel` while `WorkspaceTab::Query` to avoid duplicate
   chrome; Query draws the rich dock itself.
3. Keep floating completion; stop calling inline `draw_sql_completion`.
4. Run lives on the query status strip; More keeps secondary actions.

## Findings (runtime feedback)

- Completion popup rows overlapped: nested `right_to_left` detail inside `horizontal` painted on top of the label. Fixed with fixed-width `add_sized` columns + truncate/tooltip.
- Query status Run could disappear: same nested layout anti-pattern. Fixed with outer `right_to_left` keeping Run pinned.
- Editor stole Enter/Arrow/Page keys while completion open — skip those when `completion_open`.
- Dock close/maximize merged onto the same row as Results/Messages tabs.

## Theme / smoothness pass (goal)

- Flush editor: no rounded card, hairline only when focused; denser line rhythm (`1.48`).
- Dedicated editor tokens: gutter / current-line / selection / line-number colors.
- Dark buffer plane deeper (`#181818`); light buffer flush white with app.
- SQL syntax palette restrained (Zed-like) instead of raw UI accent clones.
- Completion: panel fill contrasts editor; selected row uses `accent_soft`; kind badges in soft pills.
- Context chip: outline, no heavy hover fill.
- Evidence: `docs/plans/completed/query-workspace-zed-shell/screenshots/theme-dark-1280x800.png`, `docs/plans/completed/query-workspace-zed-shell/screenshots/theme-light-1280x800.png`.

## Perf / lag (P0)

Idle Query tab felt continuous-refresh laggy. Root causes + fixes:

1. **Caret** — `request_repaint()` every frame while focused → `request_repaint_after` only for the next blink toggle (`renderer.rs`).
2. **Lint** — `refresh_diagnostics()` ran sqlparser every `draw_query` frame → cache on `(doc_index, buffer.version(), driver)` and only re-merge `execution_diagnostic` on hit.
3. **Status strip** — `discover_sql_parameters` every frame → cache on buffer version.
4. **Scroll width** — `max_line_len_chars` scanned every line/`chars().count()` every frame → cache on `TextBuffer` rebuild.
5. **Completion** — full `items.clone()` each popup frame → index-based draw; clone only the clicked item.

## IDE-master pass

Further snappiness + chrome density:

1. **Debounced lint** — 180ms quiet window before sqlparser while typing; rematch skipped when execution diagnostic fingerprint unchanged.
2. **Single tokenize** — `reanalyze` reuses `CachedSqlTokens` via `SqlDocumentAnalysis::from_tokens` (was double-tokenize per keystroke).
3. **Prediction default `Subtle`** — quieter ghost text / less idle AI pressure; egress note retained.
4. **Chrome** — drop UTF-8 + shell Ln/Col duplicate; context chip uses warning + chevron icons.
5. **Tab indicator** — `request_repaint_after(16ms)` while animating instead of unbounded `request_repaint()`.


## Source: `VERIFICATION.md`

# Verification — query-workspace-zed-shell

## Quality gates

| Gate | Result | Notes |
|---|---|---|
| `cargo fmt` (touched files) | PASS | |
| `cargo check -p db-pro-ui` | PASS | |
| `cargo clippy -p db-pro-ui --all-targets -- -D warnings` | PASS | |
| `cargo test -p db-pro-ui --lib` | PASS | 523 passed |

## Runtime evidence (required)

Capture at 1280×800 and 1440×900 (dark required; light sanity). One 1920×1080 sanity.

| Shot | State | Captured |
|---|---|---|
| A | New untitled query | CAPTURED · light `screenshots/A-untitled-1280x800.png`, `screenshots/A-untitled-1440x900.png`; dark `screenshots/A-untitled-dark-1280x800.png` |
| B | File-backed SQL | pending |
| C | Completion open | pending |
| D | Find open | pending |
| E | Running query | pending |
| F | Result dock | pending |
| G | Multiple results | pending |
| H | Error / Messages | pending |
| I | Parameters | pending |
| J | Manual transaction open | pending |

## Provider

UI composition only — no new provider paths.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — query-workspace-zed-shell

## 1. Task

| Field | Value |
|---|---|
| Feature | Query Workspace Zed Shell (UI05) |
| Branch | `feature/query-workspace-zed-shell` |
| State | In Progress |
| Goal | Editor-first Query chrome: chips, status Run, dock closed until result |

## 2. Source evidence

Exact SHAs pending commit. Touched presentation surfaces:

- `crates/ui/src/query_view.rs` — composition rewrite
- `crates/ui/src/query_editor_panel.rs` — fill height + editor rect
- `crates/ui/src/events_query.rs` — open dock on result/error/explain
- `crates/ui/src/app.rs` / `app_state.rs` — picker/params/dock/max state
- `crates/ui/src/editor/renderer.rs` — `SqlEditorResponse.rect` for overlays

## 3. Gates

| Command | Result |
|---|---|
| `cargo check -p db-pro-ui` | PASS |
| `cargo clippy -p db-pro-ui --all-targets -- -D warnings` | PASS |
| `cargo test -p db-pro-ui --lib` | PASS · 523/0 |
| `cargo fmt` (touched files) | PASS |
| Runtime screenshots | pending |

## 4. Review

n/a (not yet in Review)

## 5. Risks / follow-ups

- Visual builder still collapsing-header (side split deferred)
- Full workspace fmt/check/clippy/test not re-run end-to-end this pass
- Capture harness still wired to other scenarios — Query shots pending

## 6. Tổng kết bằng tiếng Việt

Đã đảo composition Query: editor chiếm chiều cao khi chưa có result; ComboBox header → chip context; Run nằm status bar; output thành dock mở khi có kết quả/lỗi; find overlay; bỏ inline completion card. Còn runtime screenshot mới đóng được RUNTIME_VERIFY.

