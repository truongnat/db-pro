---
schema_version: 1
id: "MEM-PLAN-sidebar-dbeaver-codex-layout-b350694613a4"
title: "Plan archive: Sidebar Dbeaver Codex Layout"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, sidebar-dbeaver-codex-layout]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "b350694613a4aa2a2f3b8065fd793ced6b5147806f43677419427f5a893f94a9"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/sidebar-dbeaver-codex-layout/PLAN.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/sidebar-dbeaver-codex-layout/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Sidebar DBeaver + Codex Redesign

**Status:** IMPLEMENTING  
**Branch:** `feature/sidebar-dbeaver-codex-layout`  
**P-level:** P2 (UX improvement, non-breaking)

## Goal

Redesign the Explorer sidebar panel from a single flat list into three
vertically-stacked resizable sub-panes, inspired by DBeaver's connection
navigator and Codex's minimal icon rail:

```
┌──────┬────────────────────────┐
│      │ CONNECTIONS        [+] │
│ Icon │ ● production-pg (PG)  │  ~fixed / collapsible
│ Rail │ ○ localhost-dev (SQ)  │
│      ├────────────────────────┤
│ 48px │ SCHEMAS            [↓] │  resizable
│      │ [public ▾]  analytics  │
│      ├────────────────────────┤
│      │ ▷ Filter tables…       │  fills rest
│      │  ⊞ users (selected)   │
│      │  ⊞ orders             │
│      │  ⊞ products           │
│      │  ▷ VIEWS          (3) │
│      │  ▷ FUNCTIONS      (2) │
│      │ 4/127  show all ›      │
└──────┴────────────────────────┘
```

## Changes

### `crates/ui/src/app_state.rs` (AppState fields)
- Add `connections_pane_height: f32` — persisted
- Add `schemas_pane_height: f32` — persisted

### `crates/ui/src/app.rs`
- Persist new pane heights in `save()`
- Restore from storage in `new()`

### `crates/ui/src/navigation_view.rs` — `draw_sidebar()`
- Replace single-panel content with three vertically stacked sub-panels
  using `TopBottomPanel` inside the sidebar `SidePanel`, or manual
  vertical split with separator drag handles.

### `crates/ui/src/explorer_view.rs` — split into three draw methods
- `draw_connections_pane(ui)` — connection list (currently inside `draw_explorer_connections`)
- `draw_schemas_pane(ui)` — schema selector pills (currently inline in explorer)
- `draw_objects_pane(ui)` — search + tables + views/triggers/functions tree

## Design decisions

- Use egui `TopBottomPanel` ids namespaced under the sidebar for sub-panels.
  This gives native drag-to-resize at no cost.  
- Connections pane: collapsing arrow header (DBeaver-style) — always visible.
- Schema pane: pill buttons for quick switch, dropdown for overflow.
- Objects pane: existing search + existing tree, footer count label.
- Activity bar: unchanged (already Codex-like).

## Non-goals / out of scope
- Multi-connection simultaneous tree (future)
- Schema-level object counts from backend (future — currently just "N tables")

