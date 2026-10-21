//! Tests for `../capabilities.rs` (moved out of the source file).
use super::*;

#[test]
fn postgres_capabilities_are_complete() {
    let caps = DatabaseCapabilities::postgres();
    assert!(caps.query.multi_statement);
    assert!(caps.query.explain);
    assert!(caps.query.cancel);
    assert!(caps.schema.schemas);
    assert!(caps.schema.transactional_ddl);
    assert!(caps.schema.functions);
    assert!(caps.schema.sequences);
    assert!(caps.features.server_sessions);
    assert!(caps.features.partitions);
    assert!(caps.features.tablespaces);
    assert!(caps.query.ilike);
    assert!(!caps.query.glob);
}

#[test]
fn sqlite_capabilities_have_expected_gaps() {
    let caps = DatabaseCapabilities::sqlite();
    assert!(caps.query.cancel);
    assert!(!caps.schema.schemas);
    assert!(!caps.schema.alter_column_type);
    assert!(!caps.schema.functions);
    assert!(!caps.schema.sequences);
    assert!(!caps.features.server_sessions);
    assert!(!caps.features.partitions);
    assert!(!caps.features.tablespaces);
    assert!(!caps.data.uuid_type);
    assert!(!caps.data.array_types);
    assert!(!caps.query.ilike);
    assert!(caps.query.glob);
}

#[test]
fn for_driver_returns_correct_capabilities() {
    let pg = DatabaseCapabilities::for_driver(DriverType::Postgres);
    assert_eq!(pg.driver, DriverType::Postgres);
    assert!(pg.schema.schemas);

    let sq = DatabaseCapabilities::for_driver(DriverType::SQLite);
    assert_eq!(sq.driver, DriverType::SQLite);
    assert!(!sq.schema.schemas);
}

#[test]
fn mysql_capabilities_advertise_positional_parameters_when_binder_exists() {
    let caps = DatabaseCapabilities::mysql();
    assert!(caps.query.parameters);
    assert!(caps.query.positional_parameters);
    assert!(!caps.query.numbered_parameters);
    assert!(!caps.query.ilike);
    assert!(!caps.query.glob);
}

#[test]
fn sql_server_capabilities_match_tds_provider_boundary() {
    let caps = DatabaseCapabilities::sql_server();
    assert!(caps.query.parameters);
    assert!(!caps.query.numbered_parameters);
    assert!(!caps.query.positional_parameters);
    assert!(caps.schema.schemas);
    assert!(caps.schema.indexes);
    assert!(caps.data.uuid_type);
    assert!(!caps.data.array_types);
    assert!(!caps.features.server_sessions);
    assert!(!caps.features.ssh_tunnel);
    assert!(!caps.features.backup);
    // Partitions and the dependency catalog resolve a PostgreSQL handle in
    // `runtime/src/api.rs` (`postgres_handle()`), and the TDS introspection never
    // queries `sys.sequences`; the flags must stay false (#260 honesty flip).
    assert!(!caps.features.partitions);
    assert!(!caps.features.object_dependencies);
    assert!(!caps.schema.sequences);
}

#[test]
fn mysql_capabilities_do_not_advertise_postgres_only_features() {
    // Each of these features has exactly one product path, and that path rejects MySQL:
    // user management (`application/user_service.rs`), partitions
    // (`runtime/src/api.rs`), backup/restore (`application/backup_service.rs`).
    let caps = DatabaseCapabilities::mysql();
    assert!(!caps.features.server_sessions);
    assert!(!caps.features.partitions);
    assert!(!caps.features.backup);
    assert!(caps.features.data_diff, "MySQL dialect enables shared data-diff path");
    // Already false before this correction; pinned so they cannot drift back.
    assert!(!caps.features.tablespaces);
    assert!(!caps.features.object_dependencies);
}

#[test]
fn limitation_explains_why_a_false_flag_is_unavailable() {
    let pg = DatabaseCapabilities::postgres();
    assert!(pg.limitation(CapabilityFeature::Cancel).is_none());
    assert!(pg.limitation(CapabilityFeature::Parameters).is_none());
    assert!(pg.supports(CapabilityFeature::Parameters));

    let mysql = DatabaseCapabilities::mysql();
    assert_eq!(
        mysql.limitation(CapabilityFeature::Backup),
        Some("backup/restore is not implemented for MySQL yet")
    );
    assert_eq!(
        mysql.limitation(CapabilityFeature::ServerSessions),
        Some("user/role management is PostgreSQL-only in this build")
    );
    assert!(mysql.limitation(CapabilityFeature::Parameters).is_none());

    let sql_server = DatabaseCapabilities::sql_server();
    // SQL Server cancellation shipped with the SPID guard + out-of-band KILL path
    // in `SqlServerConnector` (#260): a supported feature must expose no limitation.
    assert!(sql_server.supports(CapabilityFeature::Cancel));
    assert!(sql_server.limitation(CapabilityFeature::Cancel).is_none());
    assert_eq!(
        sql_server.limitation(CapabilityFeature::Backup),
        Some("SQL Server backup/restore is not implemented by the shared backup service yet")
    );
    // Mirror the MySQL PostgreSQL-only wording; the runtime path is the same
    // `PostgresApi::partitions`/`object_dependencies` handle gate (#260).
    assert_eq!(
        sql_server.limitation(CapabilityFeature::Partitions),
        Some("partition administration is PostgreSQL-only in this build")
    );
    assert_eq!(
        sql_server.limitation(CapabilityFeature::ObjectDependencies),
        Some("cross-schema dependency catalog is PostgreSQL-only in this build")
    );
    assert_eq!(
        sql_server.limitation(CapabilityFeature::Sequences),
        Some("sequence introspection is not implemented by the SQL Server provider yet")
    );
}

#[test]
fn mysql_capabilities_keep_the_flags_a_shipping_path_serves() {
    // The other half of the rule: a flag stays `true` only with a path that works.
    // Query/execute/DDL go through the driver-agnostic `DbConnector` methods (live
    // MySQL evidence in `crates/infrastructure/tests/mysql_fixture_matrix.rs` and
    // `docs/release/evidence/v01-runtime/providers/60-mysql-live-fixture-and-mapper.md`),
    // explain has a live test (`mysql_integration.rs`), introspection reports tables,
    // routines and indexes (`mysql/introspect.rs`), and schema diff compares two
    // introspection results, so it needs no MySQL-specific path.
    let caps = DatabaseCapabilities::mysql();
    assert!(caps.query.explain);
    assert!(caps.query.multi_statement);
    assert!(caps.query.cancel);
    assert!(caps.schema.schemas);
    assert!(caps.schema.functions);
    assert!(caps.schema.views);
    assert!(caps.schema.triggers);
    assert!(caps.schema.indexes);
    assert!(caps.data.insert);
    assert!(caps.data.update);
    assert!(caps.data.delete);
    assert!(caps.features.schema_diff);
}

#[test]
fn capabilities_are_serializable() {
    let caps = DatabaseCapabilities::postgres();
    let json = serde_json::to_string(&caps).unwrap();
    let deserialized: DatabaseCapabilities = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.driver, DriverType::Postgres);
    assert!(deserialized.schema.schemas);
}
