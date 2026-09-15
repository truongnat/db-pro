use serde::{Deserialize, Serialize};

use super::connection::DriverType;

/// Describes what a specific database driver/connection supports.
///
/// The frontend and application layer should query capabilities instead of
/// checking driver types directly. This allows adding new drivers without
/// scattering `if postgres { ... } else { ... }` throughout the codebase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseCapabilities {
    pub driver: DriverType,
    pub query: QueryCapabilities,
    pub schema: SchemaCapabilities,
    pub data: DataCapabilities,
    pub features: FeatureCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCapabilities {
    /// Supports executing multiple statements in a single call.
    pub multi_statement: bool,
    /// Supports EXPLAIN / query plan.
    pub explain: bool,
    /// Supports cancelling a running query.
    pub cancel: bool,
    /// Supports parameterized queries.
    pub parameters: bool,
    /// Supports named parameters (e.g. $1, $2).
    pub numbered_parameters: bool,
    /// Supports positional parameters (e.g. ?).
    pub positional_parameters: bool,
    /// Maximum number of rows that can be fetched in a single query.
    /// None means no enforced limit.
    pub max_rows_limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaCapabilities {
    /// Supports named schemas (PostgreSQL schemas).
    pub schemas: bool,
    /// Supports ALTER TABLE ... ALTER COLUMN (type change).
    pub alter_column_type: bool,
    /// Supports ALTER TABLE ... ADD COLUMN.
    pub add_column: bool,
    /// Supports ALTER TABLE ... DROP COLUMN.
    pub drop_column: bool,
    /// Supports ALTER TABLE ... RENAME COLUMN.
    pub rename_column: bool,
    /// DDL operations are transactional (can be rolled back).
    pub transactional_ddl: bool,
    /// Supports foreign keys.
    pub foreign_keys: bool,
    /// Supports creating/dropping indexes.
    pub indexes: bool,
    /// Supports creating/dropping triggers.
    pub triggers: bool,
    /// Supports creating/dropping views.
    pub views: bool,
    /// Supports stored functions / procedures.
    pub functions: bool,
    /// Supports sequences.
    pub sequences: bool,
    /// Supports enum types.
    pub enum_types: bool,
    /// Supports renaming schema objects.
    pub rename_objects: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCapabilities {
    /// Supports INSERT operations.
    pub insert: bool,
    /// Supports UPDATE operations.
    pub update: bool,
    /// Supports DELETE operations.
    pub delete: bool,
    /// Supports composite primary keys.
    pub composite_pk: bool,
    /// Supports tables without primary key.
    pub no_pk_tables: bool,
    /// Supports JSON/JSONB data type.
    pub json_type: bool,
    /// Supports UUID data type.
    pub uuid_type: bool,
    /// Supports BLOB/BYTEA data type.
    pub blob_type: bool,
    /// Supports array data types.
    pub array_types: bool,
    /// Supports generated / default columns.
    pub generated_columns: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureCapabilities {
    /// Supports server-side sessions / user management.
    pub server_sessions: bool,
    /// Supports database partitions.
    pub partitions: bool,
    /// Supports tablespaces.
    pub tablespaces: bool,
    /// Supports cross-schema object dependencies.
    pub object_dependencies: bool,
    /// Supports SSH tunneling.
    pub ssh_tunnel: bool,
    /// Supports backup/restore.
    pub backup: bool,
    /// Supports schema diffing.
    pub schema_diff: bool,
    /// Supports data diffing.
    pub data_diff: bool,
}

impl DatabaseCapabilities {
    /// Returns the capabilities for the given driver type.
    pub fn for_driver(driver: DriverType) -> Self {
        match driver {
            DriverType::Postgres => Self::postgres(),
            DriverType::SQLite => Self::sqlite(),
            DriverType::Mysql => Self::mysql(),
        }
    }

    pub fn postgres() -> Self {
        Self {
            driver: DriverType::Postgres,
            query: QueryCapabilities {
                multi_statement: true,
                explain: true,
                // The connector does not yet expose PostgreSQL's wire-level
                // cancellation primitive; do not advertise best-effort task
                // cancellation as provider cancellation.
                cancel: false,
                parameters: true,
                numbered_parameters: true,
                positional_parameters: false,
                max_rows_limit: None,
            },
            schema: SchemaCapabilities {
                schemas: true,
                alter_column_type: true,
                add_column: true,
                drop_column: true,
                rename_column: true,
                transactional_ddl: true,
                foreign_keys: true,
                indexes: true,
                triggers: true,
                views: true,
                functions: true,
                sequences: true,
                enum_types: true,
                rename_objects: true,
            },
            data: DataCapabilities {
                insert: true,
                update: true,
                delete: true,
                composite_pk: true,
                no_pk_tables: true,
                json_type: true,
                uuid_type: true,
                blob_type: true,
                array_types: true,
                generated_columns: true,
            },
            features: FeatureCapabilities {
                server_sessions: true,
                partitions: true,
                tablespaces: true,
                object_dependencies: true,
                ssh_tunnel: true,
                backup: true,
                schema_diff: true,
                data_diff: true,
            },
        }
    }

    pub fn sqlite() -> Self {
        Self {
            driver: DriverType::SQLite,
            query: QueryCapabilities {
                multi_statement: true,
                explain: true,
                // SQLite cancellation interrupts the active VM and waits for
                // the actor to acknowledge that it is ready for reuse.
                cancel: true,
                parameters: true,
                numbered_parameters: false,
                positional_parameters: true,
                max_rows_limit: None,
            },
            schema: SchemaCapabilities {
                schemas: false,
                alter_column_type: false,
                add_column: true,
                drop_column: true,
                rename_column: true,
                transactional_ddl: true,
                foreign_keys: true,
                indexes: true,
                triggers: true,
                views: true,
                functions: false,
                sequences: false,
                enum_types: false,
                rename_objects: false,
            },
            data: DataCapabilities {
                insert: true,
                update: true,
                delete: true,
                composite_pk: true,
                no_pk_tables: true,
                json_type: true,
                uuid_type: false,
                blob_type: true,
                array_types: false,
                generated_columns: false,
            },
            features: FeatureCapabilities {
                server_sessions: false,
                partitions: false,
                tablespaces: false,
                object_dependencies: false,
                ssh_tunnel: false,
                backup: true,
                schema_diff: true,
                data_diff: true,
            },
        }
    }

    /// MySQL 8 capabilities.
    ///
    /// A flag is `true` only when a shipping code path serves that capability for this
    /// driver. Where the engine could do the work but the product's only path rejects
    /// MySQL, the flag is `false`: a capability-driven consumer must not be told to offer
    /// an action the provider will refuse. Every corrected flag names the code path it was
    /// measured against.
    pub fn mysql() -> Self {
        Self {
            driver: DriverType::Mysql,
            query: QueryCapabilities {
                multi_statement: true,
                explain: true,
                cancel: false,
                // `MySqlConnector::query`/`execute` accept the parameter list and drop it
                // (`mysql/connector.rs`). Until a binder exists, advertising parameter
                // support would let a consumer build a `?`-bound statement that silently
                // runs unbound.
                parameters: false,
                numbered_parameters: false,
                positional_parameters: false,
                max_rows_limit: None,
            },
            schema: SchemaCapabilities {
                schemas: true,
                alter_column_type: true,
                add_column: true,
                drop_column: true,
                rename_column: true,
                transactional_ddl: false,
                foreign_keys: true,
                indexes: true,
                triggers: true,
                views: true,
                functions: true,
                sequences: false,
                enum_types: true,
                rename_objects: false,
            },
            data: DataCapabilities {
                insert: true,
                update: true,
                delete: true,
                composite_pk: true,
                no_pk_tables: true,
                json_type: true,
                uuid_type: false,
                blob_type: true,
                array_types: false,
                generated_columns: true,
            },
            features: FeatureCapabilities {
                // `UserService` rejects every driver except PostgreSQL
                // (`application/user_service.rs`, `ensure_server_sessions_config`).
                server_sessions: false,
                // `PostgresApi::partitions` resolves a PostgreSQL handle and rejects
                // anything else (`runtime/src/api.rs`); no MySQL partition path exists.
                partitions: false,
                tablespaces: false,
                object_dependencies: false,
                ssh_tunnel: false,
                // `BackupService::backup`/`restore` return a validation error for MySQL
                // (`application/backup_service.rs`).
                backup: false,
                // Introspection-based, so it needs no provider-specific path
                // (`application/schema_diff.rs` compares two `IntrospectResult`s).
                schema_diff: true,
                // `DataDiffService::diff_table_data` asks the connector for a dialect, and
                // `CompositeConnector::dialect` has no MySQL arm
                // (`infrastructure/src/connector.rs`), so the only data-diff path errors.
                data_diff: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn postgres_capabilities_are_complete() {
        let caps = DatabaseCapabilities::postgres();
        assert!(caps.query.multi_statement);
        assert!(caps.query.explain);
        assert!(!caps.query.cancel);
        assert!(caps.schema.schemas);
        assert!(caps.schema.transactional_ddl);
        assert!(caps.schema.functions);
        assert!(caps.schema.sequences);
        assert!(caps.features.server_sessions);
        assert!(caps.features.partitions);
        assert!(caps.features.tablespaces);
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
    fn mysql_capabilities_drop_parameters_until_a_binder_exists() {
        // Measured against the provider: `MySqlConnector::query`/`execute` are called with
        // the parameter list and run `sqlx::query(sql)` with no binds
        // (`crates/infrastructure/src/mysql/connector.rs`), so no parameter form is served.
        // When a binder lands, this test is the one that has to change with it.
        let caps = DatabaseCapabilities::mysql();
        assert!(!caps.query.parameters, "no MySQL parameter binder exists yet");
        assert!(
            !caps.query.positional_parameters,
            "`?` placeholders are accepted by the signature and dropped, not bound"
        );
        assert!(!caps.query.numbered_parameters);
    }

    #[test]
    fn mysql_capabilities_do_not_advertise_postgres_only_features() {
        // Each of these features has exactly one product path, and that path rejects MySQL:
        // user management (`application/user_service.rs`), partitions
        // (`runtime/src/api.rs`), backup/restore (`application/backup_service.rs`), and
        // data diff (`application/data_diff.rs` through `CompositeConnector::dialect`,
        // which has no MySQL arm). Advertising them would hand the UI an action that can
        // only fail.
        let caps = DatabaseCapabilities::mysql();
        assert!(!caps.features.server_sessions);
        assert!(!caps.features.partitions);
        assert!(!caps.features.backup);
        assert!(!caps.features.data_diff);
        // Already false before this correction; pinned so they cannot drift back.
        assert!(!caps.features.tablespaces);
        assert!(!caps.features.object_dependencies);
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
        assert!(!caps.query.cancel, "MySQL cancel returns Unsupported");
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
}
