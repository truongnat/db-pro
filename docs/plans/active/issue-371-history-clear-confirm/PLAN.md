# Issue #371: DDL history Clear has no confirm; Open-in-editor spams documents

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. Two-step confirm (`history_clear_confirm` + Confirm clear / Keep history) in draw_execution_history; `OpenSqlAsQuery` reuses a document already holding the same SQL or an empty active doc.
