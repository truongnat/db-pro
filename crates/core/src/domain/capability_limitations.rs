use super::{CapabilityFeature, DriverType};

pub(super) fn reason(driver: DriverType, feature: CapabilityFeature) -> &'static str {
    match driver {
        DriverType::Postgres => postgres_reason(feature),
        DriverType::SQLite => sqlite_reason(feature),
        DriverType::Mysql => mysql_reason(feature),
        DriverType::SqlServer => sql_server_reason(feature),
    }
}

fn postgres_reason(feature: CapabilityFeature) -> &'static str {
    match feature {
        CapabilityFeature::Cancel => "PostgreSQL wire-level query cancellation is not exposed by the connector yet",
        CapabilityFeature::PositionalParameters => "PostgreSQL uses numbered parameters ($1, $2), not positional ?",
        feature => generic_reason(feature),
    }
}

fn sqlite_reason(feature: CapabilityFeature) -> &'static str {
    match feature {
        CapabilityFeature::Schemas => "SQLite has no named schemas; attached databases are not introspected in v0.1",
        CapabilityFeature::AlterColumnType => "SQLite cannot ALTER COLUMN type without table rebuild",
        CapabilityFeature::Sequences => "this engine has no standalone SEQUENCE objects",
        CapabilityFeature::EnumTypes => "SQLite has no native ENUM type",
        CapabilityFeature::Functions => "stored functions/procedures are not available for this driver",
        CapabilityFeature::UuidType => "SQLite has no native UUID type; values are stored as TEXT",
        CapabilityFeature::ArrayTypes => "array columns are not supported for this driver",
        CapabilityFeature::ServerSessions => "user/role management is PostgreSQL-only in this build",
        CapabilityFeature::Partitions => "partition administration is PostgreSQL-only in this build",
        CapabilityFeature::Tablespaces => "tablespaces are PostgreSQL-only in this build",
        CapabilityFeature::ObjectDependencies => "cross-schema dependency catalog is PostgreSQL-only in this build",
        CapabilityFeature::SshTunnel => "SSH tunneling is not used for local SQLite files",
        CapabilityFeature::NumberedParameters => "SQLite uses positional ? parameters, not numbered $n",
        CapabilityFeature::RenameObjects => "renaming arbitrary schema objects is limited for this driver",
        feature => generic_reason(feature),
    }
}

fn mysql_reason(feature: CapabilityFeature) -> &'static str {
    match feature {
        CapabilityFeature::Sequences => "this engine has no standalone SEQUENCE objects",
        CapabilityFeature::Functions => "stored functions/procedures are not available for this driver",
        CapabilityFeature::ArrayTypes => "array columns are not supported for this driver",
        CapabilityFeature::ServerSessions => "user/role management is PostgreSQL-only in this build",
        CapabilityFeature::Partitions => "partition administration is PostgreSQL-only in this build",
        CapabilityFeature::Tablespaces => "tablespaces are PostgreSQL-only in this build",
        CapabilityFeature::ObjectDependencies => "cross-schema dependency catalog is PostgreSQL-only in this build",
        CapabilityFeature::SshTunnel => "SSH tunneling is not implemented for MySQL yet",
        CapabilityFeature::Backup => "backup/restore is not implemented for MySQL yet",
        CapabilityFeature::Cancel => "the MySQL connector does not support cancelling a running query",
        CapabilityFeature::NumberedParameters => "MySQL uses positional ? parameters, not numbered $n",
        CapabilityFeature::UuidType => "MySQL has no native UUID type in the shipped capability set",
        CapabilityFeature::TransactionalDdl => "MySQL DDL statements cause an implicit commit",
        CapabilityFeature::RenameObjects => "renaming arbitrary schema objects is limited for this driver",
        feature => generic_reason(feature),
    }
}

fn sql_server_reason(feature: CapabilityFeature) -> &'static str {
    match feature {
        CapabilityFeature::Cancel => {
            "SQL Server cancellation requires a separate attention channel that the TDS adapter does not expose yet"
        }
        CapabilityFeature::NumberedParameters => {
            "SQL Server parameters use named @pN placeholders, not PostgreSQL-style $n"
        }
        CapabilityFeature::PositionalParameters => "SQL Server parameters use named @pN placeholders, not positional ?",
        CapabilityFeature::EnumTypes => "SQL Server has no native ENUM type",
        CapabilityFeature::ArrayTypes => "SQL Server array columns are not supported",
        CapabilityFeature::ServerSessions => {
            "SQL Server session administration is not exposed by the shared monitoring service yet"
        }
        CapabilityFeature::Tablespaces => "SQL Server tablespace administration is not a supported provider operation",
        CapabilityFeature::SshTunnel => "SQL Server SSH tunneling is not implemented by the provider yet",
        CapabilityFeature::Backup => "SQL Server backup/restore is not implemented by the shared backup service yet",
        feature => generic_reason(feature),
    }
}

fn generic_reason(feature: CapabilityFeature) -> &'static str {
    match feature {
        CapabilityFeature::Explain => "Explain/query plan is not available for this driver",
        CapabilityFeature::Parameters => "parameterized queries are not available for this driver",
        CapabilityFeature::DataDiff => "data diff is not available for this driver",
        CapabilityFeature::Backup => "backup/restore is not available for this driver",
        _ => "this capability is not available for the current driver",
    }
}
