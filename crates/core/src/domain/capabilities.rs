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

/// Features a consumer can ask about when a capability flag is `false`.
///
/// This is the capability-level reason channel for #234: a boolean alone does not
/// say *why* something is unavailable; [`DatabaseCapabilities::limitation`] does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapabilityFeature {
    Cancel,
    Explain,
    Parameters,
    NumberedParameters,
    PositionalParameters,
    Schemas,
    AlterColumnType,
    Sequences,
    EnumTypes,
    Functions,
    UuidType,
    ArrayTypes,
    ServerSessions,
    Partitions,
    Tablespaces,
    ObjectDependencies,
    SshTunnel,
    Backup,
    DataDiff,
    TransactionalDdl,
    RenameObjects,
}

impl DatabaseCapabilities {
    /// Whether `feature` is advertised as available for this capability set.
    pub fn supports(&self, feature: CapabilityFeature) -> bool {
        match feature {
            CapabilityFeature::Cancel => self.query.cancel,
            CapabilityFeature::Explain => self.query.explain,
            CapabilityFeature::Parameters => self.query.parameters,
            CapabilityFeature::NumberedParameters => self.query.numbered_parameters,
            CapabilityFeature::PositionalParameters => self.query.positional_parameters,
            CapabilityFeature::Schemas => self.schema.schemas,
            CapabilityFeature::AlterColumnType => self.schema.alter_column_type,
            CapabilityFeature::Sequences => self.schema.sequences,
            CapabilityFeature::EnumTypes => self.schema.enum_types,
            CapabilityFeature::Functions => self.schema.functions,
            CapabilityFeature::UuidType => self.data.uuid_type,
            CapabilityFeature::ArrayTypes => self.data.array_types,
            CapabilityFeature::ServerSessions => self.features.server_sessions,
            CapabilityFeature::Partitions => self.features.partitions,
            CapabilityFeature::Tablespaces => self.features.tablespaces,
            CapabilityFeature::ObjectDependencies => self.features.object_dependencies,
            CapabilityFeature::SshTunnel => self.features.ssh_tunnel,
            CapabilityFeature::Backup => self.features.backup,
            CapabilityFeature::DataDiff => self.features.data_diff,
            CapabilityFeature::TransactionalDdl => self.schema.transactional_ddl,
            CapabilityFeature::RenameObjects => self.schema.rename_objects,
        }
    }

    /// User-facing reason `feature` is unavailable, if it is.
    ///
    /// Returns `None` when the feature is supported. Reasons are driver-specific where
    /// the product path (not the engine) is the limiting factor.
    pub fn limitation(&self, feature: CapabilityFeature) -> Option<&'static str> {
        if self.supports(feature) {
            return None;
        }
        Some(match (self.driver, feature) {
            (DriverType::Postgres, CapabilityFeature::Cancel) => {
                "PostgreSQL wire-level query cancellation is not exposed by the connector yet"
            }
            (_, CapabilityFeature::Explain) => "Explain/query plan is not available for this driver",
            (DriverType::Postgres, CapabilityFeature::PositionalParameters) => {
                "PostgreSQL uses numbered parameters ($1, $2), not positional ?"
            }
            (DriverType::SQLite, CapabilityFeature::Schemas) => {
                "SQLite has no named schemas; attached databases are not introspected in v0.1"
            }
            (DriverType::SQLite, CapabilityFeature::AlterColumnType) => {
                "SQLite cannot ALTER COLUMN type without table rebuild"
            }
            (DriverType::SQLite, CapabilityFeature::Sequences)
            | (DriverType::Mysql, CapabilityFeature::Sequences) => {
                "this engine has no standalone SEQUENCE objects"
            }
            (DriverType::SQLite, CapabilityFeature::EnumTypes) => {
                "SQLite has no native ENUM type"
            }
            (DriverType::SQLite, CapabilityFeature::Functions)
            | (DriverType::Mysql, CapabilityFeature::Functions) if !self.schema.functions => {
                "stored functions/procedures are not available for this driver"
            }
            (DriverType::SQLite, CapabilityFeature::UuidType) => {
                "SQLite has no native UUID type; values are stored as TEXT"
            }
            (DriverType::SQLite, CapabilityFeature::ArrayTypes)
            | (DriverType::Mysql, CapabilityFeature::ArrayTypes) => {
                "array columns are not supported for this driver"
            }
            (DriverType::SQLite, CapabilityFeature::ServerSessions)
            | (DriverType::Mysql, CapabilityFeature::ServerSessions) => {
                "user/role management is PostgreSQL-only in this build"
            }
            (DriverType::SQLite, CapabilityFeature::Partitions)
            | (DriverType::Mysql, CapabilityFeature::Partitions) => {
                "partition administration is PostgreSQL-only in this build"
            }
            (DriverType::SQLite, CapabilityFeature::Tablespaces)
            | (DriverType::Mysql, CapabilityFeature::Tablespaces) => {
                "tablespaces are PostgreSQL-only in this build"
            }
            (DriverType::SQLite, CapabilityFeature::ObjectDependencies)
            | (DriverType::Mysql, CapabilityFeature::ObjectDependencies) => {
                "cross-schema dependency catalog is PostgreSQL-only in this build"
            }
            (DriverType::SQLite, CapabilityFeature::SshTunnel) => {
                "SSH tunneling is not used for local SQLite files"
            }
            (DriverType::Mysql, CapabilityFeature::SshTunnel) => {
                "SSH tunneling is not implemented for MySQL yet"
            }
            (DriverType::Mysql, CapabilityFeature::Backup) => {
                "backup/restore is not implemented for MySQL yet"
            }
            (DriverType::Mysql, CapabilityFeature::Cancel) => {
                "the MySQL connector does not support cancelling a running query"
            }
            (DriverType::Mysql, CapabilityFeature::NumberedParameters) => {
                "MySQL uses positional ? parameters, not numbered $n"
            }
            (DriverType::Mysql, CapabilityFeature::UuidType) => {
                "MySQL has no native UUID type in the shipped capability set"
            }
            (DriverType::Mysql, CapabilityFeature::TransactionalDdl) => {
                "MySQL DDL statements cause an implicit commit"
            }
            (DriverType::Mysql, CapabilityFeature::RenameObjects)
            | (DriverType::SQLite, CapabilityFeature::RenameObjects) => {
                "renaming arbitrary schema objects is limited for this driver"
            }
            (DriverType::SQLite, CapabilityFeature::NumberedParameters) => {
                "SQLite uses positional ? parameters, not numbered $n"
            }
            (DriverType::SQLite, CapabilityFeature::DataDiff) if !self.features.data_diff => {
                "data diff is not available for this driver"
            }
            (_, CapabilityFeature::Parameters) => {
                "parameterized queries are not available for this driver"
            }
            (_, CapabilityFeature::DataDiff) => {
                "data diff is not available for this driver"
            }
            (_, CapabilityFeature::Backup) => {
                "backup/restore is not available for this driver"
            }
            _ => "this capability is not available for the current driver",
        })
    }

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
                // Positional `?` binding is served by `mysql/query_mapper::bind_params`
                // through `MySqlConnector::query`/`execute`.
                parameters: true,
                numbered_parameters: false,
                positional_parameters: true,
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
                // `DataDiffService` quotes identifiers through the connector dialect; MySQL now
                // has a dialect arm, so the shared path works.
                data_diff: true,
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
    fn mysql_capabilities_advertise_positional_parameters_when_binder_exists() {
        let caps = DatabaseCapabilities::mysql();
        assert!(caps.query.parameters);
        assert!(caps.query.positional_parameters);
        assert!(!caps.query.numbered_parameters);
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
        assert_eq!(
            pg.limitation(CapabilityFeature::Cancel),
            Some("PostgreSQL wire-level query cancellation is not exposed by the connector yet")
        );
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
