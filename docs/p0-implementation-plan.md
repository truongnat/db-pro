# DB Pro P0 Implementation Plan (Agentic SDLC)

- Source of truth: `docs/mvp-dbeaver-gap-report.md`
- Workflow file: `.agents/workflows/dbpro-mvp-p0.md`
- Execution model: incremental slices, each slice must pass `npm run build` and `cargo check --manifest-path src-tauri/Cargo.toml`.

## Slice order

1. `OBS-001/OBS-005` Query history local + re-run from history
2. `NAV-007` Navigator search/filter
3. `SQL-008` SQL formatter command
4. `GRID-007/GRID-015` Copy from grid + export CSV
5. `UX-001/UX-004/UX-005` Keyboard/loading/error consistency hardening

## Current progress

- [x] Slice 1 complete:
  - Added local query history cache and append logic.
  - Added query history UI in workbench (load, run, clear per current connection).
  - Reset flow now clears query history together with panel cache.
  - Validation passed (`npm run build`, `cargo check`).
- [x] Slice 2 complete:
  - Added navigator filter box (schemas/objects/columns).
  - Added filtered counters and no-match state.
  - Added retry action in navigator error card.
- [x] Slice 3 complete:
  - Added SQL formatter (`sql-formatter`) with toolbar button.
  - Added keyboard shortcut (`Cmd/Ctrl+Shift+F`, `Alt+Shift+F` in editor keymap).
- [x] Slice 4 complete:
  - Added result grid copy actions (`Copy Page` + keyboard copy from table).
  - Added CSV export action from result grid.
- [x] Slice 5 complete:
  - Added global shortcuts (`F5` run, `Esc` cancel running query, format shortcut).
  - Added consistent workbench loading banner.
  - Added consistent dismissible error banner.

## Validation gate

Run after each slice:

```bash
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

## Notes

- `agentic-sdlc workflow setup` was executed in this repo and scaffolded `.agents/*`.
- `agentic-sdlc workflow check` passes with warnings only (no custom roles/templates/skills yet).
