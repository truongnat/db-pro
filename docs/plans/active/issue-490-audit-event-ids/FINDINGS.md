# #490 — Findings

- `AuditEvent.id` consumers: `audit_state.rs` HashSets (bookmarks/selected),
  `audit_surface_view.rs` actions (`SetSelected`, `ToggleBookmark`), export
  preview filtering — all in-session, no cross-restart persistence requirement.
- `parse_postgres_csv_line` is `pub` but has exactly one production caller:
  `read_csvlog_page` (via `AuditService::load_events`). Keeping its signature
  unchanged and letting the page reader enrich the id avoids an API change.
- `DefaultHasher` is not specified cross-build, but the type is `std`'s stable
  constructor and ids are local/per-load; good enough per "Stable local
  identity (not a server id)" doc on the field.
- Byte offsets are file-absolute (`start + body_offset + line_in_body`), so a
  larger `max_scan_bytes` page keeps identical ids for the same lines — better
  than body-relative offsets.
- `String::from_utf8_lossy` may replace bytes, shifting nothing: offsets are
  computed over the *lossy text* which is what is parsed; consistent within a
  page since cursor walks the same `body` string.
