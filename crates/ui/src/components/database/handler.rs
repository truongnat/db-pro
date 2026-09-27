use crate::DbProTheme;
use egui::Color32;
use lucide_icons::Icon;

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
mod tests {
    use super::*;

    #[test]
    fn drivers_keep_their_display_names_and_icons() {
        assert_eq!(DatabaseDriver::PostgreSql.name(), "PostgreSQL");
        assert_eq!(format!("{:?}", DatabaseDriver::PostgreSql.icon()), "Database");
        assert_eq!(DatabaseDriver::Sqlite.name(), "SQLite");
        assert_eq!(format!("{:?}", DatabaseDriver::Sqlite.icon()), "FileCode");
        assert_eq!(DatabaseDriver::MySql.name(), "MySQL");
        assert_eq!(format!("{:?}", DatabaseDriver::MySql.icon()), "Server");
        assert_eq!(DatabaseDriver::SqlServer.name(), "SQL Server");
        assert_eq!(format!("{:?}", DatabaseDriver::SqlServer.icon()), "Layers");
    }

    #[test]
    fn each_status_maps_to_its_existing_label_and_theme_color() {
        let theme = DbProTheme::light();
        let cases = [
            (ConnectionStatus::Connected, "Active", theme.success),
            (ConnectionStatus::Connecting, "Connecting...", theme.info),
            (ConnectionStatus::Disconnected, "Offline", theme.text_tertiary),
            (ConnectionStatus::Error, "Error", theme.danger),
        ];

        for (status, expected_label, expected_color) in cases {
            let presentation = connection_status_presentation(status, &theme);
            assert_eq!(presentation.label, expected_label);
            assert_eq!(presentation.color, expected_color);
        }
    }

    #[test]
    fn each_connection_status_exposes_only_its_safe_primary_action() {
        let cases = [
            (
                ConnectionStatus::Connected,
                ConnectionActionVisibility {
                    show_connect: false,
                    show_retry: false,
                    show_connecting: false,
                    show_disconnect: true,
                },
            ),
            (
                ConnectionStatus::Connecting,
                ConnectionActionVisibility {
                    show_connect: false,
                    show_retry: false,
                    show_connecting: true,
                    show_disconnect: false,
                },
            ),
            (
                ConnectionStatus::Disconnected,
                ConnectionActionVisibility {
                    show_connect: true,
                    show_retry: false,
                    show_connecting: false,
                    show_disconnect: false,
                },
            ),
            (
                ConnectionStatus::Error,
                ConnectionActionVisibility {
                    show_connect: false,
                    show_retry: true,
                    show_connecting: false,
                    show_disconnect: false,
                },
            ),
        ];

        for (status, expected) in cases {
            assert_eq!(connection_action_visibility(status), expected);
        }
    }

    #[test]
    fn content_width_reserves_trailing_status_or_ssl_badge_space() {
        assert_eq!(remaining_content_width(300.0, 72.0, 8.0), 220.0);
        assert_eq!(remaining_content_width(24.0, 72.0, 8.0), 0.0);
    }
}
