# Issue #440: Stale SchemaLoaded response applies after disconnect

Status: FIXED in this change

Changed `on_schema_loaded` guard from `is_some_and(|e| e != request_id)` to `schema_request != Some(request_id)` — a response whose pending request was cleared by disconnect/reset is now dropped.
