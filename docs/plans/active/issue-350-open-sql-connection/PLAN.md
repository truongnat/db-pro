# Issue #350: Open in SQL Editor uses query context connection, not plan's connection

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. `OpenSql` handler binds the new document to `workbench.preview_connection_id` (captured at plan time in `run_plan`) and sets the document schema.
