use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::agent::{
    AgentObjectRef, MAX_AGENT_CELL_CHARS, MAX_AGENT_COLUMNS_PER_TABLE, MAX_AGENT_CONTEXT_CHARS, MAX_AGENT_RELATIONS,
    MAX_AGENT_RESULT_COLUMNS, MAX_AGENT_SAMPLE_ROWS, MAX_AGENT_TABLES,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentColumnContext {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentForeignKeyContext {
    pub name: String,
    pub source: AgentObjectRef,
    pub source_columns: Vec<String>,
    pub target: AgentObjectRef,
    pub target_columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentTableContext {
    pub object: AgentObjectRef,
    pub columns: Vec<AgentColumnContext>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentResultColumn {
    pub name: String,
    pub data_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentResultSummary {
    pub columns: Vec<AgentResultColumn>,
    pub sample_rows: Vec<Vec<String>>,
    pub row_count: Option<u64>,
    pub affected_rows: Option<u64>,
    pub truncated: bool,
}

impl AgentResultSummary {
    pub fn bounded(
        columns: Vec<AgentResultColumn>,
        sample_rows: Vec<Vec<String>>,
        row_count: Option<u64>,
        affected_rows: Option<u64>,
    ) -> Self {
        let columns_were_truncated = columns.len() > MAX_AGENT_RESULT_COLUMNS;
        let bounded_columns = columns.into_iter().take(MAX_AGENT_RESULT_COLUMNS).collect::<Vec<_>>();
        let rows_were_truncated = sample_rows.len() > MAX_AGENT_SAMPLE_ROWS;
        let bounded_rows = sample_rows
            .into_iter()
            .take(MAX_AGENT_SAMPLE_ROWS)
            .map(|row| {
                row.into_iter()
                    .take(MAX_AGENT_RESULT_COLUMNS)
                    .map(|cell| truncate_chars(&cell, MAX_AGENT_CELL_CHARS))
                    .collect()
            })
            .collect();
        Self {
            columns: bounded_columns,
            sample_rows: bounded_rows,
            row_count,
            affected_rows,
            truncated: columns_were_truncated || rows_were_truncated,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentDiagnosticContext {
    pub message: String,
    pub range: Option<(usize, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentContext {
    pub document_id: String,
    pub document_version: u64,
    pub connection_id: Option<String>,
    pub schema: Option<String>,
    pub current_sql: String,
    pub selected_range: Option<(usize, usize)>,
    pub referenced_tables: Vec<AgentTableContext>,
    pub foreign_keys: Vec<AgentForeignKeyContext>,
    pub diagnostics: Vec<AgentDiagnosticContext>,
    pub result_summary: Option<AgentResultSummary>,
}

#[derive(Debug, Clone, Default)]
pub struct AgentSchemaCatalog {
    pub tables: Vec<AgentTableContext>,
    pub foreign_keys: Vec<AgentForeignKeyContext>,
}

#[derive(Debug, Clone)]
pub struct AgentContextRequest<'a> {
    pub document_id: &'a str,
    pub document_version: u64,
    pub connection_id: Option<&'a str>,
    pub schema: Option<&'a str>,
    pub current_sql: &'a str,
    pub selected_range: Option<(usize, usize)>,
    pub user_request: &'a str,
    pub diagnostics: &'a [AgentDiagnosticContext],
    pub result_summary: Option<AgentResultSummary>,
    pub catalog: &'a AgentSchemaCatalog,
}

#[derive(Debug, Clone, Copy)]
pub struct AgentContextBuilder {
    pub max_tables: usize,
    pub max_columns_per_table: usize,
    pub max_relations: usize,
    pub max_context_chars: usize,
}

impl Default for AgentContextBuilder {
    fn default() -> Self {
        Self {
            max_tables: MAX_AGENT_TABLES,
            max_columns_per_table: MAX_AGENT_COLUMNS_PER_TABLE,
            max_relations: MAX_AGENT_RELATIONS,
            max_context_chars: MAX_AGENT_CONTEXT_CHARS,
        }
    }
}

impl AgentContextBuilder {
    pub fn build(&self, request: &AgentContextRequest<'_>) -> AgentContext {
        let candidates = referenced_names(request.current_sql)
            .into_iter()
            .chain(referenced_names(request.user_request))
            .collect::<Vec<_>>();
        let mut selected = Vec::new();
        let mut selected_names = HashSet::new();
        for candidate in candidates {
            if let Some(table) = find_table(&request.catalog.tables, &candidate, request.schema) {
                let key = object_key(&table.object);
                if selected_names.insert(key) {
                    selected.push(bound_table(table, self.max_columns_per_table));
                }
            }
        }

        let mut relations = request
            .catalog
            .foreign_keys
            .iter()
            .filter(|relation| {
                selected_names.contains(&object_key(&relation.source))
                    || selected_names.contains(&object_key(&relation.target))
            })
            .cloned()
            .collect::<Vec<_>>();
        relations.sort_by_key(relation_key);
        relations.truncate(self.max_relations);

        for relation in &relations {
            for object in [&relation.source, &relation.target] {
                if selected.len() >= self.max_tables {
                    break;
                }
                let key = object_key(object);
                if selected_names.insert(key) {
                    if let Some(table) = find_table(&request.catalog.tables, object.name.as_str(), request.schema) {
                        selected.push(bound_table(table, self.max_columns_per_table));
                    }
                }
            }
        }
        selected.truncate(self.max_tables);
        selected.sort_by_key(|table| object_key(&table.object));

        let mut context = AgentContext {
            document_id: request.document_id.to_owned(),
            document_version: request.document_version,
            connection_id: request.connection_id.map(str::to_owned),
            schema: request.schema.map(str::to_owned),
            current_sql: request.current_sql.to_owned(),
            selected_range: request.selected_range,
            referenced_tables: selected,
            foreign_keys: relations,
            diagnostics: request.diagnostics.to_vec(),
            result_summary: request.result_summary.clone(),
        };
        trim_context(&mut context, self.max_context_chars);
        context
    }
}

fn find_table<'a>(tables: &'a [AgentTableContext], name: &str, schema: Option<&str>) -> Option<&'a AgentTableContext> {
    let normalized = name.trim_matches('"');
    let (qualified_schema, table_name) = normalized
        .split_once('.')
        .map_or((None, normalized), |(schema, table)| {
            (Some(schema.trim_matches('"')), table.trim_matches('"'))
        });
    tables.iter().find(|table| {
        table.object.name.eq_ignore_ascii_case(table_name)
            && qualified_schema.or(schema).is_none_or(|expected| {
                table
                    .object
                    .schema
                    .as_deref()
                    .is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
            })
    })
}

fn bound_table(table: &AgentTableContext, max_columns: usize) -> AgentTableContext {
    let mut bounded = table.clone();
    bounded.columns.truncate(max_columns);
    bounded
}

fn object_key(object: &AgentObjectRef) -> String {
    format!(
        "{}.{}",
        object.schema.as_deref().unwrap_or_default().to_ascii_lowercase(),
        object.name.to_ascii_lowercase()
    )
}

fn relation_key(relation: &AgentForeignKeyContext) -> String {
    format!(
        "{}:{}",
        object_key(&relation.source),
        relation.name.to_ascii_lowercase()
    )
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

fn trim_context(context: &mut AgentContext, max_chars: usize) {
    while serde_json::to_string(context).is_ok_and(|json| json.chars().count() > max_chars) {
        if context.result_summary.is_some() {
            context.result_summary = None;
        } else if !context.diagnostics.is_empty() {
            context.diagnostics.pop();
        } else if !context.foreign_keys.is_empty() {
            context.foreign_keys.pop();
        } else if !context.referenced_tables.is_empty() {
            context.referenced_tables.pop();
        } else {
            context.current_sql = truncate_chars(&context.current_sql, max_chars / 2);
            break;
        }
    }
}

fn referenced_names(sql: &str) -> Vec<String> {
    let tokens = lexical_tokens(sql);
    let mut names = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        let keyword = token.to_ascii_uppercase();
        let expects_table = matches!(keyword.as_str(), "FROM" | "JOIN" | "UPDATE" | "INTO");
        let delete_from = keyword == "DELETE"
            && tokens
                .get(index + 1)
                .is_some_and(|next| next.eq_ignore_ascii_case("FROM"));
        if !expects_table && !delete_from {
            continue;
        }
        let name_index = if delete_from { index + 2 } else { index + 1 };
        let Some(name) = tokens.get(name_index) else {
            continue;
        };
        if !is_sql_keyword(name) {
            names.push(name.clone());
        }
    }
    names
}

fn lexical_tokens(sql: &str) -> Vec<String> {
    sql.split(|character: char| !(character.is_ascii_alphanumeric() || matches!(character, '_' | '$' | '.' | '"')))
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
        .collect()
}

fn is_sql_keyword(value: &str) -> bool {
    matches!(
        value.to_ascii_uppercase().as_str(),
        "SELECT" | "WHERE" | "ON" | "SET" | "VALUES" | "RETURNING" | "GROUP" | "ORDER" | "BY" | "AS"
    )
}

#[cfg(test)]
mod tests {
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
}
