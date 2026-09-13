use crate::editor::completion::{CompletionItem, CompletionItemKind};
use crate::runtime::UiSchemaSummary;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CompletionContext<'a> {
    pub text_before_cursor: &'a str,
    pub text_after_cursor: &'a str,
    pub cursor_offset: usize,
    pub active_schema: &'a str,
    pub schema_summary: &'a UiSchemaSummary,
    pub is_sqlite: bool,
}

pub struct SchemaCompletionProvider;

impl SchemaCompletionProvider {
    pub fn provide(ctx: &CompletionContext<'_>) -> (String, Vec<CompletionItem>) {
        let text_before = ctx.text_before_cursor;
        let prefix = extract_word_prefix(text_before);
        let prefix_lower = prefix.to_lowercase();
        let replacement_start = ctx.cursor_offset.saturating_sub(prefix.len());
        let replacement_range = (replacement_start, ctx.cursor_offset);

        // Check if there is a dot right before the prefix: e.g. "users." or "u." or "public."
        let before_prefix = &text_before[..text_before.len() - prefix.len()];
        let qualifier = extract_qualifier(before_prefix);

        let mut items = Vec::new();

        if let Some(qualifier) = qualifier {
            let qual_lower = qualifier.to_lowercase();

            // 1. Check if qualifier is a schema name (e.g. "public.")
            for table in &ctx.schema_summary.table_details {
                if table.schema.eq_ignore_ascii_case(&qual_lower) && table.name.to_lowercase().contains(&prefix_lower) {
                    items.push(CompletionItem {
                        label: table.name.clone(),
                        insert_text: table.name.clone(),
                        kind: CompletionItemKind::Table,
                        detail: Some(format!("Table · {}.{}", table.schema, table.name)),
                        documentation: Some(format!("{} columns in {}", table.columns.len(), table.name)),
                        replacement_range,
                    });
                }
            }

            // 2. Check if qualifier is a table name or an alias in the document
            let mut full_doc = String::with_capacity(ctx.text_before_cursor.len() + ctx.text_after_cursor.len() + 1);
            full_doc.push_str(ctx.text_before_cursor);
            full_doc.push(' ');
            full_doc.push_str(ctx.text_after_cursor);
            let aliases = extract_table_aliases(&full_doc);
            let resolved_table = aliases.get(qual_lower.as_str()).cloned().unwrap_or(qual_lower);

            for table in &ctx.schema_summary.table_details {
                if table.name.eq_ignore_ascii_case(&resolved_table) {
                    for col in &table.columns {
                        if col.name.to_lowercase().contains(&prefix_lower) {
                            items.push(CompletionItem {
                                label: col.name.clone(),
                                insert_text: col.name.clone(),
                                kind: CompletionItemKind::Column,
                                detail: Some(format!("Column · {}", col.data_type)),
                                documentation: Some(format!(
                                    "Column {} of {}.{}{}",
                                    col.name,
                                    table.schema,
                                    table.name,
                                    if col.is_primary_key { " (PRIMARY KEY)" } else { "" }
                                )),
                                replacement_range,
                            });
                        }
                    }
                }
            }

            return (prefix.to_owned(), items);
        }

        // Contextual analysis: FROM | or JOIN | suggests tables & views
        let trimmed_before = before_prefix.trim_end();
        let upper_before = trimmed_before.to_ascii_uppercase();
        let is_from_or_join = upper_before.ends_with("FROM")
            || upper_before.ends_with("JOIN")
            || upper_before.ends_with("INTO")
            || upper_before.ends_with("TABLE");

        if is_from_or_join {
            // Suggest Tables
            for table in &ctx.schema_summary.table_details {
                if table.name.to_lowercase().contains(&prefix_lower) {
                    items.push(CompletionItem {
                        label: table.name.clone(),
                        insert_text: table.name.clone(),
                        kind: CompletionItemKind::Table,
                        detail: Some(format!("Table · {}.{}", table.schema, table.name)),
                        documentation: Some(format!("Table with {} columns", table.columns.len())),
                        replacement_range,
                    });
                }
            }
            if items.is_empty() {
                for table_name in &ctx.schema_summary.tables {
                    if table_name.to_lowercase().contains(&prefix_lower) {
                        items.push(CompletionItem {
                            label: table_name.clone(),
                            insert_text: table_name.clone(),
                            kind: CompletionItemKind::Table,
                            detail: Some("Table".to_owned()),
                            documentation: None,
                            replacement_range,
                        });
                    }
                }
            }
            // Suggest Views
            for view in &ctx.schema_summary.views {
                if view.name.to_lowercase().contains(&prefix_lower) {
                    items.push(CompletionItem {
                        label: view.name.clone(),
                        insert_text: view.name.clone(),
                        kind: CompletionItemKind::View,
                        detail: Some(format!("View · {}.{}", view.schema, view.name)),
                        documentation: Some("Database view".to_owned()),
                        replacement_range,
                    });
                }
            }
            return (prefix.to_owned(), items);
        }

        // Default: suggest Columns from active schema tables, Tables, Views, Functions, Keywords
        // 1. Columns
        for table in &ctx.schema_summary.table_details {
            for col in &table.columns {
                if col.name.to_lowercase().contains(&prefix_lower) {
                    items.push(CompletionItem {
                        label: col.name.clone(),
                        insert_text: col.name.clone(),
                        kind: CompletionItemKind::Column,
                        detail: Some(format!("{}.{} · {}", table.name, col.name, col.data_type)),
                        documentation: None,
                        replacement_range,
                    });
                }
            }
        }

        // 2. Tables & Views
        for table in &ctx.schema_summary.table_details {
            if table.name.to_lowercase().contains(&prefix_lower) {
                items.push(CompletionItem {
                    label: table.name.clone(),
                    insert_text: table.name.clone(),
                    kind: CompletionItemKind::Table,
                    detail: Some(format!("Table · {}.{}", table.schema, table.name)),
                    documentation: None,
                    replacement_range,
                });
            }
        }
        if ctx.schema_summary.table_details.is_empty() {
            for table_name in &ctx.schema_summary.tables {
                if table_name.to_lowercase().contains(&prefix_lower) {
                    items.push(CompletionItem {
                        label: table_name.clone(),
                        insert_text: table_name.clone(),
                        kind: CompletionItemKind::Table,
                        detail: Some("Table".to_owned()),
                        documentation: None,
                        replacement_range,
                    });
                }
            }
        }

        // 3. Keywords
        const KEYWORDS: &[&str] = &[
            "SELECT",
            "FROM",
            "WHERE",
            "JOIN",
            "LEFT JOIN",
            "INNER JOIN",
            "GROUP BY",
            "ORDER BY",
            "LIMIT",
            "HAVING",
            "DISTINCT",
            "AS",
            "AND",
            "OR",
            "NOT",
            "IN",
            "IS NULL",
            "IS NOT NULL",
            "INSERT INTO",
            "VALUES",
            "UPDATE",
            "SET",
            "DELETE FROM",
            "COUNT",
            "SUM",
            "AVG",
            "COALESCE",
        ];
        for kw in KEYWORDS {
            if kw.to_lowercase().contains(&prefix_lower) {
                items.push(CompletionItem {
                    label: (*kw).to_owned(),
                    insert_text: (*kw).to_owned(),
                    kind: CompletionItemKind::Keyword,
                    detail: Some("SQL Keyword".to_owned()),
                    documentation: None,
                    replacement_range,
                });
            }
        }

        (prefix.to_owned(), items)
    }
}

fn extract_word_prefix(text: &str) -> &str {
    let mut end = text.len();
    while end > 0 && text[end - 1..end].chars().all(|c| c.is_alphanumeric() || c == '_') {
        end -= 1;
    }
    &text[end..]
}

fn extract_qualifier(text: &str) -> Option<&str> {
    let trimmed = text.trim_end();
    if let Some(without_dot) = trimmed.strip_suffix('.') {
        let start = without_dot
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|idx| idx + 1)
            .unwrap_or(0);
        let qual = &without_dot[start..];
        if !qual.is_empty() {
            return Some(qual);
        }
    }
    None
}

fn extract_table_aliases(text: &str) -> HashMap<String, String> {
    let mut aliases = HashMap::new();
    let words: Vec<&str> = text.split_whitespace().collect();
    for window in words.windows(3) {
        if window[0].eq_ignore_ascii_case("FROM") || window[0].eq_ignore_ascii_case("JOIN") {
            let table = window[1].trim_matches(|c| c == '"' || c == '`' || c == '[' || c == ']');
            let alias = window[2].trim_matches(|c| c == '"' || c == '`' || c == '[' || c == ']');
            if !alias.eq_ignore_ascii_case("WHERE")
                && !alias.eq_ignore_ascii_case("ON")
                && !alias.eq_ignore_ascii_case("JOIN")
            {
                aliases.insert(alias.to_lowercase(), table.to_lowercase());
            }
        }
    }
    aliases
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{UiSchemaColumn, UiTableSummary};

    #[test]
    fn test_extract_table_aliases() {
        let sql = "SELECT u.id, u.name FROM users u JOIN orders o ON u.id = o.user_id";
        let aliases = extract_table_aliases(sql);
        assert_eq!(aliases.get("u").map(|s| s.as_str()), Some("users"));
        assert_eq!(aliases.get("o").map(|s| s.as_str()), Some("orders"));
    }

    #[test]
    fn test_completion_resolves_alias_column() {
        let summary = UiSchemaSummary {
            table_details: vec![UiTableSummary {
                schema: "public".to_owned(),
                name: "users".to_owned(),
                row_count: Some(10),
                columns: vec![
                    UiSchemaColumn {
                        name: "id".to_owned(),
                        data_type: "integer".to_owned(),
                        nullable: false,
                        is_primary_key: true,
                    },
                    UiSchemaColumn {
                        name: "email".to_owned(),
                        data_type: "varchar".to_owned(),
                        nullable: false,
                        is_primary_key: false,
                    },
                ],
                foreign_keys: vec![],
            }],
            ..Default::default()
        };

        let ctx = CompletionContext {
            text_before_cursor: "SELECT u.em",
            text_after_cursor: " FROM users u WHERE ",
            cursor_offset: 11,
            active_schema: "public",
            schema_summary: &summary,
            is_sqlite: false,
        };

        let (prefix, items) = SchemaCompletionProvider::provide(&ctx);
        assert_eq!(prefix, "em");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "email");
        assert_eq!(items[0].kind, CompletionItemKind::Column);
    }
}
