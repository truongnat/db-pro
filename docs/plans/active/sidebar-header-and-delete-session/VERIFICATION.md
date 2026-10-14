# Verification

## Automated

```text
cargo test -p db-pro-ui -- \
  deleting_non_active_connection_preserves_active_session \
  deleting_active_connection_clears_session \
  sidebar_header_launcher_opens_full_command_palette \
  deleting_sibling_connection_does_not_auto_reconnect_active
```

Result (2026-09-18): **4 passed / 0 failed**

**[CORRECTED 2026-10-13 — `_audit-2026-10-13` slice 04]** the recorded command above
listed `connection_switcher_opens_palette_scoped_to_connections`, a test that does not
exist anywhere in the repo (the merged-launcher design intentionally has no
connections-scoped palette — header name+search opens the full Command Palette instead).
The real four tests are `events.rs:138` (`deleting_non_active_connection_preserves_active_session`),
`events.rs:179` (`deleting_active_connection_clears_session`), `app_tests.rs:3662`
(`sidebar_header_launcher_opens_full_command_palette`), and `app_tests.rs:3674`
(`deleting_sibling_connection_does_not_auto_reconnect_active`). Re-run on 2026-10-13
(`cargo test -p db-pro-ui --lib -- <four names>`): **4 passed / 0 failed**.

## Runtime

- UI runtime evidence pending (manual: header name → Switch Connection; search → Command Palette; delete sibling while A connected)

## Provider

- n/a for SQL; session state is UI/runtime only
