---
schema_version: 1
id: "MEM-PLAN-sidebar-header-and-delete-sessio-33d11f78b824"
title: "Plan archive: Sidebar Header And Delete Session"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, sidebar-header-and-delete-session]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "33d11f78b8248cdfaa54f8bbf49bf3b2b370202d8507684cb43636e9f84006a9"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/sidebar-header-and-delete-session/PLAN.md"
  - "docs/plans/active/sidebar-header-and-delete-session/FINDINGS.md"
  - "docs/plans/active/sidebar-header-and-delete-session/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/sidebar-header-and-delete-session/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# Findings

## P1 — Delete any connection resets active session

**Evidence:** `events.rs` `on_operation_completed` for `connection.deleted` always sets `active_connection_id = None` and `connected = false`, then `request_connections_once` → `on_connections_loaded` auto-`connect_to_connection` which clears schema.

**Failure:** With A loaded and B deleted, A reconnects and re-introspects.

## P2 — Header name and search are duplicate entry points

**Evidence:** Both call `open_palette(PaletteMode::Commands)` with identical scope; tooltips disagree (switch vs search).

**Decision:** Merge into one launcher pill (name + search icon, single hit target) → full Command Palette. Keep Plus separate.

## P1 — Palette/dialog card opens under its dim overlay

**Evidence:** After hosting sidebar content in `Area(Order::Middle)`, clicking the merged header launcher left the dialog card under the Foreground dim (or competing Foreground chrome such as the resize grip).

**Decision:** Revert sidebar content to a constrained raw `Ui` (keep ScrollArea `max_height` for wheel). Pin dialog card as a **sublayer** of the dim Area and `move_to_top` both; move resize grip to `Order::PanelResizeLine`.



## Source: `VERIFICATION.md`

# Verification

## Automated

```text
cargo test -p db-pro-ui -- \
  deleting_non_active_connection_preserves_active_session \
  deleting_active_connection_clears_session \
  sidebar_header_launcher_opens_full_command_palette \
  deleting_sibling_connection_does_not_auto_reconnect_active
```

Result (2026-09-18): **4 passed / 0 failed**

**[CORRECTED 2026-10-13 — `_audit-2026-10-13` slice 04]** the recorded command above
listed `connection_switcher_opens_palette_scoped_to_connections`, a test that does not
exist anywhere in the repo (the merged-launcher design intentionally has no
connections-scoped palette — header name+search opens the full Command Palette instead).
The real four tests are `events.rs:138` (`deleting_non_active_connection_preserves_active_session`),
`events.rs:179` (`deleting_active_connection_clears_session`), `app_tests.rs:3662`
(`sidebar_header_launcher_opens_full_command_palette`), and `app_tests.rs:3674`
(`deleting_sibling_connection_does_not_auto_reconnect_active`). Re-run on 2026-10-13
(`cargo test -p db-pro-ui --lib -- <four names>`): **4 passed / 0 failed**.

## Runtime

- UI runtime evidence pending (manual: header name → Switch Connection; search → Command Palette; delete sibling while A connected)

## Provider

- n/a for SQL; session state is UI/runtime only

