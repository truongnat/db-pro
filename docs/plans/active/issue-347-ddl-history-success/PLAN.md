# Issue #347: Execution history records success=true at dispatch

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. `apply_workbench_ddl` records `success=false` (pending) at dispatch and stores `(request_id, history_index)` in `workbench.pending_execution`; `finalize_execution` patches the entry on `DdlCompleted` / `QueryFailed`.
