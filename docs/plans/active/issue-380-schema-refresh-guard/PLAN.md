# Issue #380: Schema refresh has no in-flight guard

Status: VERIFIED-FIXED (already fixed in 22819b13)

No code change required. `refresh_schema_palette` returns early with 'Schema refresh already in progress…' when `schema.explorer.schema_request.is_some()`. File (palette/palette_view.rs) belongs to a sibling; verified read-only.
