use super::{DataCapabilities, DatabaseCapabilities, FeatureCapabilities, QueryCapabilities, SchemaCapabilities};
use crate::domain::connection::DriverType;

impl DatabaseCapabilities {
    /// Returns the capabilities for the given driver type.
    pub fn for_driver(driver: DriverType) -> Self {
        match driver {
            DriverType::Postgres => Self::postgres(),
            DriverType::SQLite => Self::sqlite(),
            DriverType::Mysql => Self::mysql(),
            DriverType::SqlServer => Self::sql_server(),
        }
    }

    pub fn postgres() -> Self {
        Self {
            driver: DriverType::Postgres,
            query: Self::postgres_query(),
            schema: Self::postgres_schema(),
            data: Self::postgres_data(),
            features: Self::postgres_features(),
        }
    }

    fn postgres_query() -> QueryCapabilities {
        QueryCapabilities {
            multi_statement: true,
            explain: true,
            // The connector does not yet expose PostgreSQL's wire-level
            // cancellation primitive; do not advertise best-effort task
            // cancellation as provider cancellation.
            cancel: false,
            parameters: true,
            numbered_parameters: true,
            positional_parameters: false,
            ilike: true,
            glob: false,
            max_rows_limit: None,
        }
    }

    fn postgres_schema() -> SchemaCapabilities {
        SchemaCapabilities {
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
        }
    }

    fn postgres_data() -> DataCapabilities {
        DataCapabilities {
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
        }
    }

    fn postgres_features() -> FeatureCapabilities {
        FeatureCapabilities {
            server_sessions: true,
            partitions: true,
            tablespaces: true,
            object_dependencies: true,
            ssh_tunnel: true,
            backup: true,
            schema_diff: true,
            data_diff: true,
        }
    }

    pub fn sqlite() -> Self {
        Self {
            driver: DriverType::SQLite,
            query: Self::sqlite_query(),
            schema: Self::sqlite_schema(),
            data: Self::sqlite_data(),
            features: Self::sqlite_features(),
        }
    }

    fn sqlite_query() -> QueryCapabilities {
        QueryCapabilities {
            multi_statement: true,
            explain: true,
            // SQLite cancellation interrupts the active VM and waits for
            // the actor to acknowledge that it is ready for reuse.
            cancel: true,
            parameters: true,
            numbered_parameters: false,
            positional_parameters: true,
            ilike: false,
            glob: true,
            max_rows_limit: None,
        }
    }

    fn sqlite_schema() -> SchemaCapabilities {
        SchemaCapabilities {
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
        }
    }

    fn sqlite_data() -> DataCapabilities {
        DataCapabilities {
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
        }
    }

    fn sqlite_features() -> FeatureCapabilities {
        FeatureCapabilities {
            server_sessions: false,
            partitions: false,
            tablespaces: false,
            object_dependencies: false,
            ssh_tunnel: false,
            backup: true,
            schema_diff: true,
            data_diff: true,
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
            query: Self::mysql_query(),
            schema: Self::mysql_schema(),
            data: Self::mysql_data(),
            features: Self::mysql_features(),
        }
    }

    fn mysql_query() -> QueryCapabilities {
        QueryCapabilities {
            multi_statement: true,
            explain: true,
            cancel: false,
            // Positional `?` binding is served by `mysql/query_mapper::bind_params`
            // through `MySqlConnector::query`/`execute`.
            parameters: true,
            numbered_parameters: false,
            positional_parameters: true,
            ilike: false,
            glob: false,
            max_rows_limit: None,
        }
    }

    fn mysql_schema() -> SchemaCapabilities {
        SchemaCapabilities {
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
        }
    }

    fn mysql_data() -> DataCapabilities {
        DataCapabilities {
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
        }
    }

    fn mysql_features() -> FeatureCapabilities {
        FeatureCapabilities {
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
        }
    }

    /// SQL Server capabilities. The adapter deliberately advertises only paths
    /// implemented by the TDS provider; PostgreSQL-only services stay gated.
    pub fn sql_server() -> Self {
        Self {
            driver: DriverType::SqlServer,
            query: Self::sql_server_query(),
            schema: Self::sql_server_schema(),
            data: Self::sql_server_data(),
            features: Self::sql_server_features(),
        }
    }

    fn sql_server_query() -> QueryCapabilities {
        QueryCapabilities {
            multi_statement: true,
            explain: true,
            cancel: false,
            parameters: true,
            numbered_parameters: false,
            positional_parameters: false,
            ilike: false,
            glob: false,
            max_rows_limit: None,
        }
    }

    fn sql_server_schema() -> SchemaCapabilities {
        SchemaCapabilities {
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
            enum_types: false,
            rename_objects: true,
        }
    }

    fn sql_server_data() -> DataCapabilities {
        DataCapabilities {
            insert: true,
            update: true,
            delete: true,
            composite_pk: true,
            no_pk_tables: true,
            json_type: true,
            uuid_type: true,
            blob_type: true,
            array_types: false,
            generated_columns: true,
        }
    }

    fn sql_server_features() -> FeatureCapabilities {
        FeatureCapabilities {
            server_sessions: false,
            partitions: true,
            tablespaces: false,
            object_dependencies: true,
            ssh_tunnel: false,
            backup: false,
            schema_diff: true,
            data_diff: true,
        }
    }
}
