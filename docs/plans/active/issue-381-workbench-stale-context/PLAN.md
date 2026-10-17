# Issue #381: open_schema_workbench keeps stale preview/name/confirmation when schema context changes

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. `open_schema_workbench` detects schema/table context change and calls `clear_plan_state()` + clears `name`.
