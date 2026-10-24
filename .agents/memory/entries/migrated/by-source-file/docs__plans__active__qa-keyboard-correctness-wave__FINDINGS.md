---
schema_version: 1
id: "MEM-SRC-28e9cfcaa1ef7835"
title: "Source mirror: docs/plans/active/qa-keyboard-correctness-wave/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/qa-keyboard-correctness-wave/FINDINGS.md"
source_sha256: "28e9cfcaa1ef7835e06389a66be4c440d1bfb3c31ee91df2324cf0388ac014af"
mirror_body_sha256: "28e9cfcaa1ef7835e06389a66be4c440d1bfb3c31ee91df2324cf0388ac014af"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
