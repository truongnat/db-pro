# Database component API

Types are re-exported from `components::database` and `components`.

## ConnectionCard

```rust
if let Some(action) = ConnectionCard::new(
    "Local database", DatabaseDriver::Sqlite, "./local.db", "main",
    ConnectionStatus::Disconnected, theme,
).ssl(true).show(ui) {
    handle_connection_action(action);
}
```

`ConnectionCard::new(name, driver, host, database, status, theme)` borrows display strings. `.ssl(bool)` adds the SSL badge (default false). `show(ui) -> Option<ConnectionCardAction>` emits at most one action and does not perform it.

`ConnectionStatus` values are `Connected`, `Connecting`, `Disconnected`, and `Error`. `ConnectionCardAction` values are `Connect`, `Disconnect`, `Edit`, and `Delete`; current card buttons emit Connect (including Retry), Disconnect, or Edit. `Delete` remains in the enum for compatibility, but this card has no Delete button.

Status/action mapping: Connected shows Disconnect; Connecting shows a disabled Connecting... button; Disconnected shows Connect; Error shows Retry, which emits `Connect`. Edit remains available for all states. Long text truncates visually and is available in hover text.

## DatabaseTypeBadge and driver metadata

`DatabaseTypeBadge::new(driver, theme).show(ui) -> Response` renders a non-clickable provider label. `DatabaseDriver::{PostgreSql, Sqlite, MySql, SqlServer}` provides `name()` and `icon()` metadata. The component does not imply that every provider is supported by every DB Pro feature.
