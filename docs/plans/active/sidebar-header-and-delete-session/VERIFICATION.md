# Verification

## Automated

```text
cargo test -p db-pro-ui -- \
  deleting_non_active_connection_preserves_active_session \
  deleting_active_connection_clears_session \
  connection_switcher_opens_palette_scoped_to_connections \
  deleting_sibling_connection_does_not_auto_reconnect_active
```

Result (2026-09-18): **4 passed / 0 failed**

## Runtime

- UI runtime evidence pending (manual: header name → Switch Connection; search → Command Palette; delete sibling while A connected)

## Provider

- n/a for SQL; session state is UI/runtime only
