# Issue #357: Refresh DDL wipes edits without confirm; Open-in-Query can overwrite active buffer

Status: VERIFIED-FIXED (already fixed in edbcf7ae)

No code change required (file belongs to a sibling area anyway). Refresh now checks `table_ddl_dirty()` and raises `table_ddl_discard_confirm` instead of dropping the buffer; OpenInQuery uses `open_query_document_with` (new document).
