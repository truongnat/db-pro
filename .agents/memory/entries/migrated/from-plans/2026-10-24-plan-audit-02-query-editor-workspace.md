---
schema_version: 1
id: "MEM-AUDIT-02-query-editor-workspac-3f933a18fe08"
title: "Plan archive: Audit 02 Query Editor Workspace"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, audit-02-query-editor-workspace]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "3f933a18fe0888041be8672df7035adda6b123831079dfe8aab514647a0cae90"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/_audit-2026-10-13/02-query-editor-workspace.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ audit slice `docs/plans/_audit-2026-10-13/02-query-editor-workspace.md`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

# Audit slice 02 — Query Editor / Query Workspace

Audited at HEAD `d73148cd` (main), working tree clean. Date: 2026-10-13.

## 1. Slice scope

Plans covered: `query-editor-intelligence`, `query-editor-hover-intelligence`,
`query-editor-zed-feel`, `query-workspace-zed-shell` (already in
`docs/plans/completed/`), `query-welcome-polish`, `welcome-query-editor-polish`,
`query-editor-large-schema-audit`, `ui-query-workspace`.

Code areas: `crates/ui/src/editor/*`, `crates/ui/src/query/*`,
`crates/ui/src/query_*.rs`, `crates/native-app/src/translate_{cmd,evt}.rs`,
`crates/runtime/src/api.rs`.

## 2. Per-plan verdict table

| Plan | Claimed state | Supported state | Verdict | Key evidence |
|---|---|---|---|---|
| query-editor-intelligence | All 49 code items done; live provider + viewport matrix pending | Code claims verified in tree; runtime evidence still absent | EVIDENCE_GAP | Prediction routing `translate_cmd.rs:686-713`; `StatementResultKind` routing `translate_evt.rs:313-337`; `saved_snapshot` dirty tracking `query_document.rs:100,233-237`; history retention 500 `query_history_events.rs:6`; collision-safe IDs `query_documents.rs:218-227`; 2 unchecked checklist items remain (live provider, UI evidence); `pg_integration.rs` has 29 `#[ignore]`d tests at HEAD |
| query-editor-hover-intelligence | Hover FSM + rich cards implemented; RUNTIME_VERIFY | Core claims verified; symbol occurrence highlight on confirmed hover NOT found | EVIDENCE_GAP | `HoverState` 500ms FSM `editor/hover.rs:13,31`; no `focused` guard `query_editor_surface_view.rs:423-435`; `interactable(true)` `query_editor_support.rs:110`; `rich_hover`/`rich_keyword_help` (~90 keyword arms) `query/intelligence.rs:252,659`; missing: no occurrence-highlight rects anywhere (only `rich_hover_open` used to suppress diag tooltip `renderer.rs:100,707`) |
| query-editor-zed-feel | Interaction policy, version-bound completion, caret/scroll done; runtime evidence pending | Code claims verified; runtime visual evidence still unchecked | EVIDENCE_GAP | `EditorInteractionPolicy` + tests `editor/interaction.rs:14-142`; `document_version`/`can_apply_to_version` `editor/completion.rs:36,112`; `scroll_caret_into_view` `renderer.rs:882-901`; blink via `request_repaint_after` `renderer.rs:864`; `Cmd+S→wants_save` `renderer.rs:1266`; signature help `query_editor_surface_view.rs:397-409`; 1 unchecked item (runtime visual/interaction evidence) |
| query-workspace-zed-shell (completed/) | COMPLETED; screenshots A–J required | Chrome verified in code; only 2 of ~10 required screenshot states on disk; VERIFICATION itself marks B–J "pending" | SUPPORTS (code) / thin completion evidence | `draw_context_chip` `query_context_view.rs:187-239`; Run/Stop strip `query_run_control_view.rs:23-74`; find overlay `query_search_view.rs:12-28`; dock opens on result/error `query_result_events.rs:125`, `query_failure_events.rs:98,113`; dock gating `query_view.rs:49-84`; `screenshots/` holds only `query-dark-1280x800.png` + `query-light-1280x800.png` (2 files) vs A–J matrix |
| query-welcome-polish | RUNTIME_VERIFY; Select Sm/Ghost + simplified Welcome | Code verified at HEAD; plan honestly records pending items | EVIDENCE_GAP | `Select` with `SelectSize::Sm`+`SelectVariant::Ghost` at 108px `query_context_view.rs:145-149`; simplified static Welcome `welcome_surface_view.rs:13-45` (no composer/connection cards/draft row); pending: interactive selection + exact 900/1080 logical heights (host caps at 838); evidence dir has 3 viewport PNGs + manifest |
| welcome-query-editor-polish | RUNTIME_VERIFY; toolbar Run/Explain/Format, Welcome cards, explorer filter gate | Toolbar/filter verified; Welcome surface has since been redesigned (no "Connections card"); claimed test `welcome_top_inset_*` no longer exists | STALE-DOC | Toolbar actions `query_context_view.rs:94,101`; explorer filter gated `explorer_surface_view.rs:53,83`; `welcome_surface_view.rs` at HEAD = icon/name/copy/providers + 4 quick-action cards (redesign chain `3ea9e249`..`c6a033da`); `welcome_top_inset` test absent (`grep` finds none) |
| query-editor-large-schema-audit | PLANNING; audit-only, 2 P1 + 3 P2 findings | Findings still accurate at HEAD for code state; 2 of 5 remediated by `01b3842a` | SUPPORTS (doc accurate; findings partially still open) | P1 UTF-8 panic still open `schema_completion.rs:1147-1152` (byte-decrement slicing, unguarded); P1 unbounded list FIXED `schema_completion.rs:1073-1079` (MAX 50) + virtualized popup `query_completion_popup_view.rs:97-115`; P2 result clone open `query_view.rs:270`; P2 sync index build open `schema_events.rs:60`; P2 quadratic runtime scan FIXED `runtime/api.rs:470+` (`group_columns_by_table`); statement-scoped analysis added `schema_completion.rs:87-93` |
| ui-query-workspace | All done; canonical Button migration | Verified: canonical `Button` in all query surfaces; no legacy helpers remain in `query_*.rs` | SUPPORTS | `Button::new` `query_run_control_view.rs:28-53`, `query_search_view.rs:77-214`, `query_snippets_surface_view.rs:19`, `query_output_tabs_view.rs:30-85`; zero `primary_button_with_icon`/`secondary_button*`/`compact_*` in `query_*.rs` (grep); non-regression include test covers `query_view.rs` `components/mod.rs:138`; runtime captures not in repo |

## 3. UI findings

- **P2** — Context chip is painted manually via `egui::Frame` + `interact(Sense::click)`,
  not the canonical `Button` component; `widget_info` compensates for accessibility, but this
  bypasses shared button styling/hover conventions — `query_context_view.rs:220-233`.
- **P2** — No hardcoded `Color32::from_rgb*` in audited query surfaces (editor, context chip,
  status bar, dock, search overlay, popup, hover card). Only `Color32::TRANSPARENT` uses
  (`query_context_view.rs:213`, `query_output_tabs_view.rs:131`), which are benign. Theme tokens
  respected (`theme.border_subtle`, `theme.warning_soft()`, `theme.editor_current_line_fill()`).
- **P2** — `welcome_surface_view.rs` top inset `((h-400)*0.4).max(SPACE_MD)` has no upper
  clamp; on very tall viewports the block drifts far down — cosmetic, supersedes the
  `clamp(24..96)` formula described by `welcome-query-editor-polish` (design has moved on).
- No unwrap/expect/panic found on data paths in audited editor/query view files (only inside
  `#[cfg(test)]` modules).

## 4. Feature findings

- **P1** — `extract_word_prefix` still slices `text[end-1..end]` with byte decrement:
  panics on UTF-8 boundaries (`SELECT tên`). Still open at HEAD —
  `query/schema_completion.rs:1147-1152` (unchanged since introduction in `82ac7884` per `git log -L`).
- **P2** — Whole `UiQueryResult` still cloned per output-pane frame:
  `query_view.rs:270` `active_result().cloned()`. Open.
- **P2** — `SchemaSymbolIndex::build(&schema)` still runs synchronously on the UI
  schema-load transition: `schema_events.rs:60`. Open.
- **P2** — Confirmed-hover symbol occurrence highlighting (hover-intelligence acceptance item)
  not implemented: `paint_highlights` covers current line, search, selection, delimiters only
  (`editor/renderer.rs:404-409`); `rich_hover_open` flag only suppresses the diagnostic tooltip.
- **P2** — Generation cost of unscoped completion still scans all matching columns before
  top-50 truncation (no early bound); output is bounded and popup is virtualized, so severity
  reduced vs original P1 — `schema_completion.rs:1073-1079`.
- **Resolved** — Blank results grid defect (FEATURE_AUDIT §5): FIXED by `294ab089`
  (PR #308) — `query_output_dock_surface_view.rs` now bounds the tab strip to
  `DOCK_TAB_STRIP_HEIGHT=34` and allocates `dock_height - chrome_height` to the pane
  (`draw_chrome`, `query_output_dock_surface_view.rs:29-56`). Grid no longer painted into inverted clip.
- **Resolved** — Quadratic schema summarization: `runtime/api.rs` `summarize_introspection`
  groups columns/FKs by table once.
- **Resolved** — Prediction routing to runtime worker: `translate_cmd.rs:686-713`.
- Test gap — `pg_integration.rs` now has 29 `#[ignore]`d tests (was 18 when plans were
  written); per AGENTS.md these do not count as passing and the "live PG" rows in
  `query-editor-intelligence` VERIFICATION remain unevidenced.

## 5. Captures

- Attempted: `DB_PRO_CAPTURE_TO=/tmp/audit-caps/query/query-workspace-dark-1280x800.png`
  `DB_PRO_WINDOW_SIZE=1280x800 DB_PRO_CAPTURE_QUERY=1`.
  - `./target/debug/db-pro-native` (built without `--features capture`): ran 103s, no PNG —
    the capture wrapper is compiled out in this binary.
  - `./target/release/db-pro-native` (with capture feature, built 2026-10-13): hung 300s,
    no PNG — the Screenshot viewport command never returned in this environment
    (capture.rs `CAPTURE_TIMEOUT_FRAMES=600` > my 300s job limit).
- Result: **0 captures produced**. Per assignment, not retried further. Runtime UI evidence
  remains the dominant missing artifact across all 8 plans.

## 6. Missing evidence to reach COMPLETED

- query-editor-intelligence: live AI provider request + stale/cancel evidence; 3-viewport native captures; live-PG execution (29 ignored tests must run against a fixture).
- query-editor-hover-intelligence: native capture showing delayed rich hover card + occurrence highlights (and implement the missing occurrence-highlight pass).
- query-editor-zed-feel: native visual/interaction evidence at 1280×800/1440×900/1920×1080 (typing, dot-completion, Cmd+Space, Cmd+S, scroll/caret).
- query-workspace-zed-shell: remaining capture matrix B–J (find open, completion, running, dock result/error, params, transaction) — currently 2 PNGs only.
- query-welcome-polish: interactive Select selection/keyboard verification + exact 900/1080 logical-height captures.
- welcome-query-editor-polish: refresh doc against HEAD (Welcome redesigned); re-run workspace suite + clippy (never run in that pass).
- query-editor-large-schema-audit: fix + regressions for UTF-8 panic; bound generation; fix result clone + sync index build; real company crash backtrace.
- ui-query-workspace: runtime captures of migrated surfaces; otherwise code-complete.

## 7. P0/P1/P2 totals

- P0: 0
- P1: 1 (UTF-8 completion panic, `schema_completion.rs:1147`)
- P2: 5 (result clone `query_view.rs:270`; sync index build `schema_events.rs:60`;
  missing occurrence highlight; unbounded generation scan; manual-paint context chip)

## 8. Tổng kết bằng tiếng Việt

- Cả 8 plan đều có mã nguồn hậu thuẫn phần lớn tuyên bố; không phát hiện plan nào "ảo" hoàn toàn, nhưng bằng chứng runtime (ảnh chụp + PG thật + provider AI thật) thiếu ở cả 8 → đa số verdict là EVIDENCE_GAP.
- Lỗi "results grid trắng" trong FEATURE_AUDIT §5 đã được sửa chính xác tại HEAD (commit `294ab089`, giới hạn chiều cao tab strip của dock).
- `query-editor-large-schema-audit` vẫn đúng: panic UTF-8 trong `extract_word_prefix` (P1) và 2 P2 (clone kết quả mỗi frame, build index đồng bộ trên UI) vẫn mở; P1 danh sách completion vô hạn và P2 quét metadata bậc hai đã được sửa bởi `01b3842a`.
- `welcome-query-editor-polish` là STALE-DOC: test `welcome_top_inset` không còn tồn tại và Welcome đã được thiết kế lại (không còn card "Connections" riêng).
- `query-workspace-zed-shell` được đánh dấu COMPLETED nhưng chỉ có 2/10 ảnh chụp bắt buộc — bằng chứng hoàn thành mỏng.
- Không có vi phạm màu cứng (hardcoded colors) hay unwrap trên đường dữ liệu trong các surface query đã audit; capture không chạy được trên host này (binary debug thiếu feature, bản release treo >300s).

