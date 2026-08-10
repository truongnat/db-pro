use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::QueryParam;

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
        .raw_query("SELECT COALESCE(MAX(version), 0) FROM schema_version".into(), vec![])
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
/// Each migration is wrapped in a transaction for atomicity.
pub async fn migrate(handle: &SqliteHandle) -> Result<u32, DbError> {
    let current = current_version(handle).await?;

    for migration in MIGRATIONS {
        if migration.version <= current {
            continue;
        }

        // Begin transaction for atomicity.
        handle.execute_statement("BEGIN TRANSACTION".into()).await?;

        // Execute migration SQL if non-empty.
        if !migration.sql.is_empty() {
            if let Err(e) = handle.execute_statement(migration.sql.into()).await {
                handle.execute_statement("ROLLBACK".into()).await.ok();
                return Err(e);
            }
        }

        // Record the migration.
        let now = chrono::Utc::now().to_rfc3339();
        if let Err(e) = handle
            .execute_param(
                "INSERT INTO schema_version (version, applied_at, description) VALUES (?, ?, ?)".into(),
                vec![
                    QueryParam::Int64(migration.version as i64),
                    QueryParam::Text(now),
                    QueryParam::Text(migration.description.to_string()),
                ],
            )
            .await
        {
            handle.execute_statement("ROLLBACK".into()).await.ok();
            return Err(e);
        }

        // Commit transaction.
        handle.execute_statement("COMMIT".into()).await?;
    }

    current_version(handle).await
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

    #[tokio::test]
    async fn test_migration_execution() {
        use crate::meta::schema::SCHEMA;
        use crate::sqlite::actor::SqliteActor;

        // Use in-memory SQLite
        let handle = SqliteActor::spawn(":memory:").unwrap();

        // Setup schema baseline
        handle.execute_statement(SCHEMA.into()).await.unwrap();

        // Run migrate
        let final_version = migrate(&handle).await.unwrap();
        assert_eq!(final_version, LATEST_VERSION);

        // Introspect schema_version to ensure it is inserted properly
        let rows = handle
            .raw_query(
                "SELECT version, description FROM schema_version ORDER BY version ASC".into(),
                vec![],
            )
            .await
            .unwrap();
        assert_eq!(rows.len() as u32, LATEST_VERSION);
        assert_eq!(rows[0][0], "1");
        assert_eq!(rows[0][1], "initial schema with versioning table");
        assert_eq!(rows[1][0], "2");
        assert_eq!(rows[1][1], "add database and schema columns to query_history");
    }

    #[tokio::test]
    async fn test_migration_idempotency() {
        use crate::meta::schema::SCHEMA;
        use crate::sqlite::actor::SqliteActor;

        let handle = SqliteActor::spawn(":memory:").unwrap();
        handle.execute_statement(SCHEMA.into()).await.unwrap();

        // Run migrate twice
        let v1 = migrate(&handle).await.unwrap();
        let v2 = migrate(&handle).await.unwrap();

        // Should be idempotent
        assert_eq!(v1, LATEST_VERSION);
        assert_eq!(v2, LATEST_VERSION);

        // Should only have LATEST_VERSION rows in schema_version
        let rows = handle
            .raw_query("SELECT COUNT(*) FROM schema_version".into(), vec![])
            .await
            .unwrap();
        assert_eq!(rows[0][0], LATEST_VERSION.to_string());
    }

    #[tokio::test]
    async fn test_migration_upgrade_from_old_schema() {
        use crate::meta::schema::SCHEMA;
        use crate::sqlite::actor::SqliteActor;

        let handle = SqliteActor::spawn(":memory:").unwrap();

        // Simulate old schema WITHOUT database/schema columns
        let old_schema = r#"
            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT ''
            );
            CREATE TABLE IF NOT EXISTS query_history (
                id TEXT PRIMARY KEY,
                connection_id TEXT NOT NULL,
                sql TEXT NOT NULL,
                executed_at TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                row_count INTEGER NOT NULL
            );
        "#;
        handle.execute_statement(old_schema.into()).await.unwrap();

        // Insert v1 as already applied
        handle
            .raw_query(
                "INSERT INTO schema_version (version, applied_at, description) VALUES (1, '2024-01-01T00:00:00Z', 'initial')".into(),
                vec![],
            )
            .await
            .unwrap();

        // Run migrate - should apply v2 successfully
        let final_version = migrate(&handle).await.unwrap();
        assert_eq!(final_version, LATEST_VERSION);

        // Verify columns were added
        let rows = handle
            .raw_query(
                "SELECT sql FROM sqlite_master WHERE name = 'query_history'".into(),
                vec![],
            )
            .await
            .unwrap();
        let schema_sql = &rows[0][0];
        assert!(schema_sql.contains("database"), "query_history should have database column");
        assert!(schema_sql.contains("schema"), "query_history should have schema column");
    }
}
