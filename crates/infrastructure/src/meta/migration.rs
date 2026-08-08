use db_pro_core::domain::error::DbError;

use crate::sqlite::actor::SqliteHandle;

/// A single schema migration.
pub struct Migration {
    pub version: u32,
    pub description: &'static str,
    pub sql: &'static str,
}

/// Ordered registry of all schema migrations.
///
/// Each migration is applied in order. The `schema_version` table tracks
/// which versions have been applied. Adding a new migration:
///
/// 1. Append to the `MIGRATIONS` array below.
/// 2. Increment `LATEST_VERSION`.
/// 3. Write the SQL as `ALTER TABLE` / `CREATE TABLE IF NOT EXISTS` etc.
/// 4. Never modify an already-applied migration — only add new ones.
pub const LATEST_VERSION: u32 = 2;

pub static MIGRATIONS: &[Migration] = &[
    // v1 is the baseline schema created by `SCHEMA` constant.
    // No migration SQL needed — the initial `CREATE TABLE IF NOT EXISTS`
    // statements handle bootstrap.
    Migration {
        version: 1,
        description: "initial schema with versioning table",
        sql: "",
    },
    Migration {
        version: 2,
        description: "add database and schema columns to query_history",
        sql: "ALTER TABLE query_history ADD COLUMN database TEXT; ALTER TABLE query_history ADD COLUMN schema TEXT;",
    },
];

/// Get the current schema version from the database, or 0 if no versioning exists.
pub async fn current_version(handle: &SqliteHandle) -> Result<u32, DbError> {
    let rows = handle
        .raw_query(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version".into(),
            vec![],
        )
        .await?;

    let version = rows
        .first()
        .and_then(|row| row.first())
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    Ok(version)
}

/// Apply all pending migrations to bring the schema up to `LATEST_VERSION`.
///
/// This is idempotent: already-applied versions are skipped.
pub async fn migrate(handle: &SqliteHandle) -> Result<u32, DbError> {
    let current = current_version(handle).await?;

    for migration in MIGRATIONS {
        if migration.version <= current {
            continue;
        }

        // Execute migration SQL if non-empty.
        if !migration.sql.is_empty() {
            handle.execute_statement(migration.sql.into()).await?;
        }

        // Record the migration.
        let now = chrono::Utc::now().to_rfc3339();
        handle
            .execute_statement(
                format!(
                    "INSERT INTO schema_version (version, applied_at, description) VALUES ({}, '{}', '{}')",
                    migration.version,
                    now,
                    migration.description.replace('\'', "''"),
                )
                .into(),
            )
            .await?;
    }

    Ok(current_version(handle).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_ordered() {
        for window in MIGRATIONS.windows(2) {
            assert!(
                window[0].version < window[1].version,
                "migrations must be strictly ordered: {} >= {}",
                window[0].version,
                window[1].version,
            );
        }
    }

    #[test]
    fn latest_version_matches_registry() {
        if let Some(last) = MIGRATIONS.last() {
            assert_eq!(
                LATEST_VERSION, last.version,
                "LATEST_VERSION must match the last migration"
            );
        }
    }

    #[test]
    fn migration_descriptions_non_empty() {
        for m in MIGRATIONS {
            assert!(
                !m.description.is_empty(),
                "migration {} has empty description",
                m.version
            );
        }
    }
}
