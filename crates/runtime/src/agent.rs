use std::time::Duration;

use db_pro_core::domain::safety::{classify_statement_safety, StatementSafety};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;

const DEFAULT_ENDPOINT: &str = "https://api.openai.com/v1/responses";
const DEFAULT_MODEL: &str = "gpt-5.6";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const CODEX_INSTRUCTIONS: &str = "You are the DB Pro database copilot. Use only the schema metadata provided in the user context. Explain your answer briefly. When proposing SQL, put each draft in one ```sql fenced block. Never claim that SQL was executed. Never perform or request a database mutation automatically; mutations must be clearly marked for human review. Prefer read-only SQL and include a bounded LIMIT when appropriate.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentContext {
    pub connection_name: Option<String>,
    pub driver: String,
    pub tables: Vec<String>,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentDraft {
    pub content: String,
    pub sql: Option<String>,
    pub requires_confirmation: bool,
}

#[derive(Debug, Error)]
pub enum CodexProviderError {
    #[error("Codex provider request failed: {0}")]
    Request(String),
    #[error("Codex provider returned HTTP {status}: {body}")]
    Http { status: u16, body: String },
    #[error("Codex provider response could not be decoded: {0}")]
    Decode(String),
    #[error("Codex provider returned no text")]
    EmptyResponse,
}

#[derive(Clone)]
pub struct CodexProvider {
    client: Client,
    endpoint: String,
    model: String,
    api_key: String,
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
        let api_key = std::env::var("OPENAI_API_KEY")
            .ok()
            .filter(|value| !value.trim().is_empty())?;
        let endpoint = std::env::var("DB_PRO_CODEX_ENDPOINT").unwrap_or_else(|_| DEFAULT_ENDPOINT.to_owned());
        let model = std::env::var("DB_PRO_CODEX_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_owned());
        Self::new(api_key, endpoint, model).ok()
    }

    pub fn new(
        api_key: impl Into<String>,
        endpoint: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<Self, CodexProviderError> {
        let endpoint = endpoint.into();
        if !endpoint.starts_with("https://") {
            return Err(CodexProviderError::Request("Codex endpoint must use HTTPS".to_owned()));
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
        })
    }

    pub async fn respond(&self, prompt: &str, context: &AgentContext) -> Result<AgentDraft, CodexProviderError> {
        let response = self
            .client
            .post(&self.endpoint)
            .bearer_auth(&self.api_key)
            .json(&json!({
                "model": self.model,
                "store": false,
                "instructions": CODEX_INSTRUCTIONS,
                "input": build_input(prompt, context),
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
        let text = payload.text().ok_or(CodexProviderError::EmptyResponse)?;
        Ok(parse_draft(&text))
    }
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
        "Database context:\n- connection: {connection}\n- driver: {}\n- tables: {tables}\n- columns: {columns}\n\nUser request:\n{}",
        context.driver,
        prompt.trim()
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
        };
        let input = build_input("show customers", &context);
        assert!(input.contains("customers"));
        assert!(!input.contains("password"));
    }
}
