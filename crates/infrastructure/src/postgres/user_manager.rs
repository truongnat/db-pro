use async_trait::async_trait;
use sqlx::{FromRow, PgPool};

use db_pro_core::domain::connection::ConnectionHandle;
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::user::{DatabaseUser, Privilege};
use db_pro_core::ports::UserManager;

use crate::postgres::connector::PostgresConnector;

pub struct PostgresUserManager {
    connector: std::sync::Arc<PostgresConnector>,
}

impl PostgresUserManager {
    pub fn new(connector: std::sync::Arc<PostgresConnector>) -> Self {
        Self { connector }
    }
}

#[derive(FromRow)]
struct RoleRow {
    rolname: String,
    rolsuper: bool,
    rolcreatedb: bool,
    rolcreaterole: bool,
    rolcanlogin: bool,
}

async fn pool(connector: &PostgresConnector, handle: &ConnectionHandle) -> Result<PgPool, DbError> {
    connector
        .get_pool(handle)
        .await
        .ok_or_else(|| DbError::ConnectionFailed("no active pool for handle".into()))
}

#[async_trait]
impl UserManager for PostgresUserManager {
    async fn list_users(&self, handle: &ConnectionHandle) -> Result<Vec<DatabaseUser>, DbError> {
        let pool = pool(&self.connector, handle).await?;
        let rows: Vec<RoleRow> = sqlx::query_as(
            "SELECT rolname, rolsuper, rolcreatedb, rolcreaterole, rolcanlogin FROM pg_catalog.pg_roles ORDER BY rolname",
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| DbError::Internal(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| DatabaseUser {
                name: r.rolname,
                is_super: r.rolsuper,
                can_create_db: r.rolcreatedb,
                can_create_role: r.rolcreaterole,
                can_login: r.rolcanlogin,
            })
            .collect())
    }

    async fn create_role(&self, handle: &ConnectionHandle, name: &str, login: bool) -> Result<(), DbError> {
        let pool = pool(&self.connector, handle).await?;
        let login_clause = if login { "LOGIN" } else { "NOLOGIN" };
        let sql = format!(r#"CREATE ROLE "{name}" {login_clause}"#);
        sqlx::query(&sql)
            .execute(&pool)
            .await
            .map_err(|e| DbError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn drop_role(&self, handle: &ConnectionHandle, name: &str) -> Result<(), DbError> {
        let pool = pool(&self.connector, handle).await?;
        let sql = format!(r#"DROP ROLE "{name}""#);
        sqlx::query(&sql)
            .execute(&pool)
            .await
            .map_err(|e| DbError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn list_privileges(
        &self,
        handle: &ConnectionHandle,
        role_name: &str,
    ) -> Result<Vec<Privilege>, DbError> {
        let pool = pool(&self.connector, handle).await?;
        let rows: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT table_schema, table_name, privilege_type FROM information_schema.role_table_grants WHERE grantee = $1 ORDER BY table_schema, table_name",
        )
        .bind(role_name)
        .fetch_all(&pool)
        .await
        .map_err(|e| DbError::Internal(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|(schema, table, priv_type)| Privilege {
                schema,
                table,
                privilege_type: priv_type,
            })
            .collect())
    }

    async fn grant_privilege(
        &self,
        handle: &ConnectionHandle,
        role_name: &str,
        schema: &str,
        table: &str,
        privilege: &str,
    ) -> Result<(), DbError> {
        let pool = pool(&self.connector, handle).await?;
        let sql = format!(r#"GRANT {privilege} ON "{schema}"."{table}" TO "{role_name}""#);
        sqlx::query(&sql)
            .execute(&pool)
            .await
            .map_err(|e| DbError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn revoke_privilege(
        &self,
        handle: &ConnectionHandle,
        role_name: &str,
        schema: &str,
        table: &str,
        privilege: &str,
    ) -> Result<(), DbError> {
        let pool = pool(&self.connector, handle).await?;
        let sql = format!(r#"REVOKE {privilege} ON "{schema}"."{table}" FROM "{role_name}""#);
        sqlx::query(&sql)
            .execute(&pool)
            .await
            .map_err(|e| DbError::Internal(e.to_string()))?;
        Ok(())
    }
}
