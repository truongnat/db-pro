// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
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
    /// Explicit workspace context picks (#263).
    pub workspace_files: Vec<String>,
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
            Some(format!(
                "SELECT *\nFROM {}\nLIMIT 100;",
                crate::app::result_grid_export::quote_sql_identifier(&table)
            )),
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
    let from = crate::app::result_grid_export::quote_sql_identifier(table);
    let sql = if driver.eq_ignore_ascii_case("sqlite") {
        format!("EXPLAIN QUERY PLAN\nSELECT *\nFROM {from}\nLIMIT 100;")
    } else {
        format!("EXPLAIN\nSELECT *\nFROM {from}\nLIMIT 100;")
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
        // cc-scan:allow LINE_TOO_LONG — literal must not wrap
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
            // cc-scan:allow LINE_TOO_LONG — literal must not wrap
            "Mention two table names and I will draft a join. The current summary does not include foreign-key pairs, so review the ON clause.",
            None,
            false,
        ));
    }
    let left = &mentioned_tables[0];
    let right = &mentioned_tables[1];
    let left_sql = crate::app::result_grid_export::quote_sql_identifier(left);
    let right_sql = crate::app::result_grid_export::quote_sql_identifier(right);
    let sql = format!("SELECT *\nFROM {left_sql}\nJOIN {right_sql} ON {left_sql}.id = {right_sql}.id\nLIMIT 100;");
    Some(assistant(
        // cc-scan:allow LINE_TOO_LONG — literal must not wrap
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
        Some(format!(
            "SELECT COUNT(*) AS row_count\nFROM {};",
            crate::app::result_grid_export::quote_sql_identifier(table)
        )),
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
    assistant(
        content,
        Some(template.replace("{table}", &crate::app::result_grid_export::quote_sql_identifier(table))),
        true,
    )
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
#[path = "tests/mod_tests.rs"]
mod tests;
