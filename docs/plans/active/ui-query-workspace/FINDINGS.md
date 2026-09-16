# Findings — UI05 Query Workspace

## Audit Findings

1. `crates/ui/src/query_view.rs`:
   - Header `Run` and `Stop` used legacy `primary_button_with_icon` and `secondary_button_with_icon`.
   - `Builder` used legacy `secondary_button`.
   - `More actions` used `compact_icon_button`.
   - Editor find bar used `compact_icon_button` for previous/next match and close.
   - SQL snippets list used legacy `compact_button`.
