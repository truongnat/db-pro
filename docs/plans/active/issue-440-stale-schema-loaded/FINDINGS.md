# Findings — #440

`schema_events.rs:48-53` previously only rejected when a *different* request id was pending; `None` (post-disconnect) fell through and applied the stale schema.
`request_schema_introspection` (app/mod.rs:434) always sets `schema_request` before dispatch, so a legit response always has `Some(request_id)` pending. `handle_schema_request_failure` already used exact-match (`!= Some`).
Updated two app tests that sent SchemaLoaded without a pending request. Added `schema_loaded_after_disconnect_is_dropped` regression test.
Same `is_some_and` shape exists in `connection/connection_events.rs:75-80` (`on_connected`) — outside my file ownership; flagged to parent.
