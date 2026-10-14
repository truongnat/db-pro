//! Tests for `../agent.rs` (moved out of the source file).
use super::*;
use db_pro_core::domain::agent_context::AgentContext as StructuredAgentContext;

#[test]
fn provider_rejects_non_https_endpoint() {
    let error = CodexProvider::new("secret", "http://localhost/responses", "model")
        .expect_err("HTTP endpoint must be rejected");
    assert!(error.to_string().contains("HTTPS"));
}

#[test]
fn provider_from_api_key_selects_provider_by_key_prefix() {
    let groq = CodexProvider::from_api_key("gsk_test").expect("Groq key should configure");
    assert_eq!(groq.provider_name(), "Groq");

    let openai = CodexProvider::from_api_key("sk-test").expect("OpenAI key should configure");
    assert_eq!(openai.provider_name(), "OpenAI");
}

#[test]
fn provider_from_api_key_rejects_blank_values() {
    let error = CodexProvider::from_api_key("  ").expect_err("blank keys must not configure");
    assert!(error.to_string().contains("cannot be empty"));
}

#[test]
fn parses_read_only_sql_without_confirmation() {
    let draft = parse_draft("Here is the query:\n```sql\nSELECT * FROM customers LIMIT 100;\n```");
    assert_eq!(draft.sql.as_deref(), Some("SELECT * FROM customers LIMIT 100;"));
    assert!(!draft.requires_confirmation);
    assert_eq!(draft.content, "Here is the query:");
}

#[test]
fn marks_mutations_and_multiple_statements_for_confirmation() {
    let mutation = parse_draft("```sql\nUPDATE customers SET name = 'x' WHERE id = 1;\n```");
    assert!(mutation.requires_confirmation);

    let multiple = parse_draft("```sql\nSELECT 1; DELETE FROM customers;\n```");
    assert!(multiple.requires_confirmation);
}

#[test]
fn context_input_contains_metadata_but_no_secret_field() {
    let context = AgentContext {
        connection_name: Some("Local SQLite".to_owned()),
        driver: "SQLite".to_owned(),
        tables: vec!["customers".to_owned()],
        columns: vec!["id".to_owned(), "name".to_owned()],
        ..Default::default()
    };
    let input = build_input("show customers", &context);
    assert!(input.contains("customers"));
    assert!(!input.contains("password"));
}

#[test]
fn prediction_normalization_removes_optional_code_fence() {
    assert_eq!(normalize_prediction("```sql\nWHERE id = 1\n```"), "WHERE id = 1");
    assert_eq!(normalize_prediction("  LIMIT 10  "), "LIMIT 10");
    assert_eq!(
        normalize_prediction("Suggestion:\nWHERE active = true"),
        "WHERE active = true"
    );
}

#[test]
fn prediction_normalization_preserves_internal_formatting() {
    assert_eq!(
        normalize_prediction("```sql\n  WHERE active = true\n    AND deleted_at IS NULL\n```"),
        "  WHERE active = true\n    AND deleted_at IS NULL"
    );
}

#[test]
fn parses_responses_function_call_into_typed_tool_call() {
    let payload: ResponsesPayload = serde_json::from_value(json!({
        "output": [{
            "type": "function_call",
            "call_id": "call-1",
            "name": "inspect_table",
            "arguments": "{\"table\":{\"schema\":\"public\",\"name\":\"users\"}}"
        }]
    }))
    .expect("valid response payload");

    let events = parse_provider_events(payload).expect("typed tool call");
    assert_eq!(events.len(), 2);
    assert_eq!(
        events[0].clone(),
        AgentProviderEvent::ToolCall(AgentToolCall {
            call_id: "call-1".to_owned(),
            tool: AgentTool::InspectTable,
            input: AgentToolInput::Table {
                table: AgentObjectRef {
                    schema: Some("public".to_owned()),
                    name: "users".to_owned(),
                },
            },
        })
    );
    assert_eq!(events[1], AgentProviderEvent::Completed);
}

#[test]
fn provider_input_keeps_tool_calls_structured() {
    let request = AgentProviderRequest {
        prompt: "inspect users".to_owned(),
        context: StructuredAgentContext {
            document_id: "doc".to_owned(),
            document_version: 1,
            connection_id: None,
            schema: Some("public".to_owned()),
            current_sql: "SELECT * FROM users".to_owned(),
            user_request: "inspect users".to_owned(),
            selected_range: None,
            referenced_tables: Vec::new(),
            foreign_keys: Vec::new(),
            diagnostics: Vec::new(),
            result_summary: None,
        },
        messages: vec![AgentProviderMessage::ToolCall(AgentToolCall {
            call_id: "call-1".to_owned(),
            tool: AgentTool::GetCurrentQuery,
            input: AgentToolInput::None,
        })],
    };

    let value = build_agent_provider_input(&request);
    assert_eq!(value[1]["type"], "function_call");
    assert_eq!(value[1]["call_id"], "call-1");
}
