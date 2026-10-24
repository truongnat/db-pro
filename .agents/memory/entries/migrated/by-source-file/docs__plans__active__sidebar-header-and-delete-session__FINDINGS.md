---
schema_version: 1
id: "MEM-SRC-ddc02809d61f0810"
title: "Source mirror: docs/plans/active/sidebar-header-and-delete-session/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/sidebar-header-and-delete-session/FINDINGS.md"
source_sha256: "ddc02809d61f08101b29231473b565595c743f0362117082eb544c9b43d9d473"
mirror_body_sha256: "ddc02809d61f08101b29231473b565595c743f0362117082eb544c9b43d9d473"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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

