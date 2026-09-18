use crate::{UiDriver, UiSslMode};
use lucide_icons::Icon;

/// Static configuration and UI constants for driver selection cards.
#[derive(Clone, Copy, Debug)]
pub struct DriverCardSpec {
    pub driver: UiDriver,
    pub icon: Icon,
    pub name: &'static str,
    pub subtitle: &'static str,
    pub badge: &'static str,
    pub is_disabled: bool,
}

pub const DRIVER_CARD_SPECS: &[DriverCardSpec] = &[
    DriverCardSpec {
        driver: UiDriver::Postgres,
        icon: Icon::Database,
        name: "PostgreSQL",
        subtitle: "Port 5432 · SQL",
        badge: "Active",
        is_disabled: false,
    },
    DriverCardSpec {
        driver: UiDriver::Sqlite,
        icon: Icon::FileCode,
        name: "SQLite",
        subtitle: "Embedded File",
        badge: "Active",
        is_disabled: false,
    },
    DriverCardSpec {
        driver: UiDriver::Mysql,
        icon: Icon::Database,
        name: "MySQL",
        subtitle: "Port 3306 · SQL",
        badge: "Active",
        is_disabled: false,
    },
    DriverCardSpec {
        driver: UiDriver::SqlServer,
        icon: Icon::Database,
        name: "SQL Server",
        subtitle: "Port 1433 · TDS",
        badge: "Active",
        is_disabled: false,
    },
];

/// Cloud preset option definition for UI dropdown.
pub struct CloudPresetOption {
    pub key: &'static str,
    pub label: &'static str,
}

pub const CLOUD_PRESET_OPTIONS: &[CloudPresetOption] = &[
    CloudPresetOption {
        key: "aws_rds:postgres",
        label: "AWS RDS · PostgreSQL",
    },
    CloudPresetOption {
        key: "aws_rds:mysql",
        label: "AWS RDS · MySQL",
    },
    CloudPresetOption {
        key: "aws_aurora:postgres",
        label: "AWS Aurora · PostgreSQL",
    },
    CloudPresetOption {
        key: "gcp_cloudsql:postgres",
        label: "Cloud SQL · PostgreSQL",
    },
    CloudPresetOption {
        key: "gcp_cloudsql:mysql",
        label: "Cloud SQL · MySQL",
    },
    CloudPresetOption {
        key: "azure:postgres",
        label: "Azure Database · PostgreSQL",
    },
    CloudPresetOption {
        key: "azure:mysql",
        label: "Azure Database · MySQL",
    },
];

pub const ENVIRONMENT_OPTIONS: &[&str] = &["Development", "Staging", "Production", "Custom"];
pub const SSL_MODE_OPTIONS: &[&str] = &["Disable", "Require", "Verify CA", "Verify Full"];
pub const AUTH_KIND_OPTIONS: &[&str] = &["Password", "Ephemeral token"];

/// Stable focus IDs for the New Connection form.
///
/// Keep this order aligned with the form's visual and reading order. Decorative labels and
/// validation output are intentionally excluded; driver cards are the first radio group.
pub mod focus_id {
    pub const NAME: &str = "connection.name";
    pub const GROUP: &str = "connection.group";
    pub const HOST: &str = "connection.host";
    pub const PORT: &str = "connection.port";
    pub const DATABASE: &str = "connection.database";
    pub const USERNAME: &str = "connection.username";
    pub const PASSWORD: &str = "connection.password";
    pub const SQLITE_DATABASE_FILE: &str = "connection.sqlite_database_file";
}

/// Default ports by database driver.
pub fn default_port_for_driver(driver: UiDriver) -> &'static str {
    match driver {
        UiDriver::Mysql => "3306",
        UiDriver::SqlServer => "1433",
        UiDriver::Postgres | UiDriver::Sqlite => "5432",
    }
}

/// Default database name by driver.
pub fn default_database_for_driver(driver: UiDriver) -> &'static str {
    match driver {
        UiDriver::Mysql => "mysql",
        UiDriver::SqlServer => "master",
        UiDriver::Postgres | UiDriver::Sqlite => "postgres",
    }
}

/// Default username by driver.
pub fn default_username_for_driver(driver: UiDriver) -> &'static str {
    match driver {
        UiDriver::Mysql => "root",
        UiDriver::SqlServer => "sa",
        UiDriver::Postgres | UiDriver::Sqlite => "postgres",
    }
}

/// Compute environment index for SegmentedTabs.
pub fn environment_to_index(env: &str) -> usize {
    match env {
        "Staging" => 1,
        "Production" => 2,
        "Custom" => 3,
        _ => 0,
    }
}

/// Map SegmentedTabs index back to environment string.
pub fn index_to_environment(idx: usize) -> &'static str {
    match idx {
        1 => "Staging",
        2 => "Production",
        3 => "Custom",
        _ => "Development",
    }
}

/// Compute SSL mode index for SegmentedTabs.
pub fn ssl_mode_to_index(mode: UiSslMode) -> usize {
    match mode {
        UiSslMode::Disable => 0,
        UiSslMode::Require => 1,
        UiSslMode::VerifyCa => 2,
        UiSslMode::VerifyFull => 3,
    }
}

/// Map SegmentedTabs index back to UiSslMode.
pub fn index_to_ssl_mode(idx: usize) -> UiSslMode {
    match idx {
        0 => UiSslMode::Disable,
        1 => UiSslMode::Require,
        2 => UiSslMode::VerifyCa,
        _ => UiSslMode::VerifyFull,
    }
}
