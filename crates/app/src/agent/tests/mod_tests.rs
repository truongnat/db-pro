//! Tests for `../mod.rs` (moved out of the source file).
use super::*;

fn context() -> AgentContext {
    AgentContext {
        connection_name: Some("Local PostgreSQL".to_owned()),
        driver: "Postgres".to_owned(),
        tables: vec!["customers".to_owned(), "orders".to_owned()],
        columns: vec!["id".to_owned(), "name".to_owned()],
        ..Default::default()
    }
}

#[test]
fn offline_provider_is_explicit_preview() {
    let info = OfflineAgentProvider.info();
    assert_eq!(info.kind, AgentProviderKind::OfflineDraft);
    assert_eq!(info.state, AgentProviderState::Preview);
    assert!(info.detail.contains("unexecuted"));
}

#[test]
fn provider_returns_a_draft_without_an_execution_handle() {
    let message = OfflineAgentProvider
        .respond("show customers", &context())
        .expect("offline provider should respond");
    assert_eq!(message.sql.as_deref(), Some("SELECT *\nFROM \"customers\"\nLIMIT 100;"));
    assert!(!message.requires_confirmation);
}

#[test]
fn empty_schema_explains_how_to_recover() {
    let message = respond("show tables", &AgentContext::default());
    assert!(message.content.contains("do not see any tables"));
    assert!(message.sql.is_none());
}

#[test]
fn table_prompt_generates_read_only_sql() {
    let message = respond("show customers", &context());
    assert_eq!(message.sql.as_deref(), Some("SELECT *\nFROM \"customers\"\nLIMIT 100;"));
    assert!(!message.requires_confirmation);
}

#[test]
fn sqlite_plan_uses_query_plan_without_execution() {
    let mut context = context();
    context.driver = "SQLite".to_owned();
    let message = respond("explain customers", &context);
    assert!(message.sql.unwrap().starts_with("EXPLAIN QUERY PLAN"));
}

#[test]
fn mutation_is_marked_for_confirmation() {
    let message = respond("update customers", &context());
    assert!(message.requires_confirmation);
    assert!(message.sql.unwrap().contains("UPDATE \"customers\""));
}

#[test]
fn unknown_table_gets_a_recovery_prompt() {
    let message = respond("show invoices", &context());
    assert!(message.sql.is_none());
    assert!(message.content.contains("customers"));
}
