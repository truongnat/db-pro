use async_trait::async_trait;
use std::sync::Arc;

use db_pro_core::domain::connection::ConnectionHandle;
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{CellValue, QueryParam};
use db_pro_core::domain::user::{DatabaseUser, Privilege};
use db_pro_core::ports::{DbConnector, UserManager};

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
    connector: Arc<dyn DbConnector>,
}

impl PostgresUserManager {
    pub fn new(connector: Arc<dyn DbConnector>) -> Self {
        Self { connector }
    }
}

fn cell_text(cell: &CellValue) -> Option<String> {
    match cell {
        CellValue::Text(s) => Some(s.clone()),
        CellValue::Null => None,
        other => Some(format!("{other:?}")),
    }
}

fn cell_bool(cell: &CellValue) -> bool {
    matches!(cell, CellValue::Bool(true)) || matches!(cell, CellValue::Text(s) if s == "t" || s == "true")
}

#[async_trait]
impl UserManager for PostgresUserManager {
    async fn list_users(&self, handle: &ConnectionHandle) -> Result<Vec<DatabaseUser>, DbError> {
        let result = self
            .connector
            .query(
                handle,
                "SELECT rolname, rolsuper, rolcreatedb, rolcreaterole, rolcanlogin FROM pg_catalog.pg_roles ORDER BY rolname",
                &[],
            )
            .await?;
        Ok(result
            .rows
            .into_iter()
            .filter_map(|row| {
                let cells = &row.0;
                Some(DatabaseUser {
                    name: cells.first().and_then(cell_text)?,
                    is_super: cells.get(1).map(cell_bool).unwrap_or(false),
                    can_create_db: cells.get(2).map(cell_bool).unwrap_or(false),
                    can_create_role: cells.get(3).map(cell_bool).unwrap_or(false),
                    can_login: cells.get(4).map(cell_bool).unwrap_or(false),
                })
            })
            .collect())
    }

    async fn create_role(&self, handle: &ConnectionHandle, name: &str, login: bool) -> Result<(), DbError> {
        let role = quote_identifier(name, "role name")?;
        let login_clause = if login { "LOGIN" } else { "NOLOGIN" };
        let sql = format!("CREATE ROLE {role} {login_clause}");
        self.connector.execute(handle, &sql, &[]).await?;
        Ok(())
    }

    async fn drop_role(&self, handle: &ConnectionHandle, name: &str) -> Result<(), DbError> {
        let role = quote_identifier(name, "role name")?;
        let sql = format!("DROP ROLE {role}");
        self.connector.execute(handle, &sql, &[]).await?;
        Ok(())
    }

    async fn list_privileges(&self, handle: &ConnectionHandle, role_name: &str) -> Result<Vec<Privilege>, DbError> {
        let result = self
            .connector
            .query(
                handle,
                "SELECT table_schema, table_name, privilege_type FROM information_schema.role_table_grants WHERE grantee = $1 ORDER BY table_schema, table_name",
                &[QueryParam::Text(role_name.to_owned())],
            )
            .await?;
        Ok(result
            .rows
            .into_iter()
            .filter_map(|row| {
                let cells = &row.0;
                Some(Privilege {
                    schema: cells.first().and_then(cell_text)?,
                    table: cells.get(1).and_then(cell_text)?,
                    privilege_type: cells.get(2).and_then(cell_text)?,
                })
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
        let privilege = quote_privilege(privilege)?;
        let schema = quote_identifier(schema, "schema")?;
        let table = quote_identifier(table, "table")?;
        let role = quote_identifier(role_name, "role name")?;
        let sql = format!("GRANT {privilege} ON {schema}.{table} TO {role}");
        self.connector.execute(handle, &sql, &[]).await?;
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
        let privilege = quote_privilege(privilege)?;
        let schema = quote_identifier(schema, "schema")?;
        let table = quote_identifier(table, "table")?;
        let role = quote_identifier(role_name, "role name")?;
        let sql = format!("REVOKE {privilege} ON {schema}.{table} FROM {role}");
        self.connector.execute(handle, &sql, &[]).await?;
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
