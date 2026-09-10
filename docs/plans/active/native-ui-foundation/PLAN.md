# Native UI Foundation

## Status

`IMPLEMENTING`

## Goal

Create the first runnable egui/eframe native base without changing the existing React/Tauri application yet.

## Scope

- Add `db-pro-ui` library crate.
- Add `db-pro-native` binary crate.
- Establish DB Pro visual tokens and dark-first theme.
- Implement the first shell: activity bar, sidebar, topbar, statusbar, workspace tabs, welcome view, query mock surface and agent preview panel.
- Keep all backend integration out of this slice; use explicit UI-owned placeholder state.

## Out of scope

- Database commands and service facade extraction.
- Native SQL editor parity with Monaco.
- Production grid, schema explorer data, ER diagram and packaging.
- Removing React/Tauri.

## Next slice

Extract shared backend facade and replace the mock connection/query actions with typed task-bridge commands.
