use serde::{Deserialize, Serialize};
#[path = "capabilities/capability_limitations.rs"]
mod capability_limitations;
#[path = "capabilities/capability_presets.rs"]
mod capability_presets;

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
    /// Supports the ILIKE operator (PostgreSQL-style case-insensitive LIKE).
    pub ilike: bool,
    /// Supports the GLOB operator (SQLite-style).
    pub glob: bool,
    /// Maximum number of rows that can be fetched in a single query.
    /// None means no enforced limit.
    pub max_rows_limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaCapabilities {
    /// Supports named schemas (PostgreSQL schemas).
    pub schemas: bool,
    /// Supports ALTER TABLE ... ALTER COLUMN (type, nullability, default).
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
    /// Supports enabling/disabling individual table triggers.
    #[serde(default)]
    pub trigger_enable_disable: bool,
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
    /// Supports server-side PL/pgSQL routine debugging (breakpoints, stepping,
    /// variable inspection) on this connection.
    ///
    /// Every preset sets this to `false`: debug support is *server* state (the
    /// `plugin_debugger` extension exposing `pldbgapi`), not driver state, so the honest
    /// answer only a live probe can give ([`crate::domain::routine_debug`]) — never a
    /// static table.
    #[serde(default)]
    pub routine_debug: bool,
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
    TriggerEnableDisable,
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
    RoutineDebug,
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
            CapabilityFeature::TriggerEnableDisable => self.schema.trigger_enable_disable,
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
            CapabilityFeature::RoutineDebug => self.schema.routine_debug,
        }
    }

    /// User-facing reason `feature` is unavailable, if it is.
    ///
    /// Returns `None` when the feature is supported. Reasons are driver-specific where
    /// the product path (not the engine) is the limiting factor.
    pub fn limitation(&self, feature: CapabilityFeature) -> Option<&'static str> {
        if self.supports(feature) {
            None
        } else {
            Some(capability_limitations::reason(self.driver, feature))
        }
    }
}

#[cfg(test)]
#[path = "tests/capabilities_tests.rs"]
mod tests;
