//! Pure mapping from UI query/schema data to the core Agent context contract.

use db_pro_core::domain::agent::{AgentDocumentSnapshot, AgentObjectRef};
use db_pro_core::domain::agent_context::{
    AgentColumnContext, AgentContext as CoreAgentContext, AgentContextBuilder, AgentContextRequest,
    AgentDiagnosticContext, AgentForeignKeyContext, AgentSchemaCatalog, AgentTableContext,
};

use crate::query::QueryDocument;
use crate::UiTableSummary;

pub(super) fn document_snapshot(document: &QueryDocument) -> AgentDocumentSnapshot {
    let selection = (!document.selection.is_empty()).then(|| document.selection.normalized());
    AgentDocumentSnapshot {
        document_id: document.id.clone(),
        document_version: document.buffer.version(),
        sql: document.text().to_owned(),
        cursor_offset: document.cursor.offset,
        selection,
        current_statement: document
            .analysis
            .current_statement_at(document.cursor.offset)
            .map(|statement| statement.text.clone()),
    }
}

pub(super) fn build_context(
    prompt: &str,
    document: &QueryDocument,
    connection_id: Option<&str>,
    schema: Option<&str>,
    table_details: &[UiTableSummary],
) -> CoreAgentContext {
    let tables = table_details
        .iter()
        .map(|table| AgentTableContext {
            object: AgentObjectRef {
                schema: Some(table.schema.clone()),
                name: table.name.clone(),
            },
            columns: table
                .columns
                .iter()
                .enumerate()
                .map(|(ordinal, column)| AgentColumnContext {
                    name: column.name.clone(),
                    data_type: column.data_type.clone(),
                    nullable: column.nullable,
                    ordinal,
                    default: None,
                    is_primary_key: column.is_primary_key,
                    is_unique: false,
                    is_identity: false,
                    is_generated: false,
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    let foreign_keys = table_details
        .iter()
        .flat_map(|table| {
            table.foreign_keys.iter().map(|foreign_key| AgentForeignKeyContext {
                name: foreign_key.name.clone(),
                source: AgentObjectRef {
                    schema: Some(table.schema.clone()),
                    name: table.name.clone(),
                },
                source_columns: foreign_key.from_columns.clone(),
                target: AgentObjectRef {
                    schema: Some(foreign_key.to_schema.clone()),
                    name: foreign_key.to_table.clone(),
                },
                target_columns: foreign_key.to_columns.clone(),
            })
        })
        .collect::<Vec<_>>();
    let catalog = AgentSchemaCatalog { tables, foreign_keys };
    let diagnostics = document
        .diagnostics
        .iter()
        .map(|diagnostic| AgentDiagnosticContext {
            message: diagnostic.message.clone(),
            range: Some(diagnostic.range),
        })
        .collect::<Vec<_>>();

    AgentContextBuilder::default().build(&AgentContextRequest {
        document_id: &document.id,
        document_version: document.buffer.version(),
        connection_id,
        schema,
        current_sql: document.text(),
        selected_range: (!document.selection.is_empty()).then(|| document.selection.normalized()),
        user_request: prompt,
        diagnostics: &diagnostics,
        result_summary: None,
        catalog: &catalog,
    })
}
