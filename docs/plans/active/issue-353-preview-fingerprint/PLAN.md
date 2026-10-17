# Issue #353: preview_fingerprint stored but never verified before Apply

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. `apply_workbench_ddl` calls `replan_fingerprint` and rejects apply when the live fingerprint differs (`Plan is stale — regenerate preview`).
