use egui::Color32;
use lucide_icons::Icon;

use crate::DbProTheme;

/// Database provider represented by a connection card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseDriver {
    PostgreSql,
    Sqlite,
    MySql,
    SqlServer,
}

impl DatabaseDriver {
    pub fn name(&self) -> &'static str {
        match self {
            Self::PostgreSql => "PostgreSQL",
            Self::Sqlite => "SQLite",
            Self::MySql => "MySQL",
            Self::SqlServer => "SQL Server",
        }
    }

    pub fn icon(&self) -> Icon {
        match self {
            Self::PostgreSql => Icon::Database,
            Self::Sqlite => Icon::FileCode,
            Self::MySql => Icon::Server,
            Self::SqlServer => Icon::Layers,
        }
    }
}

/// Connection lifecycle state used to choose status presentation and available action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error,
}

/// Action emitted by a connection card; the caller owns and performs the operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionCardAction {
    Connect,
    Disconnect,
    Edit,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ConnectionStatusPresentation {
    pub(super) label: &'static str,
    pub(super) color: Color32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ConnectionActionVisibility {
    pub(super) show_connect: bool,
    pub(super) show_retry: bool,
    pub(super) show_connecting: bool,
    pub(super) show_disconnect: bool,
}

pub(super) fn connection_status_presentation(
    status: ConnectionStatus,
    theme: &DbProTheme,
) -> ConnectionStatusPresentation {
    let (label, color) = match status {
        ConnectionStatus::Connected => ("Active", theme.success),
        ConnectionStatus::Connecting => ("Connecting...", theme.info),
        ConnectionStatus::Disconnected => ("Offline", theme.text_tertiary),
        ConnectionStatus::Error => ("Error", theme.danger),
    };

    ConnectionStatusPresentation { label, color }
}

pub(super) fn remaining_content_width(available_width: f32, trailing_width: f32, gap: f32) -> f32 {
    (available_width - trailing_width - gap).max(0.0)
}

pub(super) fn connection_action_visibility(status: ConnectionStatus) -> ConnectionActionVisibility {
    ConnectionActionVisibility {
        show_connect: status == ConnectionStatus::Disconnected,
        show_retry: status == ConnectionStatus::Error,
        show_connecting: status == ConnectionStatus::Connecting,
        show_disconnect: status == ConnectionStatus::Connected,
    }
}

#[cfg(test)]
#[path = "tests/handler_tests.rs"]
mod tests;
