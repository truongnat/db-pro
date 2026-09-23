# Sidebar header actions + delete-session preserve

## Goal

1. **UI (P2):** Sidebar header connection name and search icon must afford different actions.
2. **Correctness (P1):** Deleting a non-active connection must not tear down / reload the active connected session.

## Scope

- `crates/ui/src/sidebar_view.rs` — wire name → connection switcher, search → command palette
- `crates/ui/src/workspace_actions.rs` — scoped palette open helper
- `crates/ui/src/events.rs` — conditional session reset on `connection.deleted`
- Unit test covering delete-non-active vs delete-active

## Non-goals

- New palette mode enum
- Redesign of explorer tree / filter toolbar
- Changing delete confirmation dialog UX

## Acceptance

- Clicking header name opens palette scoped to **Connections**
- Clicking search opens palette scoped to **All** / Commands
- Delete connection B while A is active+connected leaves `active_connection_id=A`, `connected=true`, no auto-reconnect of A
- Delete active connection still clears session and allows auto-select of remaining list

## Provider matrix

| Provider | Impact | Evidence |
|---|---|---|
| PostgreSQL | session lifecycle only | automated unit |
| SQLite | session lifecycle only | automated unit |
