use super::{TableDataMutation, TableDataService};
use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::safety::ConnectionSafetyPolicy;
use crate::ports::dialect::SqlDialect;
use crate::ports::{
    ParameterizedTransactionStatement, TransactionFailure, TransactionFailureOutcome, TransactionFailurePhase,
    TransactionStatementResult,
};

pub(super) struct TableMutationExecution<'a> {
    service: &'a TableDataService,
    connection_id: &'a ConnectionId,
    schema: &'a str,
    table: &'a str,
    mutations: &'a [TableDataMutation],
}

impl<'a> TableMutationExecution<'a> {
    pub(super) fn new(
        service: &'a TableDataService,
        connection_id: &'a ConnectionId,
        schema: &'a str,
        table: &'a str,
        mutations: &'a [TableDataMutation],
    ) -> Self {
        Self {
            service,
            connection_id,
            schema,
            table,
            mutations,
        }
    }

    pub(super) async fn execute(&self) -> Result<u64, TransactionFailure> {
        if self.mutations.is_empty() {
            return Ok(0);
        }

        let policy = self
            .service
            .safety_policy_for(self.connection_id)
            .await
            .map_err(|error| self.validation_failure(error))?;
        self.ensure_writable(&policy)
            .map_err(|error| self.validation_failure(error))?;

        let handle = self
            .service
            .resolve_handle(self.connection_id)
            .map_err(|error| self.validation_failure(error))?;
        let dialect = self
            .service
            .connector
            .dialect(&handle)
            .map_err(|error| self.validation_failure(error))?;
        let indexed_mutations = self.indexed_mutations();
        let statements = self
            .build_statements(dialect.as_ref(), &indexed_mutations)
            .map_err(|error| self.validation_failure(error))?;
        let results = self
            .service
            .connector
            .execute_parameterized_transaction(&handle, &statements)
            .await
            .map_err(|mut failure| {
                if failure.phase == TransactionFailurePhase::Statement
                    && failure.statement_index < indexed_mutations.len()
                {
                    failure.statement_index = indexed_mutations[failure.statement_index].0;
                }
                failure
            })?;

        self.total_affected_rows(results)
            .map_err(|error| self.validation_failure(error))
    }

    fn ensure_writable(&self, policy: &ConnectionSafetyPolicy) -> Result<(), DbError> {
        if policy.read_only {
            return Err(DbError::QueryFailed(
                "connection is read-only; cannot apply table changes".into(),
            ));
        }
        Ok(())
    }

    fn indexed_mutations(&self) -> Vec<(usize, &TableDataMutation)> {
        let mut indexed_mutations: Vec<_> = self.mutations.iter().enumerate().collect();
        indexed_mutations.sort_by_key(|(_, mutation)| match mutation {
            TableDataMutation::Delete { .. } => 0,
            TableDataMutation::Update { .. } => 1,
            TableDataMutation::Insert { .. } => 2,
        });
        indexed_mutations
    }

    fn build_statements(
        &self,
        dialect: &dyn SqlDialect,
        indexed_mutations: &[(usize, &TableDataMutation)],
    ) -> Result<Vec<ParameterizedTransactionStatement>, DbError> {
        indexed_mutations
            .iter()
            .map(|(_, mutation)| {
                let (sql, params) = match mutation {
                    TableDataMutation::Update {
                        columns,
                        values,
                        pk_columns,
                        pk_values,
                    } => super::sql_builder::build_update(
                        dialect,
                        self.schema,
                        self.table,
                        columns,
                        values,
                        pk_columns,
                        pk_values,
                    ),
                    TableDataMutation::Delete { pk_columns, pk_values } => {
                        super::sql_builder::build_delete(dialect, self.schema, self.table, pk_columns, pk_values)
                    }
                    TableDataMutation::Insert { columns, values } => {
                        super::sql_builder::build_insert(dialect, self.schema, self.table, columns, values)
                    }
                }?;
                Ok(ParameterizedTransactionStatement {
                    sql,
                    params,
                    expect_affected_rows: true,
                    max_affected_rows: Some(1),
                })
            })
            .collect()
    }

    fn total_affected_rows(&self, results: Vec<TransactionStatementResult>) -> Result<u64, DbError> {
        results.into_iter().try_fold(0_u64, |total, result| match result {
            TransactionStatementResult::Affected { row_count, .. } => total
                .checked_add(row_count)
                .ok_or_else(|| DbError::Internal("affected row count overflow".into())),
            TransactionStatementResult::Query(_) => Err(DbError::Internal(
                "table mutation transaction returned a query result".into(),
            )),
        })
    }

    fn validation_failure(&self, error: DbError) -> TransactionFailure {
        TransactionFailure {
            phase: TransactionFailurePhase::Validation,
            statement_index: 0,
            outcome: TransactionFailureOutcome::NotStarted,
            results: Vec::new(),
            error,
        }
    }
}
