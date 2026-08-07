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
        }
    }

    pub fn postgres() -> Self {
        Self {
            driver: DriverType::Postgres,
            query: QueryCapabilities {
                multi_statement: true,
                explain: true,
                cancel: true,
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
                cancel: false,
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
}

#[cfg(test)]
mod tests {
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
    }

    #[test]
    fn sqlite_capabilities_have_expected_gaps() {
        let caps = DatabaseCapabilities::sqlite();
        assert!(!caps.query.cancel);
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
    fn capabilities_are_serializable() {
        let caps = DatabaseCapabilities::postgres();
        let json = serde_json::to_string(&caps).unwrap();
        let deserialized: DatabaseCapabilities = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.driver, DriverType::Postgres);
        assert!(deserialized.schema.schemas);
    }
}
