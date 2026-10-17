# Verification — #440

`cargo check -p db-pro-app` — clean. `cargo test -p db-pro-app` — 593 passed, 0 failed (includes schema-workbench/event regressions).
New regression test `schema_loaded_after_disconnect_is_dropped` passes.

