# Database Components

Native egui primitives for connection summaries and database-provider badges. These widgets are presentation-only: they return typed user actions but do not connect, disconnect, edit, delete, or mutate connection state.

## Public API

- `DatabaseDriver`: provider display name and icon for PostgreSQL, SQLite, MySQL, and SQL Server.
- `ConnectionStatus`: `Connected`, `Connecting`, `Disconnected`, and `Error` status presentation.
- `ConnectionCard`: connection identity/host/SSL summary with a `.ssl(bool)` builder and optional `ConnectionCardAction` response.
- `ConnectionCardAction`: `Connect`, `Disconnect`, `Edit`, and legacy `Delete`. `Delete` remains a public variant but the card currently renders no delete action.
- `DatabaseTypeBadge`: fixed-size, non-clickable provider badge.

Names are re-exported through both `components::database::*` and `components::*`.

## Behavior contract

- Connected cards offer `Disconnect`; disconnected cards offer `Connect`; error cards offer `Retry` (emitting the existing `Connect` action); `Edit` is always available.
- Connecting cards show a disabled `Connecting...` action so a repeated click cannot dispatch another connection request while the state is pending.
- The caller receives the action and owns runtime dispatch/state refresh; the component itself never mutates connection state.
- Long connection names, provider/database labels, and hosts are truncated to their allocated region and remain available in hover text. The SSL badge has accessible label/tooltip metadata.
- `Delete` remains in the public action enum for compatibility although no delete button is rendered.
- Provider/status mappings and visibility decisions live in `handler.rs`; egui layout and painting stay in `ui.rs`.
- Geometry is local to `config.rs`; semantic colors and shared spacing/radius/typography come from `DbProTheme` and `tokens.rs`.

## Example

```rust
use crate::components::{ConnectionCard, ConnectionCardAction, ConnectionStatus, DatabaseDriver};

if let Some(action) = ConnectionCard::new(
    "Local database",
    DatabaseDriver::Sqlite,
    "./local.db",
    "main",
    ConnectionStatus::Disconnected,
    theme,
)
.ssl(true)
.show(ui)
{
    match action {
        ConnectionCardAction::Connect => { /* dispatch connection request */ }
        ConnectionCardAction::Disconnect => { /* dispatch disconnect request */ }
        ConnectionCardAction::Edit => { /* open caller-owned editor */ }
        ConnectionCardAction::Delete => { /* legacy variant; no card button emits this */ }
    }
}
```
