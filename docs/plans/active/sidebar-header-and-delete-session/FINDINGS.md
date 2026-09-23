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

