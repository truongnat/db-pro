use std::time::Duration;

use async_trait::async_trait;
use db_pro_core::domain::agent::{AgentObjectRef, AgentTool, AgentToolCall, AgentToolInput, AgentToolOutput};
use db_pro_core::domain::agent_context::AgentContext as StructuredAgentContext;
use db_pro_core::domain::agent_workflow::AgentToolError;
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
const AGENT_TOOL_INSTRUCTIONS: &str = "You are the DB Pro database agent. Use only supplied schema and query context. Inspect before guessing. Use typed tools for schema access, SQL patches, execution, and result inspection. Never claim a tool ran unless its typed result is returned. Mutations and destructive SQL require user confirmation. Keep text concise.";
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProviderRequest {
    pub prompt: String,
    pub context: StructuredAgentContext,
    pub messages: Vec<AgentProviderMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentProviderMessage {
    AssistantText(String),
    ToolCall(AgentToolCall),
    ToolResult {
        call_id: String,
        tool: AgentTool,
        output: Result<AgentToolOutput, AgentToolError>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentProviderEvent {
    TextDelta { delta: String },
    ToolCall(AgentToolCall),
    Completed,
}

#[async_trait]
pub trait AgentProvider: Send + Sync {
    async fn complete(&self, request: AgentProviderRequest) -> Result<Vec<AgentProviderEvent>, CodexProviderError>;
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

    pub async fn complete_with_tools(
        &self,
        request: AgentProviderRequest,
    ) -> Result<Vec<AgentProviderEvent>, CodexProviderError> {
        let payload = self
            .request_json(json!({
                "model": self.model,
                "store": false,
                "instructions": AGENT_TOOL_INSTRUCTIONS,
                "input": build_agent_provider_input(&request),
                "tools": tool_definitions(),
            }))
            .await?;
        parse_provider_events(payload)
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
        let payload = self
            .request_json(json!({
                "model": self.model,
                "store": false,
                "instructions": instructions,
                "input": input,
            }))
            .await?;
        payload.text().ok_or(CodexProviderError::EmptyResponse)
    }

    async fn request_json(&self, payload: serde_json::Value) -> Result<ResponsesPayload, CodexProviderError> {
        let response = self
            .client
            .post(&self.endpoint)
            .bearer_auth(&self.api_key)
            .json(&payload)
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

        serde_json::from_str::<ResponsesPayload>(&body).map_err(|error| CodexProviderError::Decode(error.to_string()))
    }
}

#[async_trait]
impl AgentProvider for CodexProvider {
    async fn complete(&self, request: AgentProviderRequest) -> Result<Vec<AgentProviderEvent>, CodexProviderError> {
        self.complete_with_tools(request).await
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
    fn text(&self) -> Option<String> {
        self.output_text
            .clone()
            .filter(|text| !text.trim().is_empty())
            .or_else(|| {
                self.output
                    .as_ref()?
                    .iter()
                    .flat_map(|item| item.content.clone().unwrap_or_default())
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
    #[serde(rename = "type")]
    item_type: Option<String>,
    id: Option<String>,
    call_id: Option<String>,
    name: Option<String>,
    arguments: Option<serde_json::Value>,
    content: Option<Vec<OutputContent>>,
}

#[derive(Debug, Clone, Deserialize)]
struct OutputContent {
    #[serde(rename = "type")]
    kind: Option<String>,
    text: Option<String>,
}

fn build_agent_provider_input(request: &AgentProviderRequest) -> serde_json::Value {
    let context = serde_json::to_string(&request.context).unwrap_or_else(|_| "{}".to_owned());
    let user_content = format!("Agent context:\n{context}\nUser request:\n{}", request.prompt);
    if request.messages.is_empty() {
        return serde_json::Value::String(user_content);
    }

    let mut input = vec![json!({"role": "user", "content": user_content})];
    for message in &request.messages {
        match message {
            AgentProviderMessage::AssistantText(text) => {
                input.push(json!({"role": "assistant", "content": text}));
            }
            AgentProviderMessage::ToolCall(call) => {
                input.push(json!({
                    "type": "function_call",
                    "call_id": call.call_id,
                    "name": tool_name(call.tool),
                    "arguments": serde_json::to_string(&call.input).unwrap_or_else(|_| "null".to_owned()),
                }));
            }
            AgentProviderMessage::ToolResult { call_id, output, .. } => {
                let output = match output {
                    Ok(value) => serde_json::to_string(value).unwrap_or_else(|_| "{}".to_owned()),
                    Err(error) => serde_json::to_string(error).unwrap_or_else(|_| format!("{error}")),
                };
                input.push(json!({
                    "type": "function_call_output",
                    "call_id": call_id,
                    "output": output,
                }));
            }
        }
    }
    serde_json::Value::Array(input)
}

fn parse_provider_events(payload: ResponsesPayload) -> Result<Vec<AgentProviderEvent>, CodexProviderError> {
    let mut events = Vec::new();
    let has_top_level_text = payload
        .output_text
        .as_deref()
        .is_some_and(|text| !text.trim().is_empty());
    if let Some(text) = payload.output_text.filter(|text| !text.trim().is_empty()) {
        events.push(AgentProviderEvent::TextDelta { delta: text });
    }
    for item in payload.output.unwrap_or_default() {
        if item.item_type.as_deref() == Some("function_call") {
            events.push(AgentProviderEvent::ToolCall(parse_tool_call(item)?));
        } else if !has_top_level_text {
            for content in item.content.unwrap_or_default() {
                if content.kind.as_deref() == Some("output_text") {
                    if let Some(text) = content.text.filter(|text| !text.trim().is_empty()) {
                        events.push(AgentProviderEvent::TextDelta { delta: text });
                    }
                }
            }
        }
    }
    if events.is_empty() {
        return Err(CodexProviderError::EmptyResponse);
    }
    events.push(AgentProviderEvent::Completed);
    Ok(events)
}

fn parse_tool_call(item: OutputItem) -> Result<AgentToolCall, CodexProviderError> {
    let call_id = item
        .call_id
        .or(item.id)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| CodexProviderError::Decode("tool call is missing call_id".to_owned()))?;
    let name = item
        .name
        .ok_or_else(|| CodexProviderError::Decode("tool call is missing name".to_owned()))?;
    let tool = parse_tool_name(&name)?;
    let arguments = match item.arguments.unwrap_or(serde_json::Value::Null) {
        serde_json::Value::String(arguments) => serde_json::from_str(&arguments)
            .map_err(|error| CodexProviderError::Decode(format!("tool arguments are not valid JSON: {error}")))?,
        arguments => arguments,
    };
    let input = parse_tool_input(tool, arguments)?;
    Ok(AgentToolCall { call_id, tool, input })
}

fn parse_tool_name(name: &str) -> Result<AgentTool, CodexProviderError> {
    match name.to_ascii_lowercase().replace('_', "").as_str() {
        "inspectschema" => Ok(AgentTool::InspectSchema),
        "inspecttable" => Ok(AgentTool::InspectTable),
        "inspectcolumns" => Ok(AgentTool::InspectColumns),
        "inspectforeignkeys" => Ok(AgentTool::InspectForeignKeys),
        "getcurrentquery" => Ok(AgentTool::GetCurrentQuery),
        "patchquery" => Ok(AgentTool::PatchQuery),
        "runquery" => Ok(AgentTool::RunQuery),
        "inspectqueryresult" => Ok(AgentTool::InspectQueryResult),
        "explainquery" => Ok(AgentTool::ExplainQuery),
        "suggestindexes" => Ok(AgentTool::SuggestIndexes),
        "monitoringread" => Ok(AgentTool::MonitoringRead),
        _ => Err(CodexProviderError::Decode(format!("unknown agent tool: {name}"))),
    }
}

fn parse_tool_input(tool: AgentTool, arguments: serde_json::Value) -> Result<AgentToolInput, CodexProviderError> {
    let object = arguments.as_object();
    match tool {
        AgentTool::InspectSchema => Ok(AgentToolInput::Schema {
            schema: object
                .and_then(|value| value.get("schema"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
        }),
        AgentTool::InspectTable | AgentTool::InspectColumns | AgentTool::InspectForeignKeys => {
            let table = object
                .and_then(|value| value.get("table"))
                .and_then(parse_object_ref)
                .ok_or_else(|| CodexProviderError::Decode("table tool requires table.name".to_owned()))?;
            Ok(AgentToolInput::Table { table })
        }
        AgentTool::GetCurrentQuery => Ok(AgentToolInput::None),
        AgentTool::PatchQuery => Ok(AgentToolInput::Patch {
            patch: parse_patch(object.ok_or_else(|| invalid_arguments(tool))?)?,
        }),
        AgentTool::RunQuery | AgentTool::ExplainQuery => Ok(AgentToolInput::Query {
            sql: required_string(object, "sql", tool)?,
        }),
        AgentTool::SuggestIndexes => {
            let table = object
                .and_then(|value| value.get("table"))
                .and_then(parse_object_ref)
                .ok_or_else(|| CodexProviderError::Decode("suggest_indexes requires table.name".to_owned()))?;
            Ok(AgentToolInput::Table { table })
        }
        AgentTool::MonitoringRead => Ok(AgentToolInput::None),
        AgentTool::InspectQueryResult => Ok(AgentToolInput::ResultSample {
            max_rows: object
                .and_then(|value| value.get("max_rows"))
                .and_then(serde_json::Value::as_u64)
                .map_or(20, |value| usize::try_from(value).ok().unwrap_or(20)),
            statement_index: object
                .and_then(|value| value.get("statement_index"))
                .and_then(serde_json::Value::as_u64)
                .map(|value| value as usize),
        }),
    }
}

fn parse_object_ref(value: &serde_json::Value) -> Option<AgentObjectRef> {
    Some(AgentObjectRef {
        schema: value
            .get("schema")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        name: value.get("name").and_then(serde_json::Value::as_str)?.to_owned(),
    })
}

fn parse_patch(
    object: &serde_json::Map<String, serde_json::Value>,
) -> Result<db_pro_core::domain::agent::AgentSqlPatch, CodexProviderError> {
    let range = object
        .get("range")
        .and_then(serde_json::Value::as_array)
        .filter(|range| range.len() == 2)
        .and_then(|range| Some((range[0].as_u64()? as usize, range[1].as_u64()? as usize)))
        .ok_or_else(|| CodexProviderError::Decode("patch requires range [start,end]".to_owned()))?;
    Ok(db_pro_core::domain::agent::AgentSqlPatch {
        document_id: required_string(Some(object), "document_id", AgentTool::PatchQuery)?,
        expected_version: object
            .get("expected_version")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| CodexProviderError::Decode("patch requires expected_version".to_owned()))?,
        range,
        replacement: required_string(Some(object), "replacement", AgentTool::PatchQuery)?,
    })
}

fn required_string(
    object: Option<&serde_json::Map<String, serde_json::Value>>,
    key: &str,
    tool: AgentTool,
) -> Result<String, CodexProviderError> {
    object
        .and_then(|value| value.get(key))
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| invalid_arguments(tool))
}

fn invalid_arguments(tool: AgentTool) -> CodexProviderError {
    CodexProviderError::Decode(format!("invalid arguments for {tool:?}"))
}

fn tool_definitions() -> Vec<serde_json::Value> {
    [
        AgentTool::InspectSchema,
        AgentTool::InspectTable,
        AgentTool::InspectColumns,
        AgentTool::InspectForeignKeys,
        AgentTool::GetCurrentQuery,
        AgentTool::PatchQuery,
        AgentTool::RunQuery,
        AgentTool::InspectQueryResult,
        AgentTool::ExplainQuery,
        AgentTool::SuggestIndexes,
        AgentTool::MonitoringRead,
    ]
    .into_iter()
    .map(|tool| {
        json!({
            "type": "function",
            "name": tool_name(tool),
            "description": tool_description(tool),
            "parameters": tool_parameters(tool),
        })
    })
    .collect()
}

fn tool_name(tool: AgentTool) -> &'static str {
    match tool {
        AgentTool::InspectSchema => "inspect_schema",
        AgentTool::InspectTable => "inspect_table",
        AgentTool::InspectColumns => "inspect_columns",
        AgentTool::InspectForeignKeys => "inspect_foreign_keys",
        AgentTool::GetCurrentQuery => "get_current_query",
        AgentTool::PatchQuery => "patch_query",
        AgentTool::RunQuery => "run_query",
        AgentTool::InspectQueryResult => "inspect_query_result",
        AgentTool::ExplainQuery => "explain_query",
        AgentTool::SuggestIndexes => "suggest_indexes",
        AgentTool::MonitoringRead => "monitoring_read",
    }
}

fn tool_description(tool: AgentTool) -> &'static str {
    match tool {
        AgentTool::InspectSchema => "Inspect bounded tables and views in the pinned schema.",
        AgentTool::InspectTable => "Inspect one table's columns and constraints.",
        AgentTool::InspectColumns => "Inspect one table's columns in ordinal order.",
        AgentTool::InspectForeignKeys => "Inspect foreign-key relations.",
        AgentTool::GetCurrentQuery => "Read the current query document snapshot.",
        AgentTool::PatchQuery => "Propose a range-based SQL editor patch for user review.",
        AgentTool::RunQuery => "Execute SQL through the database safety policy.",
        AgentTool::InspectQueryResult => "Inspect a bounded summary of the latest agent result.",
        AgentTool::ExplainQuery => "Run EXPLAIN without ANALYZE via the query runtime.",
        AgentTool::SuggestIndexes => "Suggest missing indexes from table metadata heuristics (not autonomous DDL).",
        AgentTool::MonitoringRead => "Read a bounded monitoring snapshot (sessions/locks) via MonitoringService.",
    }
}

fn tool_parameters(tool: AgentTool) -> serde_json::Value {
    match tool {
        AgentTool::InspectSchema => json!({"type":"object","properties":{"schema":{"type":"string"}}}),
        AgentTool::InspectTable
        | AgentTool::InspectColumns
        | AgentTool::InspectForeignKeys
        | AgentTool::SuggestIndexes => {
            json!({
                "type":"object","properties":{"table":{"type":"object","properties":{"schema":{"type":"string"},"name":{"type":"string"}},"required":["name"]}},"required":["table"]
            })
        }
        AgentTool::GetCurrentQuery | AgentTool::MonitoringRead => json!({"type":"object","properties":{}}),
        AgentTool::PatchQuery => json!({
            "type":"object","properties":{"document_id":{"type":"string"},"expected_version":{"type":"integer"},"range":{"type":"array","items":{"type":"integer"},"minItems":2,"maxItems":2},"replacement":{"type":"string"}},"required":["document_id","expected_version","range","replacement"]
        }),
        AgentTool::RunQuery | AgentTool::ExplainQuery => {
            json!({"type":"object","properties":{"sql":{"type":"string"}},"required":["sql"]})
        }
        AgentTool::InspectQueryResult => {
            json!({"type":"object","properties":{"max_rows":{"type":"integer"},"statement_index":{"type":"integer"}}})
        }
    }
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
    use db_pro_core::domain::agent_context::AgentContext as StructuredAgentContext;

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
}
