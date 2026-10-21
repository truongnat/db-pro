//! Tests for `../routine_debug.rs`.

use super::*;
use crate::domain::capabilities::{CapabilityFeature, DatabaseCapabilities};
use crate::domain::connection::DriverType;
use crate::domain::query::{ColumnMeta, Row};

fn probe_row(installed: bool, available: bool, preloaded: bool, version: &str) -> QueryResult {
    QueryResult {
        columns: vec![
            ColumnMeta {
                name: "debug_extension_installed".into(),
                data_type: "bool".into(),
                nullable: false,
            },
            ColumnMeta {
                name: "debug_extension_available".into(),
                data_type: "bool".into(),
                nullable: false,
            },
            ColumnMeta {
                name: "preload_configured".into(),
                data_type: "bool".into(),
                nullable: true,
            },
            ColumnMeta {
                name: "server_version".into(),
                data_type: "text".into(),
                nullable: false,
            },
        ],
        rows: vec![Row(vec![
            CellValue::Bool(installed),
            CellValue::Bool(available),
            CellValue::Bool(preloaded),
            CellValue::Text(version.into()),
        ])],
        row_count: 1,
        duration_ms: 0,
    }
}

#[test]
fn probe_sql_reads_both_extension_catalogs_and_preload_setting() {
    // The exact SQL the running probe sends; pinned so guidance text cannot drift from
    // what is actually measured.
    assert_eq!(
        DEBUG_CAPABILITY_PROBE_SQL,
        "SELECT \
         EXISTS (SELECT 1 FROM pg_catalog.pg_extension WHERE extname IN ('plugin_debugger', 'pldbgapi')) AS debug_extension_installed, \
         EXISTS (SELECT 1 FROM pg_catalog.pg_available_extensions WHERE name IN ('plugin_debugger', 'pldbgapi')) AS debug_extension_available, \
         pg_catalog.current_setting('shared_preload_libraries', true) LIKE '%plugin_debugger%' AS preload_configured, \
         pg_catalog.current_setting('server_version') AS server_version"
    );
    assert_eq!(DEBUG_EXTENSION_NAMES, ["plugin_debugger", "pldbgapi"]);
}

#[test]
fn probe_sql_only_reads_catalogs() {
    // Safety: the probe must never write, install, or reload anything. The substring list
    // stays verb-shaped because a column alias may legitimately mention installation.
    let lower = DEBUG_CAPABILITY_PROBE_SQL.to_ascii_lowercase();
    for forbidden in [
        "insert",
        "update ",
        "delete ",
        "create ",
        "alter ",
        "drop ",
        "grant ",
        "set_config",
        "pg_reload_conf",
        "load(",
    ] {
        assert!(!lower.contains(forbidden), "probe SQL must not contain {forbidden:?}");
    }
    assert!(lower.starts_with("select"));
    assert!(lower.contains("pg_catalog.pg_extension"));
    assert!(lower.contains("pg_catalog.pg_available_extensions"));
}

#[test]
fn missing_extension_probe_reports_extension_missing() {
    let probe = RoutineDebugProbe::from_probe_result(&probe_row(false, false, false, "18.0"));
    assert_eq!(probe.state, RoutineDebugState::ExtensionMissing);
    assert!(!probe.extension_installed);
    assert!(!probe.extension_available);
    assert_eq!(probe.server_version, "18.0");
    assert_eq!(probe.probe_error, None);
}

#[test]
fn installable_but_not_installed_still_reports_missing_with_available_flag() {
    let probe = RoutineDebugProbe::from_probe_result(&probe_row(false, true, false, "16.4"));
    assert_eq!(probe.state, RoutineDebugState::ExtensionMissing);
    assert!(
        probe.extension_available,
        "pg_available_extensions hit must survive parsing"
    );
}

#[test]
fn installed_but_not_preloaded_reports_not_preloaded() {
    let probe = RoutineDebugProbe::from_probe_result(&probe_row(true, true, false, "16.4"));
    assert_eq!(probe.state, RoutineDebugState::ExtensionNotPreloaded);
    assert!(probe.extension_installed);
}

#[test]
fn fully_prepared_server_reports_server_ready_but_stepping_stays_off() {
    let probe = RoutineDebugProbe::from_probe_result(&probe_row(true, true, true, "16.4"));
    assert_eq!(probe.state, RoutineDebugState::ServerReady);
    assert!(probe.preload_configured);
    // No pldbgapi protocol client ships in this build; claiming otherwise would fake it.
    assert!(!probe.state.stepping_supported());
}

#[test]
fn empty_or_unreadable_probe_rows_report_probe_failed() {
    let empty = QueryResult {
        columns: Vec::new(),
        rows: Vec::new(),
        row_count: 0,
        duration_ms: 0,
    };
    let probe = RoutineDebugProbe::from_probe_result(&empty);
    assert_eq!(probe.state, RoutineDebugState::ProbeFailed);
    assert!(probe.probe_error.is_some());

    let unreadable = QueryResult {
        columns: Vec::new(),
        rows: vec![Row(vec![CellValue::Null])],
        row_count: 1,
        duration_ms: 0,
    };
    assert_eq!(
        RoutineDebugProbe::from_probe_result(&unreadable).state,
        RoutineDebugState::ProbeFailed
    );
}

#[test]
fn unsupported_driver_probe_carries_no_server_claims() {
    let probe = RoutineDebugProbe::unsupported_driver();
    assert_eq!(probe.state, RoutineDebugState::UnsupportedDriver);
    assert!(!probe.extension_installed);
    assert!(!probe.extension_available);
    assert!(!probe.preload_configured);
    assert!(probe.probe_error.is_none());
    assert!(!probe.state.stepping_supported());
    let caps = DatabaseCapabilities::for_driver(DriverType::SQLite);
    assert!(!caps.supports(CapabilityFeature::RoutineDebug));
}

#[test]
fn every_preset_keeps_routine_debug_false_until_probed() {
    for driver in [
        DriverType::Postgres,
        DriverType::SQLite,
        DriverType::Mysql,
        DriverType::SqlServer,
    ] {
        let caps = DatabaseCapabilities::for_driver(driver);
        assert!(!caps.schema.routine_debug, "{driver:?} preset must not claim debug");
        assert!(!caps.supports(CapabilityFeature::RoutineDebug));
        let reason = caps
            .limitation(CapabilityFeature::RoutineDebug)
            .unwrap_or_else(|| panic!("{driver:?} must explain why routine debug is off"));
        assert!(!reason.is_empty());
    }
    // PostgreSQL's reason names the real blocker and the no-auto-install rule.
    let pg = DatabaseCapabilities::for_driver(DriverType::Postgres);
    let reason = pg.limitation(CapabilityFeature::RoutineDebug).unwrap();
    assert!(reason.contains("plugin_debugger"), "{reason}");
    assert!(reason.contains("pldbgapi"), "{reason}");
    assert!(reason.contains("PostgreSQL 18"), "{reason}");
    assert!(reason.contains("will not install server extensions"), "{reason}");
}

#[test]
fn probe_round_trips_through_serde() {
    let probe = RoutineDebugProbe::from_probe_result(&probe_row(false, true, false, "18.0"));
    let json = serde_json::to_string(&probe).unwrap();
    let back: RoutineDebugProbe = serde_json::from_str(&json).unwrap();
    assert_eq!(back, probe);
}
