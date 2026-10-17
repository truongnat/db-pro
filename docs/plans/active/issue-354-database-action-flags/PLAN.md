# Issue #354: plan_database_action sets both if_exists and if_not_exists; cascade leaks into DROP DATABASE

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. `plan_database_action` delegates to `build_database_request` which sets `if_exists: action == Drop`, `if_not_exists: action == Create`, `cascade: false`.
