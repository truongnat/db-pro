---
schema_version: 1
id: "MEM-SRC-e8289d781c8afb2a"
title: "Source mirror: docs/plans/active/qa-keyboard-correctness-wave/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/qa-keyboard-correctness-wave/PLAN.md"
source_sha256: "e8289d781c8afb2aee8e2820e96b85d3f4e7f6e6225244ecaa08d3961a8824ae"
mirror_body_sha256: "e8289d781c8afb2aee8e2820e96b85d3f4e7f6e6225244ecaa08d3961a8824ae"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
