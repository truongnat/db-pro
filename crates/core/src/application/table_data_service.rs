use std::sync::Arc;

use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::query::{CellValue, QueryResult};
use crate::domain::safety::ConnectionSafetyPolicy;
use crate::ports::{ConnectionRepository, DbConnector};

use super::registry::ConnectionRegistry;
use super::sql_builder::{self, SortClause, TableFilter};
use table_mutation_execution::TableMutationExecution;

#[path = "table_mutation_execution.rs"]
mod table_mutation_execution;

#[derive(Debug, Clone)]
pub enum TableDataMutation {
    Update {
        columns: Vec<String>,
        values: Vec<CellValue>,
        pk_columns: Vec<String>,
        pk_values: Vec<CellValue>,
    },
    Delete {
        pk_columns: Vec<String>,
        pk_values: Vec<CellValue>,
    },
    Insert {
        columns: Vec<String>,
        values: Vec<CellValue>,
    },
}

pub struct TableDataService {
    connector: Box<dyn DbConnector>,
    registry: Arc<ConnectionRegistry>,
    connections: Box<dyn ConnectionRepository>,
}

impl TableDataService {
    pub fn new(
        connector: Box<dyn DbConnector>,
        registry: Arc<ConnectionRegistry>,
        connections: Box<dyn ConnectionRepository>,
    ) -> Self {
        Self {
            connector,
            registry,
            connections,
        }
    }

    async fn safety_policy_for(&self, connection_id: &ConnectionId) -> Result<ConnectionSafetyPolicy, DbError> {
        let config = self
            .connections
            .get_config(connection_id)
            .await?
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} not found")))?;
        if config.readonly {
            Ok(ConnectionSafetyPolicy::read_only())
        } else {
            Ok(ConnectionSafetyPolicy::full_access())
        }
    }

    // allow: parameters are distinct components of the grid request (table, filter, sort, pagination)
    // — kept flat so call sites map directly to the underlying data query port.
    #[allow(clippy::too_many_arguments)]
    pub async fn fetch_rows(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
        filters: &[TableFilter],
        sorts: &[SortClause],
        limit: u64,
        offset: u64,
    ) -> Result<(QueryResult, u64), DbError> {
        let handle = self.resolve_handle(connection_id)?;
        let dialect = self.connector.dialect(&handle)?;

        let (select_sql, select_params) =
            sql_builder::build_select(dialect.as_ref(), schema, table, filters, sorts, limit, offset)?;
        let (count_sql, count_params) = sql_builder::build_count(dialect.as_ref(), schema, table, filters);

        let count_result = self.connector.query(&handle, &count_sql, &count_params).await?;
        count_result.validate().map_err(DbError::QueryFailed)?;
        let data_result = self.connector.query(&handle, &select_sql, &select_params).await?;
        data_result.validate().map_err(DbError::QueryFailed)?;

        let total_count = parse_total_count(&count_result)?;

        let duration_ms = data_result.duration_ms;
        Ok((
            QueryResult {
                duration_ms,
                ..data_result
            },
            total_count,
        ))
    }

    pub async fn fetch_row_by_pk(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
        pk_columns: &[String],
        pk_values: &[CellValue],
    ) -> Result<Option<QueryResult>, DbError> {
        let handle = self.resolve_handle(connection_id)?;
        let dialect = self.connector.dialect(&handle)?;
        let (select_sql, select_params) =
            sql_builder::build_select_by_pk(dialect.as_ref(), schema, table, pk_columns, pk_values)?;
        let data_result = self.connector.query(&handle, &select_sql, &select_params).await?;
        data_result.validate().map_err(DbError::QueryFailed)?;
        if data_result.rows.is_empty() {
            Ok(None)
        } else {
            Ok(Some(data_result))
        }
    }

    pub async fn insert_row(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
        columns: &[String],
        values: &[CellValue],
    ) -> Result<u64, DbError> {
        let policy = self.safety_policy_for(connection_id).await?;
        if policy.read_only {
            return Err(DbError::QueryFailed(
                "connection is read-only; cannot insert rows".into(),
            ));
        }
        let handle = self.resolve_handle(connection_id)?;
        let dialect = self.connector.dialect(&handle)?;
        let (sql, params) = sql_builder::build_insert(dialect.as_ref(), schema, table, columns, values)?;
        let affected_rows = self.connector.execute(&handle, &sql, &params).await?;
        if affected_rows == 0 {
            return Err(DbError::NotFound(format!("row not found in {schema}.{table}")));
        }
        Ok(affected_rows)
    }

    // allow: columns/values and pk_columns/pk_values are positional parallel arrays,
    // mapping directly to the SET/WHERE clauses of UPDATE — an options struct would obscure that positional contract.
    #[allow(clippy::too_many_arguments)]
    pub async fn update_row(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
        columns: &[String],
        values: &[CellValue],
        pk_columns: &[String],
        pk_values: &[CellValue],
    ) -> Result<u64, DbError> {
        if pk_columns.is_empty() || pk_columns.len() != pk_values.len() {
            return Err(DbError::Validation("update requires a primary key".into()));
        }
        if columns.is_empty() || columns.len() != values.len() {
            return Err(DbError::Validation(
                "update columns and values must have the same non-zero length".into(),
            ));
        }
        let policy = self.safety_policy_for(connection_id).await?;
        if policy.read_only {
            return Err(DbError::QueryFailed(
                "connection is read-only; cannot update rows".into(),
            ));
        }
        let handle = self.resolve_handle(connection_id)?;
        let dialect = self.connector.dialect(&handle)?;
        let (sql, params) =
            sql_builder::build_update(dialect.as_ref(), schema, table, columns, values, pk_columns, pk_values)?;
        let affected_rows = self.connector.execute(&handle, &sql, &params).await?;
        require_single_row_mutation(affected_rows, schema, table)
    }

    pub async fn delete_row(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
        pk_columns: &[String],
        pk_values: &[CellValue],
    ) -> Result<u64, DbError> {
        if pk_columns.is_empty() || pk_columns.len() != pk_values.len() {
            return Err(DbError::Validation("delete requires a primary key".into()));
        }
        let policy = self.safety_policy_for(connection_id).await?;
        if policy.read_only {
            return Err(DbError::QueryFailed(
                "connection is read-only; cannot delete rows".into(),
            ));
        }
        let handle = self.resolve_handle(connection_id)?;
        let dialect = self.connector.dialect(&handle)?;
        let (sql, params) = sql_builder::build_delete(dialect.as_ref(), schema, table, pk_columns, pk_values)?;
        let affected_rows = self.connector.execute(&handle, &sql, &params).await?;
        require_single_row_mutation(affected_rows, schema, table)
    }

    /// Apply a complete table-editor change set in one parameterized transaction.
    /// Mutations are ordered delete -> update -> insert so stale rows and
    /// replaced identities are handled before new rows are written.
    pub async fn apply_mutations(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
        mutations: &[TableDataMutation],
    ) -> Result<u64, DbError> {
        self.apply_mutations_detailed(connection_id, schema, table, mutations)
            .await
            .map_err(|failure| failure.error)
    }

    /// Apply mutations while preserving the provider failure metadata for UI
    /// callers that need to identify the failed staged mutation.
    #[allow(clippy::result_large_err)]
    pub async fn apply_mutations_detailed(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
        mutations: &[TableDataMutation],
    ) -> Result<u64, crate::ports::TransactionFailure> {
        TableMutationExecution::new(self, connection_id, schema, table, mutations)
            .execute()
            .await
    }

    fn resolve_handle(
        &self,
        connection_id: &ConnectionId,
    ) -> Result<crate::domain::connection::ConnectionHandle, DbError> {
        self.registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))
    }
}

fn parse_total_count(result: &QueryResult) -> Result<u64, DbError> {
    if result.columns.len() != 1 || result.rows.len() != 1 || result.row_count != 1 {
        return Err(DbError::Internal(
            "count query must return exactly one row and one column".into(),
        ));
    }

    let cell = result.rows[0]
        .0
        .first()
        .ok_or_else(|| DbError::Internal("count query returned no value".into()))?;

    match cell {
        CellValue::Int64(value) => {
            u64::try_from(*value).map_err(|_| DbError::Internal("count query returned a negative value".into()))
        }
        _ => Err(DbError::Internal("count query returned a non-integer value".into())),
    }
}

fn require_single_row_mutation(affected_rows: u64, schema: &str, table: &str) -> Result<u64, DbError> {
    match affected_rows {
        0 => Err(DbError::Conflict(format!(
            "This row was changed or deleted by another transaction in {schema}.{table}"
        ))),
        1 => Ok(affected_rows),
        count => Err(DbError::Internal(format!(
            "table mutation affected {count} rows; expected exactly one (invariant violation)"
        ))),
    }
}

#[cfg(test)]
#[path = "tests/table_data_service_tests.rs"]
mod tests;
