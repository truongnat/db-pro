#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentMessage {
    pub role: AgentRole,
    pub content: String,
    pub sql: Option<String>,
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentContext {
    pub connection_name: Option<String>,
    pub driver: String,
    pub tables: Vec<String>,
    pub columns: Vec<String>,
    pub schema: Option<String>,
    pub selected_table: Option<String>,
    pub selected_columns: Vec<String>,
    pub current_sql: String,
    pub result_summary: Option<String>,
    pub explain_plan: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentProviderKind {
    OfflineDraft,
    Codex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentProviderState {
    Preview,
    Ready,
    NotConfigured,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentProviderInfo {
    pub kind: AgentProviderKind,
    pub state: AgentProviderState,
    pub label: &'static str,
    pub detail: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProviderError {
    pub message: String,
}

impl std::fmt::Display for AgentProviderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

/// Provider boundary for the native Agent workspace.
///
/// Providers return inspectable drafts only. Query execution, mutation
/// confirmation, and database writes remain separate runtime operations.
pub trait AgentProvider: Send + Sync {
    fn info(&self) -> AgentProviderInfo;

    fn respond(&self, prompt: &str, context: &AgentContext) -> Result<AgentMessage, AgentProviderError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct OfflineAgentProvider;

impl AgentProvider for OfflineAgentProvider {
    fn info(&self) -> AgentProviderInfo {
        AgentProviderInfo {
            kind: AgentProviderKind::OfflineDraft,
            state: AgentProviderState::Preview,
            label: "Offline draft",
            detail: "Pure Rust responder · writes stay unexecuted",
        }
    }

    fn respond(&self, prompt: &str, context: &AgentContext) -> Result<AgentMessage, AgentProviderError> {
        Ok(respond(prompt, context))
    }
}

pub fn respond(prompt: &str, context: &AgentContext) -> AgentMessage {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return assistant("Tell me what you want to inspect, query, or change.", None, false);
    }
    if context.tables.is_empty() {
        return assistant(
            "I do not see any tables in the current connection. Connect to a database and refresh the schema first.",
            None,
            false,
        );
    }

    let lower = prompt.to_lowercase();
    let table = find_table(&lower, &context.tables);
    let mentioned_tables = context
        .tables
        .iter()
        .filter(|candidate| lower.contains(&candidate.to_lowercase()))
        .cloned()
        .collect::<Vec<_>>();

    if let Some(message) = mutation_response(&lower, table.as_deref()) {
        return message;
    }
    if let Some(message) = explain_response(&lower, table.as_deref(), &context.driver) {
        return message;
    }
    if let Some(message) = overview_response(&lower, context) {
        return message;
    }
    if let Some(message) = relationship_response(&lower, &mentioned_tables) {
        return message;
    }
    if let Some(message) = count_response(&lower, table.as_deref()) {
        return message;
    }
    if let Some(table) = table {
        return assistant(
            &format!("Here is a read-only query draft for `{table}`."),
            Some(format!("SELECT *\nFROM {table}\nLIMIT 100;")),
            false,
        );
    }

    assistant(
        &format!(
            "I can draft a query for one of these tables: {}. Mention a table name or ask for an overview.",
            context
                .tables
                .iter()
                .take(10)
                .map(|table| format!("`{table}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        None,
        false,
    )
}

fn mutation_response(prompt: &str, table: Option<&str>) -> Option<AgentMessage> {
    if contains_any(prompt, &["delete", "remove", "drop row"]) {
        return Some(mutation_draft(
            table,
            "DELETE FROM {table}\nWHERE id = ?;",
            "I drafted a delete statement. Review the target and WHERE clause before running it.",
        ));
    }
    if contains_any(prompt, &["update", "modify", "change", "edit", "set "]) {
        return Some(mutation_draft(
            table,
            "UPDATE {table}\nSET column_name = ?\nWHERE id = ?;",
            "I drafted an update statement. Review the target, values, and WHERE clause before running it.",
        ));
    }
    if contains_any(prompt, &["insert", "add row", "create row"]) {
        return Some(mutation_draft(
            table,
            "INSERT INTO {table} (column_name)\nVALUES (?);",
            "I drafted an insert statement. Review the target and values before running it.",
        ));
    }
    None
}

fn explain_response(prompt: &str, table: Option<&str>, driver: &str) -> Option<AgentMessage> {
    if !contains_any(prompt, &["explain", "query plan", "optimize", "slow", "performance"]) {
        return None;
    }
    let Some(table) = table else {
        return Some(assistant(
            "Which table should I inspect? Mention its name so I can draft a query plan request.",
            None,
            false,
        ));
    };
    let sql = if driver.eq_ignore_ascii_case("sqlite") {
        format!("EXPLAIN QUERY PLAN\nSELECT *\nFROM {table}\nLIMIT 100;")
    } else {
        format!("EXPLAIN\nSELECT *\nFROM {table}\nLIMIT 100;")
    };
    Some(assistant(
        &format!("Here is a read-only query plan draft for `{table}`."),
        Some(sql),
        false,
    ))
}

fn overview_response(prompt: &str, context: &AgentContext) -> Option<AgentMessage> {
    if !contains_any(
        prompt,
        &["overview", "schema", "structure", "tables", "columns", "describe"],
    ) {
        return None;
    }
    let table_list = context
        .tables
        .iter()
        .take(20)
        .map(|table| format!("- `{table}`"))
        .collect::<Vec<_>>()
        .join("\n");
    let sql = if context.driver.eq_ignore_ascii_case("sqlite") {
        "SELECT name\nFROM sqlite_master\nWHERE type = 'table'\nORDER BY name;".to_owned()
    } else {
        "SELECT table_name, column_name, data_type\nFROM information_schema.columns\nWHERE table_schema = 'public'\nORDER BY table_name, ordinal_position;"
            .to_owned()
    };
    Some(assistant(
        &format!(
            "I found {} table(s) in {}.\n\n{}",
            context.tables.len(),
            connection_label(context),
            table_list
        ),
        Some(sql),
        false,
    ))
}

fn relationship_response(prompt: &str, mentioned_tables: &[String]) -> Option<AgentMessage> {
    if !contains_any(prompt, &["join", "relate", "relationship", "foreign key"]) {
        return None;
    }
    if mentioned_tables.len() < 2 {
        return Some(assistant(
            "Mention two table names and I will draft a join. The current summary does not include foreign-key pairs, so review the ON clause.",
            None,
            false,
        ));
    }
    let left = &mentioned_tables[0];
    let right = &mentioned_tables[1];
    let sql = format!("SELECT *\nFROM {left}\nJOIN {right} ON {left}.id = {right}.id\nLIMIT 100;");
    Some(assistant(
        &format!("I drafted a join for `{left}` and `{right}`. The ON clause is a placeholder because foreign-key details are not in the summary context."),
        Some(sql),
        false,
    ))
}

fn count_response(prompt: &str, table: Option<&str>) -> Option<AgentMessage> {
    if !contains_any(prompt, &["count", "how many", "number of"]) {
        return None;
    }
    let Some(table) = table else {
        return Some(assistant(
            "Which table should I count? Mention its name in the request.",
            None,
            false,
        ));
    };
    Some(assistant(
        &format!("Here is a count query for `{table}`."),
        Some(format!("SELECT COUNT(*) AS row_count\nFROM {table};")),
        false,
    ))
}

fn mutation_draft(table: Option<&str>, template: &str, content: &str) -> AgentMessage {
    let Some(table) = table else {
        return assistant(
            "Which table should this change target? Mention its name first.",
            None,
            false,
        );
    };
    assistant(content, Some(template.replace("{table}", table)), true)
}

fn assistant(content: &str, sql: Option<String>, requires_confirmation: bool) -> AgentMessage {
    AgentMessage {
        role: AgentRole::Assistant,
        content: content.to_owned(),
        sql,
        requires_confirmation,
    }
}

fn find_table(prompt: &str, tables: &[String]) -> Option<String> {
    tables
        .iter()
        .find(|table| prompt.split_whitespace().any(|word| word == table.to_lowercase()))
        .cloned()
        .or_else(|| {
            tables
                .iter()
                .find(|table| prompt.contains(&table.to_lowercase()))
                .cloned()
        })
        .or_else(|| (tables.len() == 1).then(|| tables[0].clone()))
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn connection_label(context: &AgentContext) -> &str {
    context.connection_name.as_deref().unwrap_or("the active connection")
}

#[cfg(test)]
mod tests {
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
        assert_eq!(message.sql.as_deref(), Some("SELECT *\nFROM customers\nLIMIT 100;"));
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
        assert_eq!(message.sql.as_deref(), Some("SELECT *\nFROM customers\nLIMIT 100;"));
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
        assert!(message.sql.unwrap().contains("UPDATE customers"));
    }

    #[test]
    fn unknown_table_gets_a_recovery_prompt() {
        let message = respond("show invoices", &context());
        assert!(message.sql.is_none());
        assert!(message.content.contains("customers"));
    }
}
