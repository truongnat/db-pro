use std::time::Duration;

use db_pro_core::domain::safety::{classify_statement_safety, StatementSafety};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;

pub(crate) const DEFAULT_ENDPOINT: &str = "https://api.openai.com/v1/responses";
pub(crate) const DEFAULT_MODEL: &str = "gpt-5.6";
pub(crate) const DEFAULT_GROQ_ENDPOINT: &str = "https://api.groq.com/openai/v1/responses";
pub(crate) const DEFAULT_GROQ_MODEL: &str = "openai/gpt-oss-120b";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const CODEX_INSTRUCTIONS: &str = "You are the DB Pro database copilot. Use only the schema metadata provided in the user context. Explain your answer briefly. When proposing SQL, put each draft in one ```sql fenced block. Never claim that SQL was executed. Never perform or request a database mutation automatically; mutations must be clearly marked for human review. Prefer read-only SQL and include a bounded LIMIT when appropriate.";
const SQL_PREDICTION_INSTRUCTIONS: &str = "You are an inline SQL completion engine. Return only the SQL text that should be inserted at the cursor. Do not return Markdown, explanations, comments about the request, or code fences. Use only the supplied context. Preserve the user's dialect and do not invent schema objects.";

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentDraft {
    pub content: String,
    pub sql: Option<String>,
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SqlPredictionContext {
    pub sql_before_cursor: String,
    pub sql_after_cursor: String,
    pub current_statement: String,
    pub active_schema: String,
    pub dialect: String,
    pub referenced_tables: Vec<String>,
    pub table_aliases: std::collections::HashMap<String, String>,
    pub relevant_columns: Vec<String>,
    pub fk_neighbors: Vec<String>,
    pub cte_names: Vec<String>,
}

#[derive(Debug, Error)]
pub enum CodexProviderError {
    #[error("AI provider request failed: {0}")]
    Request(String),
    #[error("AI provider returned HTTP {status}: {body}")]
    Http { status: u16, body: String },
    #[error("AI provider response could not be decoded: {0}")]
    Decode(String),
    #[error("AI provider returned no text")]
    EmptyResponse,
}

#[derive(Clone)]
pub struct CodexProvider {
    client: Client,
    endpoint: String,
    model: String,
    api_key: String,
    provider_name: String,
}

impl std::fmt::Debug for CodexProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CodexProvider")
            .field("endpoint", &self.endpoint)
            .field("model", &self.model)
            .field("api_key", &"[redacted]")
            .finish()
    }
}

impl CodexProvider {
    pub fn from_env() -> Option<Self> {
        if let Some(api_key) = non_empty_env("GROQ_API_KEY") {
            let endpoint = std::env::var("DB_PRO_GROQ_ENDPOINT")
                .or_else(|_| std::env::var("DB_PRO_CODEX_ENDPOINT"))
                .unwrap_or_else(|_| DEFAULT_GROQ_ENDPOINT.to_owned());
            let model = std::env::var("DB_PRO_GROQ_MODEL")
                .or_else(|_| std::env::var("DB_PRO_CODEX_MODEL"))
                .unwrap_or_else(|_| DEFAULT_GROQ_MODEL.to_owned());
            return Self::configured(api_key, endpoint, model, "Groq");
        }

        let api_key = non_empty_env("OPENAI_API_KEY")?;
        let endpoint = std::env::var("DB_PRO_CODEX_ENDPOINT").unwrap_or_else(|_| DEFAULT_ENDPOINT.to_owned());
        let model = std::env::var("DB_PRO_CODEX_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_owned());
        Self::configured(api_key, endpoint, model, "OpenAI")
    }

    pub fn new(
        api_key: impl Into<String>,
        endpoint: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<Self, CodexProviderError> {
        Self::with_provider(api_key, endpoint, model, "Codex")
    }

    fn with_provider(
        api_key: impl Into<String>,
        endpoint: impl Into<String>,
        model: impl Into<String>,
        provider_name: impl Into<String>,
    ) -> Result<Self, CodexProviderError> {
        let endpoint = endpoint.into();
        if !endpoint.starts_with("https://") {
            return Err(CodexProviderError::Request("AI endpoint must use HTTPS".to_owned()));
        }
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|error| CodexProviderError::Request(error.to_string()))?;
        Ok(Self {
            client,
            endpoint,
            model: model.into(),
            api_key: api_key.into(),
            provider_name: provider_name.into(),
        })
    }

    pub fn provider_name(&self) -> &str {
        &self.provider_name
    }

    fn configured(api_key: String, endpoint: String, model: String, provider_name: &str) -> Option<Self> {
        match Self::with_provider(api_key, endpoint, model, provider_name) {
            Ok(provider) => Some(provider),
            Err(error) => {
                tracing::warn!(provider = provider_name, error = %error, "AI provider configuration rejected");
                None
            }
        }
    }

    /// Public-to-crate constructor used by the `ConfigureAgent` worker command.
    pub(crate) fn with_provider_pub(
        api_key: impl Into<String>,
        endpoint: impl Into<String>,
        model: impl Into<String>,
        provider_name: impl Into<String>,
    ) -> Result<Self, CodexProviderError> {
        Self::with_provider(api_key, endpoint, model, provider_name)
    }

    pub async fn respond(&self, prompt: &str, context: &AgentContext) -> Result<AgentDraft, CodexProviderError> {
        let text = self
            .request_text(CODEX_INSTRUCTIONS, build_input(prompt, context))
            .await?;
        Ok(parse_draft(&text))
    }

    pub async fn predict_sql(&self, context: &SqlPredictionContext) -> Result<String, CodexProviderError> {
        let text = self
            .request_text(SQL_PREDICTION_INSTRUCTIONS, build_prediction_input(context))
            .await?;
        let prediction = normalize_prediction(&text);
        if prediction.is_empty() {
            Err(CodexProviderError::EmptyResponse)
        } else {
            Ok(prediction)
        }
    }

    async fn request_text(&self, instructions: &str, input: String) -> Result<String, CodexProviderError> {
        let response = self
            .client
            .post(&self.endpoint)
            .bearer_auth(&self.api_key)
            .json(&json!({
                "model": self.model,
                "store": false,
                "instructions": instructions,
                "input": input,
            }))
            .send()
            .await
            .map_err(|error| CodexProviderError::Request(error.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| CodexProviderError::Request(error.to_string()))?;
        if !status.is_success() {
            return Err(CodexProviderError::Http {
                status: status.as_u16(),
                body: truncate(&body),
            });
        }

        let payload = serde_json::from_str::<ResponsesPayload>(&body)
            .map_err(|error| CodexProviderError::Decode(error.to_string()))?;
        payload.text().ok_or(CodexProviderError::EmptyResponse)
    }
}

fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.trim().is_empty())
}

fn build_input(prompt: &str, context: &AgentContext) -> String {
    let connection = context.connection_name.as_deref().unwrap_or("active connection");
    let tables = if context.tables.is_empty() {
        "(no tables loaded)".to_owned()
    } else {
        context.tables.iter().take(200).cloned().collect::<Vec<_>>().join(", ")
    };
    let columns = if context.columns.is_empty() {
        "(no columns loaded)".to_owned()
    } else {
        context.columns.iter().take(400).cloned().collect::<Vec<_>>().join(", ")
    };
    format!(
        "Database context:\n- connection: {connection}\n- driver: {}\n- schema: {}\n- selected table: {}\n- selected columns: {}\n- tables: {tables}\n- columns: {columns}\n- current SQL: {}\n- result summary: {}\n- explain plan: {}\n- last error: {}\n\nUser request:\n{}",
        context.driver,
        context.schema.as_deref().unwrap_or("(default)"),
        context.selected_table.as_deref().unwrap_or("(none)"),
        if context.selected_columns.is_empty() {
            "(none)".to_owned()
        } else {
            context.selected_columns.join(", ")
        },
        if context.current_sql.trim().is_empty() {
            "(none)".to_owned()
        } else {
            context.current_sql.clone()
        },
        context.result_summary.as_deref().unwrap_or("(none)"),
        context.explain_plan.as_deref().unwrap_or("(none)"),
        context.last_error.as_deref().unwrap_or("(none)"),
        prompt.trim()
    )
}

fn build_prediction_input(context: &SqlPredictionContext) -> String {
    let aliases = if context.table_aliases.is_empty() {
        "(none)".to_owned()
    } else {
        let mut alias_pairs: Vec<_> = context.table_aliases.iter().collect();
        alias_pairs.sort_unstable_by(|left, right| left.0.cmp(right.0).then_with(|| left.1.cmp(right.1)));
        alias_pairs
            .into_iter()
            .map(|(alias, table)| format!("{alias} -> {table}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!(
        "Dialect: {}\nActive schema: {}\nSQL before cursor:\n{}\nSQL after cursor:\n{}\nCurrent statement:\n{}\nReferenced tables: {}\nAliases: {}\nRelevant columns: {}\nFK neighbors: {}\nCTEs: {}",
        context.dialect,
        context.active_schema,
        context.sql_before_cursor,
        context.sql_after_cursor,
        context.current_statement,
        context.referenced_tables.join(", "),
        aliases,
        context.relevant_columns.join(", "),
        context.fk_neighbors.join(", "),
        context.cte_names.join(", "),
    )
}

#[derive(Debug, Deserialize)]
struct ResponsesPayload {
    output_text: Option<String>,
    output: Option<Vec<OutputItem>>,
}

impl ResponsesPayload {
    fn text(self) -> Option<String> {
        self.output_text.filter(|text| !text.trim().is_empty()).or_else(|| {
            self.output?
                .into_iter()
                .flat_map(|item| item.content.unwrap_or_default())
                .find_map(|part| {
                    (part.kind.as_deref() == Some("output_text"))
                        .then_some(part.text)
                        .flatten()
                        .filter(|text| !text.trim().is_empty())
                })
        })
    }
}

#[derive(Debug, Deserialize)]
struct OutputItem {
    content: Option<Vec<OutputContent>>,
}

#[derive(Debug, Deserialize)]
struct OutputContent {
    #[serde(rename = "type")]
    kind: Option<String>,
    text: Option<String>,
}

fn parse_draft(text: &str) -> AgentDraft {
    let sql = extract_sql_block(text);
    let content = sql
        .as_ref()
        .map(|sql| {
            text.replace(sql_fenced_block(text, sql).as_deref().unwrap_or_default(), "")
                .trim()
                .to_owned()
        })
        .filter(|content| !content.is_empty())
        .unwrap_or_else(|| text.trim().to_owned());
    let requires_confirmation = sql.as_deref().is_some_and(draft_requires_confirmation);
    AgentDraft {
        content,
        sql,
        requires_confirmation,
    }
}

fn extract_sql_block(text: &str) -> Option<String> {
    let start = text.find("```")?;
    let after_ticks = &text[start + 3..];
    let language_end = after_ticks.find('\n')?;
    let language = after_ticks[..language_end].trim();
    if !language.is_empty() && !language.eq_ignore_ascii_case("sql") {
        return None;
    }
    let body = &after_ticks[language_end + 1..];
    let end = body.find("```")?;
    let sql = body[..end].trim();
    (!sql.is_empty()).then(|| sql.to_owned())
}

fn sql_fenced_block(text: &str, sql: &str) -> Option<String> {
    let start = text.find("```")?;
    let after_ticks = &text[start + 3..];
    let language_end = after_ticks.find('\n')?;
    let body = &after_ticks[language_end + 1..];
    let end = body.find("```")?;
    (body[..end].trim() == sql).then(|| text[start..start + 3 + language_end + 1 + end + 3].to_owned())
}

fn draft_requires_confirmation(sql: &str) -> bool {
    if sql.trim_end().trim_end_matches(';').contains(';') {
        return true;
    }
    !matches!(classify_statement_safety(sql), Some(StatementSafety::Read))
}

fn truncate(value: &str) -> String {
    value.chars().take(500).collect()
}

fn normalize_prediction(text: &str) -> String {
    let trimmed = text.trim();
    let body = if let Some(fence_start) = trimmed.find("```") {
        let fenced = &trimmed[fence_start + 3..];
        let body_start = fenced.find('\n').map(|offset| offset + 1).unwrap_or(0);
        let body = &fenced[body_start..];
        body.split_once("```").map_or(body, |(body, _)| body)
    } else {
        trimmed
    };

    let body = body.trim_matches(['\r', '\n']);
    let mut lines = body.lines();
    let Some(first_line) = lines.next() else {
        return String::new();
    };
    if is_explanation_line(first_line) {
        lines
            .collect::<Vec<_>>()
            .join("\n")
            .trim_matches(['\r', '\n'])
            .to_owned()
    } else {
        body.to_owned()
    }
}

fn is_explanation_line(line: &str) -> bool {
    let lower = line.trim().to_ascii_lowercase();
    [
        "here is",
        "here's",
        "suggestion:",
        "suggested query:",
        "sql:",
        "answer:",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_rejects_non_https_endpoint() {
        let error = CodexProvider::new("secret", "http://localhost/responses", "model")
            .expect_err("HTTP endpoint must be rejected");
        assert!(error.to_string().contains("HTTPS"));
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
}
