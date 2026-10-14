//! Tests for `../agent_context.rs` (moved out of the source file).
use super::*;

fn table(schema: &str, name: &str, columns: usize) -> AgentTableContext {
    AgentTableContext {
        object: AgentObjectRef {
            schema: Some(schema.to_owned()),
            name: name.to_owned(),
        },
        columns: (0..columns)
            .map(|index| AgentColumnContext {
                name: format!("column_{index}"),
                data_type: "text".to_owned(),
                nullable: true,
                ordinal: index,
                default: None,
                is_primary_key: false,
                is_unique: false,
                is_identity: false,
                is_generated: false,
            })
            .collect(),
    }
}

#[test]
fn context_selects_sql_tables_and_one_hop_fk_neighbor() {
    let users = table("public", "users", 30);
    let orders = table("public", "orders", 2);
    let relation = AgentForeignKeyContext {
        name: "orders_user_id_fkey".to_owned(),
        source: orders.object.clone(),
        source_columns: vec!["user_id".to_owned()],
        target: users.object.clone(),
        target_columns: vec!["id".to_owned()],
    };
    let catalog = AgentSchemaCatalog {
        tables: vec![users, orders],
        foreign_keys: vec![relation],
    };
    let request = AgentContextRequest {
        document_id: "doc-1",
        document_version: 7,
        connection_id: Some("conn-1"),
        schema: Some("public"),
        current_sql: "SELECT * FROM users",
        selected_range: None,
        user_request: "show users",
        diagnostics: &[],
        result_summary: None,
        catalog: &catalog,
    };
    let context = AgentContextBuilder::default().build(&request);
    assert_eq!(context.referenced_tables.len(), 2);
    assert!(context
        .referenced_tables
        .iter()
        .all(|table| table.columns.len() <= MAX_AGENT_COLUMNS_PER_TABLE));
    assert_eq!(context.foreign_keys.len(), 1);
}

#[test]
fn result_summary_bounds_rows_columns_and_cell_text() {
    let summary = AgentResultSummary::bounded(
        (0..60)
            .map(|index| AgentResultColumn {
                name: format!("column_{index}"),
                data_type: None,
            })
            .collect(),
        vec![vec!["x".repeat(MAX_AGENT_CELL_CHARS + 10); 60]; MAX_AGENT_SAMPLE_ROWS + 1],
        Some(1000),
        None,
    );
    assert_eq!(summary.columns.len(), MAX_AGENT_RESULT_COLUMNS);
    assert_eq!(summary.sample_rows.len(), MAX_AGENT_SAMPLE_ROWS);
    assert_eq!(summary.sample_rows[0][0].chars().count(), MAX_AGENT_CELL_CHARS);
    assert!(summary.truncated);
}

#[test]
fn context_serialization_is_bounded() {
    let catalog = AgentSchemaCatalog {
        tables: vec![table("public", "users", 24)],
        foreign_keys: Vec::new(),
    };
    let sql = format!("SELECT {}", "x".repeat(20_000));
    let request = AgentContextRequest {
        document_id: "doc",
        document_version: 1,
        connection_id: None,
        schema: Some("public"),
        current_sql: &sql,
        selected_range: None,
        user_request: "users",
        diagnostics: &[],
        result_summary: None,
        catalog: &catalog,
    };
    let context = AgentContextBuilder {
        max_context_chars: 1000,
        ..Default::default()
    }
    .build(&request);
    assert!(
        serde_json::to_string(&context)
            .expect("context serializes")
            .chars()
            .count()
            <= 1000
    );
}
