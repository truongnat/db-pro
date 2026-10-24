---
schema_version: 1
id: "MEM-SRC-6782e49c42c2a73a"
title: "Source mirror: docs/plans/active/macos-app-menu-refresh/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/macos-app-menu-refresh/PLAN.md"
source_sha256: "6782e49c42c2a73a4981310fecad156a09d865025a80ccad1433963de28de6b7"
mirror_body_sha256: "6782e49c42c2a73a4981310fecad156a09d865025a80ccad1433963de28de6b7"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# macOS app menu and schema refresh

State: RUNTIME_VERIFY. Baseline: `main@3dd988e2ff06ac71942c7ec331acf66389d3c5cb`.

## Goal

Provide a standard macOS application menu for DB Pro and expose the existing schema refresh plus an explicit cache-clearing refresh from the native menu. `Refresh` forces a fresh schema introspection. `Refresh Cache` invalidates the active connection's schema cache first, then introspects and updates the UI through the existing request/event lifecycle.

## Scope

- Add standard DB Pro, File, Edit, View, Window, and Help menus on macOS, with working New Connection, New Query, documentation, and standard window/edit actions.
- Route View → Refresh and View → Refresh Cache to the active connection.
- Return cache invalidation failures through the existing `UiEvent::Failed` path.
- Preserve normal eframe app behavior and the optional capture driver.

## Non-goals

- Changing the app's connection-selection rules, schema cache policy, or provider introspection.
- Adding unsupported menu commands or a custom help/documentation surface.
- Changing non-macOS window menus.

## Architecture and invariants

macOS uses `muda` for the system menu. Menu events are forwarded to the UI thread and invoke public `DbProApp` actions. The cache-refresh request remains typed across `UiCommand` and `RuntimeCommand`; the runtime invalidates the connection-scoped cache before forced introspection and emits either `SchemaLoaded` or `Failed`. The root menu remains alive for the native window lifetime. Existing UI refresh behavior remains unchanged.

## Acceptance

- [ ] Native macOS menu bar shows the standard app, File, Edit, View, and Window menus.
- [ ] View → Refresh forces schema introspection for the active connection.
- [ ] View → Refresh Cache invalidates that connection's schema cache, then refreshes it; failures surface to the UI.
- [x] No active connection produces a clear user-facing message and no runtime command (UI regression test passes).
- [x] Capture and non-capture builds retain their existing launch paths (capture check and release build pass).
- [x] Targeted tests and the native release build pass.
- [ ] Record live macOS menu interaction and independent SQLite/PostgreSQL runtime evidence; keep the plan active until then.
