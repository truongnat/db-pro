# Findings

## #489 — idle monitoring refresh

`request_monitoring_poll_if_due` ran only during Monitor drawing. With no repaint source active, an idle egui context could remain asleep past the five-second poll interval. `prepare_frame` now schedules a repaint for the remaining interval while Monitor is active, connected, and auto-refresh is enabled. The next draw dispatches the poll and resets the timestamp.

## #473 — stale monitoring/security responses

`UiEvent` already carried request IDs, but the event router discarded them; loaded events also lacked the originating connection ID. Monitoring snapshots/workload and security users/privileges/memberships now retain pending `(request_id, connection_id)` pairs, and successful response events are accepted only when both match and the connection remains active.
