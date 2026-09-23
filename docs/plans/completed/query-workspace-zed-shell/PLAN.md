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
