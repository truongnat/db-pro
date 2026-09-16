//! PostgreSQL event trigger inventory + confirmed admin DDL (#231).

use db_pro_core::domain::connection::ConnectionHandle;
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::event_trigger::{enabled_label, validate_ident, EventTriggerInfo, EventTriggerInventory};
use db_pro_core::domain::query::CellValue;
use db_pro_core::ports::DbConnector;
use std::sync::Arc;

pub struct PostgresEventTriggerPort {
    connector: Arc<dyn DbConnector>,
}

impl PostgresEventTriggerPort {
    pub fn new(connector: Arc<dyn DbConnector>) -> Self {
        Self { connector }
    }

    pub async fn inventory(
        &self,
        handle: &ConnectionHandle,
        connection_id: &str,
    ) -> Result<EventTriggerInventory, DbError> {
        let triggers = self.list_triggers(handle).await?;
        let message = format!("{} event trigger(s) · distinct from table/row triggers", triggers.len());
        Ok(EventTriggerInventory {
            connection_id: connection_id.to_owned(),
            triggers,
            message,
        })
    }

    async fn list_triggers(&self, handle: &ConnectionHandle) -> Result<Vec<EventTriggerInfo>, DbError> {
        let result = self
            .connector
            .query(
                handle,
                r#"
SELECT
    e.evtname::text,
    e.evtevent::text,
    e.evtenabled::text,
    e.evtfoid::regprocedure::text,
    pg_catalog.pg_get_userbyid(e.evtowner)::text,
    COALESCE(array_to_string(e.evttags, ','), '')
FROM pg_catalog.pg_event_trigger e
ORDER BY e.evtname
"#,
                &[],
            )
            .await?;
        Ok(result
            .rows
            .into_iter()
            .filter_map(|row| {
                let name = cell_text(row.0.first())?;
                let enabled = cell_text(row.0.get(2)).unwrap_or_else(|| "?".into());
                let tags_raw = cell_text(row.0.get(5)).unwrap_or_default();
                let tags = tags_raw
                    .split(',')
                    .map(str::trim)
                    .filter(|t| !t.is_empty())
                    .map(str::to_owned)
                    .collect();
                Some(EventTriggerInfo {
                    name,
                    event: cell_text(row.0.get(1)).unwrap_or_default(),
                    enabled_label: enabled_label(&enabled),
                    enabled,
                    function_signature: cell_text(row.0.get(3)).unwrap_or_default(),
                    owner: cell_text(row.0.get(4)),
                    tags,
                })
            })
            .collect())
    }

    async fn execute_ddl(&self, handle: &ConnectionHandle, sql: &str) -> Result<(), DbError> {
        let trimmed = sql.trim();
        if trimmed.is_empty() {
            return Err(DbError::Validation("empty DDL".into()));
        }
        self.connector.execute(handle, trimmed, &[]).await?;
        Ok(())
    }

    pub async fn create_event_trigger(
        &self,
        handle: &ConnectionHandle,
        name: &str,
        event: &str,
        function_ref: &str,
        tags_csv: &str,
        confirmed: bool,
    ) -> Result<(), DbError> {
        if !confirmed {
            return Err(DbError::Validation(
                "creating an event trigger requires explicit confirmation".into(),
            ));
        }
        let sql = db_pro_core::domain::event_trigger::preview_create_event_trigger(name, event, function_ref, tags_csv)
            .map_err(DbError::Validation)?;
        self.execute_ddl(handle, &sql).await
    }

    pub async fn drop_event_trigger(
        &self,
        handle: &ConnectionHandle,
        name: &str,
        confirmed: bool,
    ) -> Result<(), DbError> {
        if !confirmed {
            return Err(DbError::Validation(
                "dropping an event trigger requires explicit confirmation".into(),
            ));
        }
        validate_ident(name).map_err(DbError::Validation)?;
        let sql = db_pro_core::domain::event_trigger::preview_drop_event_trigger(name).map_err(DbError::Validation)?;
        self.execute_ddl(handle, &sql).await
    }

    pub async fn alter_event_trigger(
        &self,
        handle: &ConnectionHandle,
        name: &str,
        mode: &str,
        confirmed: bool,
    ) -> Result<(), DbError> {
        if !confirmed {
            return Err(DbError::Validation(
                "altering an event trigger requires explicit confirmation".into(),
            ));
        }
        let sql =
            db_pro_core::domain::event_trigger::preview_alter_event_trigger(name, mode).map_err(DbError::Validation)?;
        self.execute_ddl(handle, &sql).await
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
