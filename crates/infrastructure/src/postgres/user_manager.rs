use std::sync::Arc;

use async_trait::async_trait;
use db_pro_core::domain::connection::ConnectionHandle;
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{CellValue, QueryParam};
use db_pro_core::domain::user::{DatabaseUser, Privilege, PrivilegeObjectKind, RoleAttributes, RoleMembership};
use db_pro_core::ports::{DbConnector, UserManager};
use uuid::Uuid;

fn quote_identifier(value: &str, field: &str) -> Result<String, DbError> {
    if value.trim().is_empty() {
        return Err(DbError::Validation(format!("{field} must not be empty")));
    }
    if value.contains('\0') {
        return Err(DbError::Validation(format!("{field} contains an invalid null byte")));
    }
    Ok(format!(r#""{}""#, value.replace('"', "\"\"")))
}

fn quote_table_privilege(value: &str) -> Result<&'static str, DbError> {
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

fn quote_schema_privilege(value: &str) -> Result<&'static str, DbError> {
    match value.trim().to_ascii_uppercase().as_str() {
        "ALL" => Ok("ALL"),
        "USAGE" => Ok("USAGE"),
        "CREATE" => Ok("CREATE"),
        _ => Err(DbError::Unsupported(format!("unsupported schema privilege: {value}"))),
    }
}

fn quote_database_privilege(value: &str) -> Result<&'static str, DbError> {
    match value.trim().to_ascii_uppercase().as_str() {
        "ALL" => Ok("ALL"),
        "CONNECT" => Ok("CONNECT"),
        "CREATE" => Ok("CREATE"),
        "TEMPORARY" | "TEMP" => Ok("TEMPORARY"),
        _ => Err(DbError::Unsupported(format!("unsupported database privilege: {value}"))),
    }
}

fn quote_sequence_privilege(value: &str) -> Result<&'static str, DbError> {
    match value.trim().to_ascii_uppercase().as_str() {
        "ALL" => Ok("ALL"),
        "USAGE" => Ok("USAGE"),
        "SELECT" => Ok("SELECT"),
        "UPDATE" => Ok("UPDATE"),
        _ => Err(DbError::Unsupported(format!("unsupported sequence privilege: {value}"))),
    }
}

fn quote_privilege_for_kind(kind: PrivilegeObjectKind, value: &str) -> Result<&'static str, DbError> {
    match kind {
        PrivilegeObjectKind::Table => quote_table_privilege(value),
        PrivilegeObjectKind::Schema => quote_schema_privilege(value),
        PrivilegeObjectKind::Database => quote_database_privilege(value),
        PrivilegeObjectKind::Sequence => quote_sequence_privilege(value),
    }
}

/// Dollar-quote a password. Callers must never log the password or returned literal.
fn dollar_quote_password(password: &str) -> Result<String, DbError> {
    if password.is_empty() {
        return Err(DbError::Validation("password must not be empty".into()));
    }
    if password.contains('\0') {
        return Err(DbError::Validation("password contains an invalid null byte".into()));
    }
    for _ in 0..8 {
        let tag = format!("pwd{}", &Uuid::new_v4().simple().to_string()[..12]);
        let open = format!("${tag}$");
        if !password.contains(&open) {
            return Ok(format!("{open}{password}{open}"));
        }
    }
    Err(DbError::Validation("unable to safely encode password literal".into()))
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
                "SELECT rolname, rolsuper, rolcreatedb, rolcreaterole, rolcanlogin \
                 FROM pg_catalog.pg_roles ORDER BY rolname",
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

    async fn alter_role(
        &self,
        handle: &ConnectionHandle,
        name: &str,
        attributes: &RoleAttributes,
    ) -> Result<(), DbError> {
        let role = quote_identifier(name, "role name")?;
        let mut clauses = Vec::new();
        if let Some(login) = attributes.login {
            clauses.push(if login { "LOGIN" } else { "NOLOGIN" });
        }
        if let Some(superuser) = attributes.superuser {
            clauses.push(if superuser { "SUPERUSER" } else { "NOSUPERUSER" });
        }
        if let Some(createdb) = attributes.createdb {
            clauses.push(if createdb { "CREATEDB" } else { "NOCREATEDB" });
        }
        if let Some(createrole) = attributes.createrole {
            clauses.push(if createrole { "CREATEROLE" } else { "NOCREATEROLE" });
        }
        if clauses.is_empty() {
            return Err(DbError::Validation("alter role requires at least one attribute".into()));
        }
        let sql = format!("ALTER ROLE {role} WITH {}", clauses.join(" "));
        self.connector.execute(handle, &sql, &[]).await?;
        Ok(())
    }

    async fn update_password(&self, handle: &ConnectionHandle, name: &str, password: &str) -> Result<(), DbError> {
        let role = quote_identifier(name, "role name")?;
        let literal = dollar_quote_password(password)?;
        // Intentionally do not log `literal` or `password`.
        let sql = format!("ALTER ROLE {role} WITH PASSWORD {literal}");
        self.connector.execute(handle, &sql, &[]).await?;
        Ok(())
    }

    async fn list_memberships(&self, handle: &ConnectionHandle, member: &str) -> Result<Vec<RoleMembership>, DbError> {
        let result = self
            .connector
            .query(
                handle,
                "SELECT r.rolname, m.rolname, am.admin_option \
                 FROM pg_auth_members am \
                 JOIN pg_roles r ON r.oid = am.roleid \
                 JOIN pg_roles m ON m.oid = am.member \
                 WHERE m.rolname = $1 \
                 ORDER BY r.rolname",
                &[QueryParam::Text(member.to_owned())],
            )
            .await?;
        Ok(result
            .rows
            .into_iter()
            .filter_map(|row| {
                let cells = &row.0;
                Some(RoleMembership {
                    role: cells.first().and_then(cell_text)?,
                    member: cells.get(1).and_then(cell_text).unwrap_or_else(|| member.to_owned()),
                    admin_option: cells.get(2).map(cell_bool).unwrap_or(false),
                })
            })
            .collect())
    }

    async fn grant_membership(&self, handle: &ConnectionHandle, role: &str, member: &str) -> Result<(), DbError> {
        let role_q = quote_identifier(role, "role")?;
        let member_q = quote_identifier(member, "member")?;
        let sql = format!("GRANT {role_q} TO {member_q}");
        self.connector.execute(handle, &sql, &[]).await?;
        Ok(())
    }

    async fn revoke_membership(&self, handle: &ConnectionHandle, role: &str, member: &str) -> Result<(), DbError> {
        let role_q = quote_identifier(role, "role")?;
        let member_q = quote_identifier(member, "member")?;
        let sql = format!("REVOKE {role_q} FROM {member_q}");
        self.connector.execute(handle, &sql, &[]).await?;
        Ok(())
    }

    async fn list_privileges(&self, handle: &ConnectionHandle, role_name: &str) -> Result<Vec<Privilege>, DbError> {
        let mut privileges = Vec::new();
        let table = self
            .connector
            .query(
                handle,
                "SELECT table_schema, table_name, privilege_type \
                 FROM information_schema.role_table_grants \
                 WHERE grantee = $1 \
                 ORDER BY table_schema, table_name, privilege_type",
                &[QueryParam::Text(role_name.to_owned())],
            )
            .await?;
        privileges.extend(table.rows.into_iter().filter_map(|row| {
            let cells = &row.0;
            Some(Privilege {
                object_kind: PrivilegeObjectKind::Table,
                schema: cells.first().and_then(cell_text)?,
                object_name: cells.get(1).and_then(cell_text)?,
                privilege_type: cells.get(2).and_then(cell_text)?,
            })
        }));

        if let Ok(usage) = self
            .connector
            .query(
                handle,
                "SELECT object_schema, object_name, object_type, privilege_type \
                 FROM information_schema.usage_privileges \
                 WHERE grantee = $1 \
                 ORDER BY object_type, object_schema, object_name, privilege_type",
                &[QueryParam::Text(role_name.to_owned())],
            )
            .await
        {
            for row in usage.rows {
                let cells = &row.0;
                let object_type = cells.get(2).and_then(cell_text).unwrap_or_default();
                let kind = match object_type.as_str() {
                    "SCHEMA" => PrivilegeObjectKind::Schema,
                    "SEQUENCE" => PrivilegeObjectKind::Sequence,
                    _ => continue,
                };
                let schema = cells.first().and_then(cell_text).unwrap_or_default();
                let object_name = if kind == PrivilegeObjectKind::Schema {
                    schema.clone()
                } else {
                    cells.get(1).and_then(cell_text).unwrap_or_default()
                };
                let privilege_type = cells.get(3).and_then(cell_text).unwrap_or_default();
                privileges.push(Privilege {
                    object_kind: kind,
                    schema,
                    object_name,
                    privilege_type,
                });
            }
        }

        if let Ok(database) = self
            .connector
            .query(
                handle,
                "SELECT d.datname, p.priv \
                 FROM pg_database d \
                 CROSS JOIN (VALUES ('CONNECT'), ('CREATE'), ('TEMPORARY')) AS p(priv) \
                 WHERE NOT d.datistemplate \
                   AND has_database_privilege($1, d.datname, p.priv) \
                 ORDER BY d.datname, p.priv",
                &[QueryParam::Text(role_name.to_owned())],
            )
            .await
        {
            privileges.extend(database.rows.into_iter().filter_map(|row| {
                let cells = &row.0;
                Some(Privilege {
                    object_kind: PrivilegeObjectKind::Database,
                    schema: String::new(),
                    object_name: cells.first().and_then(cell_text)?,
                    privilege_type: cells.get(1).and_then(cell_text)?,
                })
            }));
        }

        Ok(privileges)
    }

    async fn grant_privilege(
        &self,
        handle: &ConnectionHandle,
        role_name: &str,
        object_kind: PrivilegeObjectKind,
        schema: &str,
        object_name: &str,
        privilege: &str,
    ) -> Result<(), DbError> {
        let privilege = quote_privilege_for_kind(object_kind, privilege)?;
        let role = quote_identifier(role_name, "role name")?;
        let sql = match object_kind {
            PrivilegeObjectKind::Table => {
                let schema = quote_identifier(schema, "schema")?;
                let table = quote_identifier(object_name, "table")?;
                format!("GRANT {privilege} ON TABLE {schema}.{table} TO {role}")
            }
            PrivilegeObjectKind::Schema => {
                let schema = quote_identifier(
                    if object_name.trim().is_empty() {
                        schema
                    } else {
                        object_name
                    },
                    "schema",
                )?;
                format!("GRANT {privilege} ON SCHEMA {schema} TO {role}")
            }
            PrivilegeObjectKind::Database => {
                let database = quote_identifier(object_name, "database")?;
                format!("GRANT {privilege} ON DATABASE {database} TO {role}")
            }
            PrivilegeObjectKind::Sequence => {
                let schema = quote_identifier(schema, "schema")?;
                let sequence = quote_identifier(object_name, "sequence")?;
                format!("GRANT {privilege} ON SEQUENCE {schema}.{sequence} TO {role}")
            }
        };
        self.connector.execute(handle, &sql, &[]).await?;
        Ok(())
    }

    async fn revoke_privilege(
        &self,
        handle: &ConnectionHandle,
        role_name: &str,
        object_kind: PrivilegeObjectKind,
        schema: &str,
        object_name: &str,
        privilege: &str,
    ) -> Result<(), DbError> {
        let privilege = quote_privilege_for_kind(object_kind, privilege)?;
        let role = quote_identifier(role_name, "role name")?;
        let sql = match object_kind {
            PrivilegeObjectKind::Table => {
                let schema = quote_identifier(schema, "schema")?;
                let table = quote_identifier(object_name, "table")?;
                format!("REVOKE {privilege} ON TABLE {schema}.{table} FROM {role}")
            }
            PrivilegeObjectKind::Schema => {
                let schema = quote_identifier(
                    if object_name.trim().is_empty() {
                        schema
                    } else {
                        object_name
                    },
                    "schema",
                )?;
                format!("REVOKE {privilege} ON SCHEMA {schema} FROM {role}")
            }
            PrivilegeObjectKind::Database => {
                let database = quote_identifier(object_name, "database")?;
                format!("REVOKE {privilege} ON DATABASE {database} FROM {role}")
            }
            PrivilegeObjectKind::Sequence => {
                let schema = quote_identifier(schema, "schema")?;
                let sequence = quote_identifier(object_name, "sequence")?;
                format!("REVOKE {privilege} ON SEQUENCE {schema}.{sequence} FROM {role}")
            }
        };
        self.connector.execute(handle, &sql, &[]).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(quote_table_privilege(" select ").unwrap(), "SELECT");
        assert!(matches!(
            quote_table_privilege("SELECT; DROP TABLE users"),
            Err(DbError::Validation(_))
        ));
    }

    #[test]
    fn unsupported_sequence_privilege_is_explicit() {
        assert!(matches!(
            quote_sequence_privilege("DELETE"),
            Err(DbError::Unsupported(_))
        ));
    }

    #[test]
    fn dollar_quote_password_avoids_single_quoted_literal() {
        let quoted = dollar_quote_password("s3cret").expect("encode");
        assert!(quoted.starts_with("$pwd"));
        assert!(quoted.ends_with('$'));
        assert!(!quoted.contains("'s3cret'"));
        assert!(quoted.contains("s3cret"));
    }
}
