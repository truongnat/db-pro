# DB Client — Frontend Architecture

Status: ratified; aligned with `docs/08-technology-decisions.md` and `docs/09-architecture-decisions.md`

## Stack

- React + TypeScript + Vite
- source-owned shadcn/ui-style components
- Radix UI primitives for accessible behavior
- Tailwind CSS and CSS variables for DB Pro visual tokens
- Monaco Editor for SQL editing
- TanStack Query for command/query lifecycle and cache invalidation
- Zustand for local UI preferences only
- TanStack Router for typed navigation

MUI is not part of the baseline. The UI must follow the approved DB Pro prototype: dark-first, compact, keyboard-first, database-client focused, and visually inspired by modern Codex/IDE surfaces without copying product branding.

## Component ownership

Components live in `src/ui` and are owned by the product. Radix provides primitives such as dialog, popover, dropdown, tooltip, tabs, and focus management. Styling, tokens, density, states, and composition remain DB Pro code.

Feature modules consume these components and must not create one-off visual variants without adding a reusable primitive or documented feature variant.

## State boundary

- TanStack Query: asynchronous command results, schema cache, connection status, query history.
- Zustand: active tab, panel sizes, theme, editor preferences, filters, and local settings.
- Monaco state: editor content and selection, persisted through feature services when saved.
- Tauri `Channel<T>`: streamed query batches; do not put row streams into a global store.

## UI states required

Every data-facing component defines loading, empty, error, disabled, focused, selected, editing, and permission/restricted states. Destructive actions require a confirmation surface showing the connection, target, and SQL classification.

## Module layout

```text
src/
  app/
  ui/
    tokens/
    primitives/
    overlays/
    navigation/
    data-display/
    editor/
  modules/
    connection/
    query/
    schema/
    data-grid/
    export/
  services/
  stores/
```
