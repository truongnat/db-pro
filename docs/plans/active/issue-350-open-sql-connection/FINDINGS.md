# Findings — #350

schema_workbench_actions.rs:21-38. `preview_connection_id` stored at :102-103. Handler calls `set_document_connection(index, Some(connection_id))` and `set_document_schema`. The handling lives in schema_workbench_actions.rs (my file), not events_query_dispatch.
