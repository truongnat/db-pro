use std::sync::Arc;

use db_pro_core::application::{MultiQueryResult, StatementResultKind};
use db_pro_core::domain::agent::{
    allows_stale_document_version, execution_decision, AgentMode, AgentObjectRef, AgentSqlSafety, AgentTool,
    AgentToolInput, AgentToolOutput, AgentToolRequest, AgentToolResult, MAX_AGENT_COLUMNS_PER_TABLE,
    MAX_AGENT_EXPLAIN_CHARS, MAX_AGENT_RELATIONS, MAX_AGENT_SAMPLE_ROWS, MAX_AGENT_TABLES,
};
use db_pro_core::domain::agent_context::{
    AgentColumnContext, AgentForeignKeyContext, AgentResultSummary, AgentTableContext,
};

use crate::{DbErrorDto, DbProRuntime};
use async_trait::async_trait;
use db_pro_core::domain::agent_workflow::{
    is_tool_allowed, AgentConfirmationKind, AgentExecutionContext, AgentToolError,
};
use db_pro_core::domain::schema::{Column, ForeignKey, IntrospectResult, TableInfo};

#[derive(Clone)]
pub struct AgentToolExecutor {
    runtime: Arc<DbProRuntime>,
}

#[async_trait]
pub trait AgentToolRunner: Send + Sync {
    async fn execute(
        &self,
        request: &AgentToolRequest,
        context: &AgentExecutionContext,
    ) -> Result<AgentToolResult, AgentToolError>;
}

impl AgentToolExecutor {
    pub fn new(runtime: Arc<DbProRuntime>) -> Self {
        Self { runtime }
    }

    pub async fn execute(
        &self,
        request: &AgentToolRequest,
        context: &AgentExecutionContext,
    ) -> Result<AgentToolResult, AgentToolError> {
        validate_request(request, context)?;
        if !is_tool_allowed(context.mode, request.tool) {
            return Err(AgentToolError::PermissionDenied {
                tool: request.tool,
                mode: context.mode,
            });
        }

        let output = match request.tool {
            AgentTool::GetCurrentQuery => current_query(request, context)?,
            AgentTool::InspectSchema => self.inspect_schema(request, context).await?,
            AgentTool::InspectTable => self.inspect_table(request, context).await?,
            AgentTool::InspectColumns => self.inspect_columns(request, context).await?,
            AgentTool::InspectForeignKeys => self.inspect_foreign_keys(request, context).await?,
            AgentTool::PatchQuery => patch_preview(request, context)?,
            AgentTool::RunQuery => self.run_query(request, context).await?,
            AgentTool::InspectQueryResult => result_summary(request, context)?,
            AgentTool::ExplainQuery => self.explain_query(request, context).await?,
        };

        Ok(AgentToolResult {
            tool: request.tool,
            output,
        })
    }

    async fn inspect_schema(
        &self,
        request: &AgentToolRequest,
        context: &AgentExecutionContext,
    ) -> Result<AgentToolOutput, AgentToolError> {
        let connection_id = connection_id(context)?;
        let requested_schema = match &request.input {
            AgentToolInput::None => context.session.schema.as_deref(),
            AgentToolInput::Schema { schema } => schema.as_deref().or(context.session.schema.as_deref()),
            _ => return Err(AgentToolError::InvalidInput { tool: request.tool }),
        };
        let introspection = self
            .runtime
            .schema_api()
            .introspect(connection_id, false)
            .await
            .map_err(query_error)?;
        let (tables, views) = bounded_objects(&introspection, requested_schema);
        Ok(AgentToolOutput::Schema { tables, views })
    }

    async fn inspect_table(
        &self,
        request: &AgentToolRequest,
        context: &AgentExecutionContext,
    ) -> Result<AgentToolOutput, AgentToolError> {
        let table = table_input(request)?;
        let info = self.load_table_info(context, table).await?;
        Ok(AgentToolOutput::Table {
            table: map_table_context(&info),
            primary_key: info
                .primary_key
                .as_ref()
                .map_or_else(Vec::new, |key| key.columns.clone()),
            unique_columns: unique_columns(&info),
            indexes: info.indexes.iter().map(|index| index.name.clone()).collect(),
            foreign_keys: info.foreign_keys.iter().map(map_foreign_key).collect(),
        })
    }

    async fn inspect_columns(
        &self,
        request: &AgentToolRequest,
        context: &AgentExecutionContext,
    ) -> Result<AgentToolOutput, AgentToolError> {
        let table = table_input(request)?;
        let info = self.load_table_info(context, table).await?;
        let columns = info
            .columns
            .iter()
            .take(MAX_AGENT_COLUMNS_PER_TABLE)
            .map(map_column)
            .collect();
        Ok(AgentToolOutput::Columns {
            table: table.clone(),
            columns,
        })
    }

    async fn inspect_foreign_keys(
        &self,
        request: &AgentToolRequest,
        context: &AgentExecutionContext,
    ) -> Result<AgentToolOutput, AgentToolError> {
        let connection_id = connection_id(context)?;
        let (mut relations, table_filter) = match &request.input {
            AgentToolInput::Table { table } => (
                self.runtime
                    .schema_api()
                    .introspect(connection_id, false)
                    .await
                    .map_err(query_error)?
                    .foreign_keys,
                Some(table),
            ),
            AgentToolInput::None => (
                self.runtime
                    .schema_api()
                    .introspect(connection_id, false)
                    .await
                    .map_err(query_error)?
                    .foreign_keys,
                None,
            ),
            _ => return Err(AgentToolError::InvalidInput { tool: request.tool }),
        };
        let schema = context.session.schema.as_deref();
        relations.retain(|relation| {
            if let Some(table) = table_filter {
                return relation_matches_table(relation, table);
            }
            schema.is_none_or(|expected| relation.schema == expected)
        });
        relations.sort_by(|left, right| {
            (left.schema.as_str(), left.from_table.as_str(), left.name.as_str()).cmp(&(
                right.schema.as_str(),
                right.from_table.as_str(),
                right.name.as_str(),
            ))
        });
        relations.truncate(MAX_AGENT_RELATIONS);
        Ok(AgentToolOutput::ForeignKeys {
            relations: relations.iter().map(map_foreign_key).collect(),
        })
    }

    async fn load_table_info(
        &self,
        context: &AgentExecutionContext,
        table: &AgentObjectRef,
    ) -> Result<TableInfo, AgentToolError> {
        let connection_id = connection_id(context)?;
        let schema = table
            .schema
            .as_deref()
            .or(context.session.schema.as_deref())
            .unwrap_or("public");
        self.runtime
            .schema_api()
            .table_info(connection_id, schema, &table.name)
            .await
            .map_err(|error| schema_error(error, table))
    }

    async fn run_query(
        &self,
        request: &AgentToolRequest,
        context: &AgentExecutionContext,
    ) -> Result<AgentToolOutput, AgentToolError> {
        let AgentToolInput::Query { sql } = &request.input else {
            return Err(AgentToolError::InvalidInput { tool: request.tool });
        };
        ensure_execution_confirmation(context, sql)?;
        let connection_id = connection_id(context)?;
        let output = self
            .runtime
            .query_api()
            .execute_multi(connection_id, sql, None, context.session.schema.as_deref())
            .await
            .map_err(query_error)?;
        map_multi_query_output(output)
    }

    async fn explain_query(
        &self,
        request: &AgentToolRequest,
        context: &AgentExecutionContext,
    ) -> Result<AgentToolOutput, AgentToolError> {
        let AgentToolInput::Query { sql } = &request.input else {
            return Err(AgentToolError::InvalidInput { tool: request.tool });
        };
        ensure_execution_confirmation(context, sql)?;
        let connection_id = connection_id(context)?;
        let plan = self
            .runtime
            .query_api()
            .explain(connection_id, sql, false)
            .await
            .map_err(query_error)?;
        let plan = serde_json::to_string_pretty(&plan).map_err(|error| AgentToolError::QueryFailed {
            code: "INTERNAL_ERROR".to_owned(),
            message: format!("failed to serialize explain plan: {error}"),
            position: None,
            detail: None,
            hint: None,
        })?;
        Ok(AgentToolOutput::Explain {
            plan: truncate_chars(&plan, MAX_AGENT_EXPLAIN_CHARS),
        })
    }
}

#[async_trait]
impl AgentToolRunner for AgentToolExecutor {
    async fn execute(
        &self,
        request: &AgentToolRequest,
        context: &AgentExecutionContext,
    ) -> Result<AgentToolResult, AgentToolError> {
        self.execute(request, context).await
    }
}

fn validate_request(request: &AgentToolRequest, context: &AgentExecutionContext) -> Result<(), AgentToolError> {
    if request.session_id != context.session.id {
        return Err(AgentToolError::SessionMismatch);
    }
    if request.document_id != context.session.document_id {
        return Err(AgentToolError::DocumentMismatch);
    }
    let Some(run) = context.session.active_run.as_ref() else {
        return Err(AgentToolError::RunNotActive);
    };
    if request.run_id != run.id {
        return Err(AgentToolError::RunMismatch);
    }
    if request.document_version != run.document_version && !allows_stale_document_version(request.tool) {
        return Err(AgentToolError::StaleDocument {
            expected: run.document_version,
            actual: request.document_version,
        });
    }
    if let Some(document) = &context.document {
        if document.document_id != request.document_id {
            return Err(AgentToolError::DocumentMismatch);
        }
        if document.document_version != request.document_version && !allows_stale_document_version(request.tool) {
            return Err(AgentToolError::StaleDocument {
                expected: request.document_version,
                actual: document.document_version,
            });
        }
    }
    Ok(())
}

fn current_query(
    _request: &AgentToolRequest,
    context: &AgentExecutionContext,
) -> Result<AgentToolOutput, AgentToolError> {
    let Some(document) = &context.document else {
        return Err(AgentToolError::DocumentNotFound);
    };
    Ok(AgentToolOutput::CurrentQuery {
        document_id: document.document_id.clone(),
        document_version: document.document_version,
        sql: document.sql.clone(),
        cursor_offset: document.cursor_offset,
        selection: document.selection,
        current_statement: document.current_statement.clone(),
    })
}

fn patch_preview(
    request: &AgentToolRequest,
    context: &AgentExecutionContext,
) -> Result<AgentToolOutput, AgentToolError> {
    let Some(document) = &context.document else {
        return Err(AgentToolError::DocumentNotFound);
    };
    let AgentToolInput::Patch { patch } = &request.input else {
        return Err(AgentToolError::InvalidInput { tool: request.tool });
    };
    let proposed = patch
        .apply_to(&document.document_id, document.document_version, &document.sql)
        .map_err(AgentToolError::InvalidPatch)?;
    let (start, end) = patch.range;
    Ok(AgentToolOutput::PatchPreview {
        patch: patch.clone(),
        original: document.sql[start..end].to_owned(),
        proposed,
    })
}

fn result_summary(
    request: &AgentToolRequest,
    context: &AgentExecutionContext,
) -> Result<AgentToolOutput, AgentToolError> {
    let Some(summary) = &context.latest_result else {
        return Err(AgentToolError::ResultUnavailable);
    };
    let (max_rows, statement_index) = match &request.input {
        AgentToolInput::None => (MAX_AGENT_SAMPLE_ROWS, None),
        AgentToolInput::ResultSample {
            max_rows,
            statement_index,
        } => ((*max_rows).min(MAX_AGENT_SAMPLE_ROWS), *statement_index),
        _ => return Err(AgentToolError::InvalidInput { tool: request.tool }),
    };
    let bounded = AgentResultSummary::bounded(
        summary.columns.clone(),
        summary.sample_rows.iter().take(max_rows).cloned().collect(),
        summary.row_count,
        summary.affected_rows,
    );
    Ok(AgentToolOutput::QueryResult {
        statement_index,
        result_count: context.result_count,
        summary: bounded,
    })
}

fn table_input(request: &AgentToolRequest) -> Result<&AgentObjectRef, AgentToolError> {
    match &request.input {
        AgentToolInput::Table { table } => Ok(table),
        _ => Err(AgentToolError::InvalidInput { tool: request.tool }),
    }
}

fn connection_id(context: &AgentExecutionContext) -> Result<&str, AgentToolError> {
    context
        .session
        .connection_id
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or(AgentToolError::ConnectionUnavailable)
}

fn ensure_execution_confirmation(context: &AgentExecutionContext, sql: &str) -> Result<(), AgentToolError> {
    if !matches!(context.mode, AgentMode::Agent) {
        return Err(AgentToolError::PermissionDenied {
            tool: AgentTool::RunQuery,
            mode: context.mode,
        });
    }
    let safety = AgentSqlSafety::classify(sql);
    if execution_decision(context.mode, safety, context.allow_read_only_auto_run)
        != db_pro_core::domain::agent::AgentExecutionDecision::Allowed
        && !context.confirmed
    {
        return Err(AgentToolError::ConfirmationRequired {
            kind: match safety {
                AgentSqlSafety::ReadOnly => AgentConfirmationKind::RunReadOnly,
                AgentSqlSafety::Mutating => AgentConfirmationKind::RunMutation,
                AgentSqlSafety::Destructive => AgentConfirmationKind::RunDestructive,
                AgentSqlSafety::Unknown => AgentConfirmationKind::RunUnknown,
            },
        });
    }
    Ok(())
}

fn bounded_objects(
    introspection: &IntrospectResult,
    requested_schema: Option<&str>,
) -> (Vec<AgentObjectRef>, Vec<AgentObjectRef>) {
    let mut tables = introspection
        .tables
        .iter()
        .filter(|table| requested_schema.is_none_or(|schema| schema == table.schema))
        .map(|table| AgentObjectRef {
            schema: Some(table.schema.clone()),
            name: table.name.clone(),
        })
        .collect::<Vec<_>>();
    let mut views = introspection
        .views
        .iter()
        .filter(|view| requested_schema.is_none_or(|schema| schema == view.schema))
        .map(|view| AgentObjectRef {
            schema: Some(view.schema.clone()),
            name: view.name.clone(),
        })
        .collect::<Vec<_>>();
    tables.sort_by(object_ref_order);
    views.sort_by(object_ref_order);
    tables.truncate(MAX_AGENT_TABLES);
    views.truncate(MAX_AGENT_TABLES.saturating_sub(tables.len()));
    (tables, views)
}

fn map_table_context(info: &TableInfo) -> AgentTableContext {
    AgentTableContext {
        object: AgentObjectRef {
            schema: Some(info.table.schema.clone()),
            name: info.table.name.clone(),
        },
        columns: info
            .columns
            .iter()
            .take(MAX_AGENT_COLUMNS_PER_TABLE)
            .map(map_column)
            .collect(),
    }
}

fn map_column(column: &Column) -> AgentColumnContext {
    AgentColumnContext {
        name: column.name.clone(),
        data_type: column.data_type.clone(),
        nullable: column.nullable,
        ordinal: column.ordinal,
        default: column.default.clone(),
        is_primary_key: column.is_primary_key,
        is_unique: column.is_unique,
        is_identity: column.is_identity,
        is_generated: column.is_generated,
    }
}

fn map_foreign_key(foreign_key: &ForeignKey) -> AgentForeignKeyContext {
    AgentForeignKeyContext {
        name: foreign_key.name.clone(),
        source: AgentObjectRef {
            schema: Some(foreign_key.schema.clone()),
            name: foreign_key.from_table.clone(),
        },
        source_columns: foreign_key.from_columns.clone(),
        target: AgentObjectRef {
            schema: Some(foreign_key.to_schema.clone()),
            name: foreign_key.to_table.clone(),
        },
        target_columns: foreign_key.to_columns.clone(),
    }
}

fn unique_columns(info: &TableInfo) -> Vec<String> {
    info.columns
        .iter()
        .filter(|column| column.is_unique)
        .map(|column| column.name.clone())
        .collect()
}

fn object_ref_order(left: &AgentObjectRef, right: &AgentObjectRef) -> std::cmp::Ordering {
    (left.schema.as_deref().unwrap_or_default(), left.name.as_str())
        .cmp(&(right.schema.as_deref().unwrap_or_default(), right.name.as_str()))
}

fn map_multi_query_output(output: MultiQueryResult) -> Result<AgentToolOutput, AgentToolError> {
    if let Some((_statement_index, error)) = output.error {
        return Err(AgentToolError::QueryFailed {
            code: error.code,
            message: error.message,
            position: error.position,
            detail: error.detail,
            hint: error.hint,
        });
    }
    let result_count = output
        .result_kinds
        .iter()
        .filter(|kind| matches!(kind, StatementResultKind::ResultSet))
        .count();
    let statement_index = output
        .result_kinds
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, kind)| matches!(kind, StatementResultKind::ResultSet).then_some(index));
    let result_position = statement_index.or_else(|| output.results.len().checked_sub(1));
    let Some(result) = result_position.and_then(|index| output.results.get(index)) else {
        return Ok(AgentToolOutput::QueryResult {
            statement_index: None,
            result_count,
            summary: AgentResultSummary::bounded(Vec::new(), Vec::new(), None, Some(0)),
        });
    };
    let mut summary = AgentResultSummary::from_query_result(result);
    if result_count == 0 {
        summary.affected_rows = Some(result.row_count);
    }
    Ok(AgentToolOutput::QueryResult {
        statement_index,
        result_count,
        summary,
    })
}

fn schema_error(error: DbErrorDto, table: &AgentObjectRef) -> AgentToolError {
    if error.code == "NOT_FOUND" {
        AgentToolError::SchemaObjectNotFound {
            name: qualified_name(table),
        }
    } else {
        query_error(error)
    }
}

fn query_error(error: DbErrorDto) -> AgentToolError {
    AgentToolError::QueryFailed {
        code: error.code,
        message: error.message,
        position: error.position,
        detail: None,
        hint: None,
    }
}

fn qualified_name(object: &AgentObjectRef) -> String {
    object
        .schema
        .as_deref()
        .map_or_else(|| object.name.clone(), |schema| format!("{schema}.{}", object.name))
}

fn relation_matches_table(relation: &ForeignKey, table: &AgentObjectRef) -> bool {
    let source = AgentObjectRef {
        schema: Some(relation.schema.clone()),
        name: relation.from_table.clone(),
    };
    let target = AgentObjectRef {
        schema: Some(relation.to_schema.clone()),
        name: relation.to_table.clone(),
    };
    object_matches(&source, table) || object_matches(&target, table)
}

fn object_matches(left: &AgentObjectRef, right: &AgentObjectRef) -> bool {
    left.name.eq_ignore_ascii_case(&right.name)
        && right.schema.as_deref().is_none_or(|schema| {
            left.schema
                .as_deref()
                .is_some_and(|actual| actual.eq_ignore_ascii_case(schema))
        })
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use db_pro_core::domain::agent::{AgentDocumentSnapshot, AgentRun, AgentRunId, AgentSession, AgentSessionId};
    use db_pro_core::domain::query::{CellValue, ColumnMeta, QueryResult, Row};

    fn context() -> (AgentToolRequest, AgentExecutionContext) {
        let session_id = AgentSessionId::new();
        let run_id = AgentRunId::new();
        let session = AgentSession {
            id: session_id,
            document_id: "doc-a".to_owned(),
            connection_id: None,
            schema: Some("public".to_owned()),
            messages: Vec::new(),
            state: db_pro_core::domain::agent::AgentSessionState::Running,
            active_run: Some(AgentRun {
                id: run_id,
                document_version: 7,
            }),
        };
        let document = AgentDocumentSnapshot {
            document_id: "doc-a".to_owned(),
            document_version: 7,
            sql: "SELECT café".to_owned(),
            cursor_offset: 12,
            selection: None,
            current_statement: Some("SELECT café".to_owned()),
        };
        let request = AgentToolRequest {
            session_id,
            run_id,
            document_id: "doc-a".to_owned(),
            document_version: 7,
            tool: AgentTool::GetCurrentQuery,
            input: AgentToolInput::None,
        };
        (request, AgentExecutionContext::new(session, document, AgentMode::Ask))
    }

    #[test]
    fn request_validation_rejects_late_run() {
        let (mut request, mut context) = context();
        context.session.active_run = None;
        assert_eq!(validate_request(&request, &context), Err(AgentToolError::RunNotActive));
        request.document_version = 8;
        assert_eq!(validate_request(&request, &context), Err(AgentToolError::RunNotActive));
    }

    #[test]
    fn truncate_chars_preserves_utf8() {
        assert_eq!(truncate_chars("café 🐘", 5), "café ");
    }

    #[test]
    fn current_query_uses_document_snapshot() {
        let (request, context) = context();
        let output = current_query(&request, &context).expect("snapshot");
        assert!(matches!(
            output,
            AgentToolOutput::CurrentQuery { cursor_offset: 12, .. }
        ));
    }

    #[test]
    fn multi_result_summary_uses_latest_result_set_not_trailing_command() {
        let command = QueryResult {
            columns: Vec::new(),
            rows: Vec::new(),
            row_count: 3,
            duration_ms: 1,
        };
        let result_set = QueryResult {
            columns: vec![ColumnMeta {
                name: "name".to_owned(),
                data_type: "text".to_owned(),
                nullable: false,
            }],
            rows: vec![Row(vec![CellValue::Text("users".to_owned())])],
            row_count: 1,
            duration_ms: 1,
        };
        let output = map_multi_query_output(MultiQueryResult {
            results: vec![command, result_set],
            result_kinds: vec![StatementResultKind::Command, StatementResultKind::ResultSet],
            total_duration_ms: 2,
            error: None,
        })
        .expect("summary");
        assert!(matches!(
            output,
            AgentToolOutput::QueryResult {
                statement_index: Some(1),
                result_count: 1,
                summary: AgentResultSummary { row_count: Some(1), .. }
            }
        ));
    }

    #[test]
    fn read_only_execution_respects_explicit_auto_run_setting() {
        let (_, mut context) = context();
        context.mode = AgentMode::Agent;
        assert_eq!(
            ensure_execution_confirmation(&context, "SELECT 1"),
            Err(AgentToolError::ConfirmationRequired {
                kind: AgentConfirmationKind::RunReadOnly
            })
        );
        context.allow_read_only_auto_run = true;
        assert_eq!(ensure_execution_confirmation(&context, "SELECT 1"), Ok(()));
    }

    #[test]
    fn mixed_batch_is_never_auto_run_before_execute_multi() {
        let (_, mut context) = context();
        context.mode = AgentMode::Agent;
        context.allow_read_only_auto_run = true;
        assert_eq!(
            ensure_execution_confirmation(&context, "SELECT 1; DROP TABLE t;"),
            Err(AgentToolError::ConfirmationRequired {
                kind: AgentConfirmationKind::RunDestructive
            })
        );
        assert_eq!(ensure_execution_confirmation(&context, "SELECT 1; SELECT 2"), Ok(()));
        // An explicit confirmation still executes the batch; the guard decides
        // whether consent is needed, not whether the batch is legal.
        context.confirmed = true;
        assert_eq!(
            ensure_execution_confirmation(&context, "SELECT 1; DROP TABLE t;"),
            Ok(())
        );
    }
}
