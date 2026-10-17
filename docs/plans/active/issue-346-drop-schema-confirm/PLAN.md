# Issue #346: plan_drop_schema opens confirm even when plan fails / SQL empty

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. `plan_drop_schema` (schema_workbench_actions.rs:56-63) now sets `apply_confirmation = true` only when `preview_error.is_none() && !preview_sql.trim().is_empty()`.
