use async_trait::async_trait;
use sqlx::{FromRow, PgPool};

use db_pro_core::domain::connection::ConnectionHandle;
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::user::{DatabaseUser, Privilege};
use db_pro_core::ports::UserManager;

use crate::postgres::connector::{with_query_timeout, PostgresConnector};

fn quote_identifier(value: &str, field: &str) -> Result<String, DbError> {
    if value.trim().is_empty() {
        return Err(DbError::Validation(format!("{field} must not be empty")));
    }
    if value.contains('\0') {
        return Err(DbError::Validation(format!("{field} contains an invalid null byte")));
    }
    Ok(format!(r#""{}""#, value.replace('"', "\"\"")))
}

fn quote_privilege(value: &str) -> Result<&'static str, DbError> {
    match value.trim().to_ascii_uppercase().as_str() {
        "ALL" => Ok("ALL"),
        "SELECT" => Ok("SELECT"),
        "INSERT" => Ok("INSERT"),
        "UPDATE" => Ok("UPDATE"),
        "DELETE" => Ok("DELETE"),
        "TRUNCATE" => Ok("TRUNCATE"),
        "REFERENCES" => Ok("REFERENCES"),
        "TRIGGER" => Ok("TRIGGER"),
        _ => Err(DbError::Validation(format!("unsupported table privilege: {value}"))),
    }
}

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
        let timeout = self.connector.query_timeout(handle).await?;
        let rows: Vec<RoleRow> = with_query_timeout(timeout, async {
            sqlx::query_as(
                "SELECT rolname, rolsuper, rolcreatedb, rolcreaterole, rolcanlogin FROM pg_catalog.pg_roles ORDER BY rolname",
            )
            .fetch_all(&pool)
            .await
            .map_err(crate::error::from_sqlx)
        })
        .await?;

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
        let role = quote_identifier(name, "role name")?;
        let login_clause = if login { "LOGIN" } else { "NOLOGIN" };
        let sql = format!("CREATE ROLE {role} {login_clause}");
        let timeout = self.connector.query_timeout(handle).await?;
        with_query_timeout(timeout, async {
            sqlx::query(&sql).execute(&pool).await.map_err(crate::error::from_sqlx)
        })
        .await?;
        Ok(())
    }

    async fn drop_role(&self, handle: &ConnectionHandle, name: &str) -> Result<(), DbError> {
        let pool = pool(&self.connector, handle).await?;
        let role = quote_identifier(name, "role name")?;
        let sql = format!("DROP ROLE {role}");
        let timeout = self.connector.query_timeout(handle).await?;
        with_query_timeout(timeout, async {
            sqlx::query(&sql).execute(&pool).await.map_err(crate::error::from_sqlx)
        })
        .await?;
        Ok(())
    }

    async fn list_privileges(&self, handle: &ConnectionHandle, role_name: &str) -> Result<Vec<Privilege>, DbError> {
        let pool = pool(&self.connector, handle).await?;
        let timeout = self.connector.query_timeout(handle).await?;
        let rows: Vec<(String, String, String)> = with_query_timeout(timeout, async {
            sqlx::query_as(
                "SELECT table_schema, table_name, privilege_type FROM information_schema.role_table_grants WHERE grantee = $1 ORDER BY table_schema, table_name",
            )
                .bind(role_name)
                .fetch_all(&pool)
                .await
                .map_err(crate::error::from_sqlx)
        })
        .await?;

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
        let privilege = quote_privilege(privilege)?;
        let schema = quote_identifier(schema, "schema")?;
        let table = quote_identifier(table, "table")?;
        let role = quote_identifier(role_name, "role name")?;
        let sql = format!("GRANT {privilege} ON {schema}.{table} TO {role}");
        let timeout = self.connector.query_timeout(handle).await?;
        with_query_timeout(timeout, async {
            sqlx::query(&sql).execute(&pool).await.map_err(crate::error::from_sqlx)
        })
        .await?;
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
        let privilege = quote_privilege(privilege)?;
        let schema = quote_identifier(schema, "schema")?;
        let table = quote_identifier(table, "table")?;
        let role = quote_identifier(role_name, "role name")?;
        let sql = format!("REVOKE {privilege} ON {schema}.{table} FROM {role}");
        let timeout = self.connector.query_timeout(handle).await?;
        with_query_timeout(timeout, async {
            sqlx::query(&sql).execute(&pool).await.map_err(crate::error::from_sqlx)
        })
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{quote_identifier, quote_privilege};
    use db_pro_core::domain::error::DbError;

    #[test]
    fn quote_identifier_escapes_embedded_quotes() {
        assert_eq!(quote_identifier(r#"role"name"#, "role").unwrap(), r#""role""name""#);
    }

    #[test]
    fn quote_identifier_rejects_empty_and_null_values() {
        assert!(matches!(quote_identifier(" ", "role"), Err(DbError::Validation(_))));
        assert!(matches!(
            quote_identifier("role\0name", "role"),
            Err(DbError::Validation(_))
        ));
    }

    #[test]
    fn quote_privilege_allows_known_table_privileges_only() {
        assert_eq!(quote_privilege(" select ").unwrap(), "SELECT");
        assert!(matches!(
            quote_privilege("SELECT; DROP TABLE users"),
            Err(DbError::Validation(_))
        ));
    }
}
