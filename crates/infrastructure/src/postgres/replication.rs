//! PostgreSQL logical replication inventory + confirmed admin DDL (#230).

use db_pro_core::domain::connection::ConnectionHandle;
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::CellValue;
use db_pro_core::domain::replication::{
    redact_conninfo, validate_repl_ident, PublicationInfo, ReplicationInventory, ReplicationSlotInfo, SubscriptionInfo,
};
use db_pro_core::ports::DbConnector;
use std::sync::Arc;

pub struct PostgresReplicationPort {
    connector: Arc<dyn DbConnector>,
}

impl PostgresReplicationPort {
    pub fn new(connector: Arc<dyn DbConnector>) -> Self {
        Self { connector }
    }

    pub async fn inventory(
        &self,
        handle: &ConnectionHandle,
        connection_id: &str,
    ) -> Result<ReplicationInventory, DbError> {
        let publications = self.list_publications(handle).await?;
        let subscriptions = self.list_subscriptions(handle).await?;
        let slots = self.list_slots(handle).await?;
        let message = format!(
            "{} publication(s) · {} subscription(s) · {} slot(s)",
            publications.len(),
            subscriptions.len(),
            slots.len()
        );
        Ok(ReplicationInventory {
            connection_id: connection_id.to_owned(),
            publications,
            subscriptions,
            slots,
            message,
        })
    }

    async fn list_publications(&self, handle: &ConnectionHandle) -> Result<Vec<PublicationInfo>, DbError> {
        let result = self
            .connector
            .query(
                handle,
                r#"
SELECT
    p.pubname::text,
    pg_catalog.pg_get_userbyid(p.pubowner)::text,
    p.puballtables,
    p.pubinsert,
    p.pubupdate,
    p.pubdelete,
    p.pubtruncate
FROM pg_catalog.pg_publication p
ORDER BY p.pubname
"#,
                &[],
            )
            .await?;
        let mut out = Vec::new();
        for row in result.rows {
            let name = match cell_text(row.0.first()) {
                Some(n) => n,
                None => continue,
            };
            let tables = self.publication_tables(handle, &name).await.unwrap_or_default();
            out.push(PublicationInfo {
                name,
                owner: cell_text(row.0.get(1)),
                all_tables: cell_bool(row.0.get(2)),
                insert: cell_bool(row.0.get(3)),
                update: cell_bool(row.0.get(4)),
                delete: cell_bool(row.0.get(5)),
                truncate: cell_bool(row.0.get(6)),
                tables,
            });
        }
        Ok(out)
    }

    async fn publication_tables(&self, handle: &ConnectionHandle, name: &str) -> Result<Vec<String>, DbError> {
        let result = self
            .connector
            .query(
                handle,
                r#"
SELECT schemaname::text || '.' || tablename::text
FROM pg_catalog.pg_publication_tables
WHERE pubname = $1
ORDER BY 1
"#,
                &[db_pro_core::domain::query::QueryParam::Text(name.to_owned())],
            )
            .await?;
        Ok(result
            .rows
            .into_iter()
            .filter_map(|row| cell_text(row.0.first()))
            .take(200)
            .collect())
    }

    async fn list_subscriptions(&self, handle: &ConnectionHandle) -> Result<Vec<SubscriptionInfo>, DbError> {
        let result = self
            .connector
            .query(
                handle,
                r#"
SELECT
    s.subname::text,
    pg_catalog.pg_get_userbyid(s.subowner)::text,
    s.subenabled,
    s.subslotname::text,
    COALESCE(array_to_string(s.subpublications, ','), ''),
    COALESCE(s.subconninfo, '')
FROM pg_catalog.pg_subscription s
ORDER BY s.subname
"#,
                &[],
            )
            .await?;
        Ok(result
            .rows
            .into_iter()
            .filter_map(|row| {
                let name = cell_text(row.0.first())?;
                let conninfo = cell_text(row.0.get(5)).unwrap_or_default();
                let has_conninfo = !conninfo.is_empty();
                Some(SubscriptionInfo {
                    name,
                    owner: cell_text(row.0.get(1)),
                    enabled: cell_bool(row.0.get(2)),
                    slot_name: cell_text(row.0.get(3)),
                    publications: cell_text(row.0.get(4))
                        .unwrap_or_default()
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::to_owned)
                        .collect(),
                    conninfo_redacted: if has_conninfo {
                        redact_conninfo(&conninfo)
                    } else {
                        String::new()
                    },
                    has_conninfo,
                })
            })
            .collect())
    }

    async fn list_slots(&self, handle: &ConnectionHandle) -> Result<Vec<ReplicationSlotInfo>, DbError> {
        let result = self
            .connector
            .query(
                handle,
                r#"
SELECT
    slot_name::text,
    plugin::text,
    slot_type::text,
    database::text,
    active,
    restart_lsn::text,
    confirmed_flush_lsn::text
FROM pg_catalog.pg_replication_slots
ORDER BY slot_name
"#,
                &[],
            )
            .await?;
        Ok(result
            .rows
            .into_iter()
            .filter_map(|row| {
                Some(ReplicationSlotInfo {
                    slot_name: cell_text(row.0.first())?,
                    plugin: cell_text(row.0.get(1)),
                    slot_type: cell_text(row.0.get(2)),
                    database: cell_text(row.0.get(3)),
                    active: cell_bool(row.0.get(4)),
                    restart_lsn: cell_text(row.0.get(5)),
                    confirmed_flush_lsn: cell_text(row.0.get(6)),
                })
            })
            .collect())
    }

    async fn execute_repl_ddl(&self, handle: &ConnectionHandle, sql: &str) -> Result<(), DbError> {
        let trimmed = sql.trim();
        let upper = trimmed.to_ascii_uppercase();
        if !(upper.starts_with("CREATE PUBLICATION")
            || upper.starts_with("DROP PUBLICATION")
            || upper.starts_with("ALTER PUBLICATION")
            || upper.starts_with("CREATE SUBSCRIPTION")
            || upper.starts_with("DROP SUBSCRIPTION")
            || upper.starts_with("ALTER SUBSCRIPTION"))
        {
            return Err(DbError::Validation(
                "only publication/subscription DDL is allowed on this path".into(),
            ));
        }
        if upper.contains("PASSWORD") || upper.contains("CONNINFO") && upper.contains("CREATE SUBSCRIPTION") {
            // CREATE SUBSCRIPTION with conninfo must not be logged; still allow execute but
            // require caller to have confirmed. Reject obvious password literals in SQL text
            // for defense-in-depth on this admin path except ALTER ENABLE/DISABLE.
            if upper.starts_with("CREATE SUBSCRIPTION") {
                return Err(DbError::Validation(
                    "CREATE SUBSCRIPTION with connection strings is not supported from this UI yet \
                     (conninfo is secret); use a reviewed SQL script outside the inventory panel"
                        .into(),
                ));
            }
        }
        self.connector.execute(handle, trimmed, &[]).await?;
        Ok(())
    }

    pub async fn create_publication_all(
        &self,
        handle: &ConnectionHandle,
        name: &str,
        confirmed: bool,
    ) -> Result<(), DbError> {
        if !confirmed {
            return Err(DbError::Validation(
                "creating a publication requires explicit confirmation".into(),
            ));
        }
        let sql =
            db_pro_core::domain::replication::preview_create_publication_all(name).map_err(DbError::Validation)?;
        self.execute_repl_ddl(handle, &sql).await
    }

    pub async fn drop_publication(
        &self,
        handle: &ConnectionHandle,
        name: &str,
        confirmed: bool,
    ) -> Result<(), DbError> {
        if !confirmed {
            return Err(DbError::Validation(
                "dropping a publication requires explicit confirmation".into(),
            ));
        }
        validate_repl_ident(name).map_err(DbError::Validation)?;
        let sql = db_pro_core::domain::replication::preview_drop_publication(name).map_err(DbError::Validation)?;
        self.execute_repl_ddl(handle, &sql).await
    }

    pub async fn drop_subscription(
        &self,
        handle: &ConnectionHandle,
        name: &str,
        confirmed: bool,
    ) -> Result<(), DbError> {
        if !confirmed {
            return Err(DbError::Validation(
                "dropping a subscription requires administrative confirmation".into(),
            ));
        }
        validate_repl_ident(name).map_err(DbError::Validation)?;
        let sql = db_pro_core::domain::replication::preview_drop_subscription(name).map_err(DbError::Validation)?;
        self.execute_repl_ddl(handle, &sql).await
    }
}

fn cell_text(cell: Option<&CellValue>) -> Option<String> {
    match cell? {
        CellValue::Null => None,
        CellValue::Text(s) => Some(s.clone()),
        CellValue::Int64(v) => Some(v.to_string()),
        CellValue::Bool(v) => Some(v.to_string()),
        other => Some(format!("{other:?}")),
    }
}

fn cell_bool(cell: Option<&CellValue>) -> bool {
    match cell {
        Some(CellValue::Bool(v)) => *v,
        Some(CellValue::Text(s)) => s == "t" || s.eq_ignore_ascii_case("true"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_helper_used() {
        assert!(redact_conninfo("password=x").contains("[redacted]"));
    }
}
