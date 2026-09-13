use crate::editor::completion::{CompletionItem, CompletionItemKind};
use crate::editor::prediction::AiSqlContext;
use crate::editor::syntax::CachedSqlTokens;
use crate::runtime::UiSchemaSummary;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CompletionContext<'a> {
    pub text_before_cursor: &'a str,
    pub text_after_cursor: &'a str,
    pub cursor_offset: usize,
    pub active_schema: &'a str,
    pub schema_summary: &'a UiSchemaSummary,
    pub cached_tokens: Option<&'a CachedSqlTokens>,
    pub is_sqlite: bool,
    pub is_manual_trigger: bool,
}

pub struct SchemaCompletionProvider;

impl SchemaCompletionProvider {
    pub fn provide(ctx: &CompletionContext<'_>) -> (String, Vec<CompletionItem>) {
        // 1. Suppress completion inside strings and comments unless manual trigger (Ctrl+Space)
        if !ctx.is_manual_trigger {
            if let Some(tokens) = ctx.cached_tokens {
                if tokens.is_in_string_or_comment(ctx.cursor_offset.saturating_sub(1)) {
                    return (String::new(), Vec::new());
                }
            }
        }

        let text_before = ctx.text_before_cursor;
        let prefix = extract_word_prefix(text_before);
        let prefix_lower = prefix.to_lowercase();
        let replacement_start = ctx.cursor_offset.saturating_sub(prefix.len());
        let replacement_range = (replacement_start, ctx.cursor_offset);

        // Check if there is a dot qualifier right before the prefix: e.g. "users." or "u." or "public."
        let before_prefix = &text_before[..text_before.len() - prefix.len()];
        let qualifier = extract_qualifier(before_prefix);

        let mut items = Vec::new();

        if let Some(qualifier) = qualifier {
            let qual_lower = qualifier.to_lowercase();

            // 1. Check if qualifier is a qualified schema.table (e.g. "public.users.")
            if let Some((schema_part, table_part)) = qual_lower.split_once('.') {
                for table in &ctx.schema_summary.table_details {
                    if table.schema.eq_ignore_ascii_case(schema_part) && table.name.eq_ignore_ascii_case(table_part) {
                        for col in &table.columns {
                            if col.name.to_lowercase().contains(&prefix_lower) {
                                items.push(CompletionItem {
                                    label: col.name.clone(),
                                    insert_text: col.name.clone(),
                                    kind: CompletionItemKind::Column,
                                    detail: Some(format!(
                                        "Column · {}{}",
                                        col.data_type,
                                        if col.is_primary_key { " [PK]" } else { "" }
                                    )),
                                    documentation: Some(format!(
                                        "Column {} of {}.{}{}",
                                        col.name,
                                        table.schema,
                                        table.name,
                                        if col.nullable { " (NULLABLE)" } else { " (NOT NULL)" }
                                    )),
                                    replacement_range,
                                    sort_score: 1,
                                });
                            }
                        }
                    }
                }
                rank_items(&mut items, &prefix_lower);
                return (prefix.to_owned(), items);
            }

            // 2. Check if qualifier is a schema name (e.g. "public.")
            for table in &ctx.schema_summary.table_details {
                if table.schema.eq_ignore_ascii_case(&qual_lower) && table.name.to_lowercase().contains(&prefix_lower) {
                    items.push(CompletionItem {
                        label: table.name.clone(),
                        insert_text: table.name.clone(),
                        kind: CompletionItemKind::Table,
                        detail: Some(format!("Table · {}.{}", table.schema, table.name)),
                        documentation: Some(format!("Table with {} columns", table.columns.len())),
                        replacement_range,
                        sort_score: 2,
                    });
                }
            }
            for view in &ctx.schema_summary.views {
                if view.schema.eq_ignore_ascii_case(&qual_lower) && view.name.to_lowercase().contains(&prefix_lower) {
                    items.push(CompletionItem {
                        label: view.name.clone(),
                        insert_text: view.name.clone(),
                        kind: CompletionItemKind::View,
                        detail: Some(format!("View · {}.{}", view.schema, view.name)),
                        documentation: Some("Database View".to_owned()),
                        replacement_range,
                        sort_score: 3,
                    });
                }
            }

            // 3. Check if qualifier is a table name or an alias / CTE in the document
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
                                detail: Some(format!(
                                    "Column · {}{}",
                                    col.data_type,
                                    if col.is_primary_key { " [PK]" } else { "" }
                                )),
                                documentation: Some(format!(
                                    "Column {} of {}.{}{}",
                                    col.name,
                                    table.schema,
                                    table.name,
                                    if col.nullable { " (NULLABLE)" } else { " (NOT NULL)" }
                                )),
                                replacement_range,
                                sort_score: 1,
                            });
                        }
                    }
                }
            }

            rank_items(&mut items, &prefix_lower);
            return (prefix.to_owned(), items);
        }

        // Contextual analysis: FROM | or JOIN | suggests tables & views first
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
                        sort_score: 2,
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
                            sort_score: 2,
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
                        documentation: Some("Database View".to_owned()),
                        replacement_range,
                        sort_score: 3,
                    });
                }
            }
            rank_items(&mut items, &prefix_lower);
            return (prefix.to_owned(), items);
        }

        // Default Context: suggest Columns from active schema tables, Tables, Views, Functions, Keywords
        // 1. Columns
        for table in &ctx.schema_summary.table_details {
            for col in &table.columns {
                if col.name.to_lowercase().contains(&prefix_lower) {
                    items.push(CompletionItem {
                        label: col.name.clone(),
                        insert_text: col.name.clone(),
                        kind: CompletionItemKind::Column,
                        detail: Some(format!(
                            "{}.{} · {}{}",
                            table.name,
                            col.name,
                            col.data_type,
                            if col.is_primary_key { " [PK]" } else { "" }
                        )),
                        documentation: None,
                        replacement_range,
                        sort_score: 1,
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
                    sort_score: 2,
                });
            }
        }
        for view in &ctx.schema_summary.views {
            if view.name.to_lowercase().contains(&prefix_lower) {
                items.push(CompletionItem {
                    label: view.name.clone(),
                    insert_text: view.name.clone(),
                    kind: CompletionItemKind::View,
                    detail: Some(format!("View · {}.{}", view.schema, view.name)),
                    documentation: None,
                    replacement_range,
                    sort_score: 3,
                });
            }
        }

        // 3. Functions
        for func in &ctx.schema_summary.functions {
            if func.name.to_lowercase().contains(&prefix_lower) {
                items.push(CompletionItem {
                    label: format!("{}()", func.name),
                    insert_text: format!("{}(", func.name),
                    kind: CompletionItemKind::Function,
                    detail: Some(format!("Function · {}.{}", func.schema, func.name)),
                    documentation: (!func.data_type.is_empty()).then(|| format!("Returns: {}", func.data_type)),
                    replacement_range,
                    sort_score: 4,
                });
            }
        }

        // 4. SQL Keywords & Snippets
        const KEYWORDS: &[&str] = &[
            "SELECT",
            "FROM",
            "WHERE",
            "JOIN",
            "LEFT JOIN",
            "INNER JOIN",
            "RIGHT JOIN",
            "FULL OUTER JOIN",
            "GROUP BY",
            "ORDER BY",
            "HAVING",
            "LIMIT",
            "OFFSET",
            "DISTINCT",
            "AS",
            "AND",
            "OR",
            "NOT",
            "IN",
            "IS NULL",
            "IS NOT NULL",
            "LIKE",
            "ILIKE",
            "BETWEEN",
            "EXISTS",
            "CASE",
            "WHEN",
            "THEN",
            "ELSE",
            "END",
            "INSERT INTO",
            "VALUES",
            "UPDATE",
            "SET",
            "DELETE FROM",
            "COUNT",
            "SUM",
            "AVG",
            "MIN",
            "MAX",
            "COALESCE",
            "NOW()",
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
                    sort_score: 5,
                });
            }
        }

        rank_items(&mut items, &prefix_lower);
        (prefix.to_owned(), items)
    }
}

pub fn build_ai_sql_context(
    doc_text: &str,
    cursor_offset: usize,
    active_schema: &str,
    schema_summary: &UiSchemaSummary,
) -> AiSqlContext {
    let (before_cursor, after_cursor) = if cursor_offset <= doc_text.len() {
        doc_text.split_at(cursor_offset)
    } else {
        (doc_text, "")
    };

    let aliases = extract_table_aliases(doc_text);
    let mut referenced_tables = Vec::new();
    for table_val in aliases.values() {
        if !referenced_tables.contains(table_val) {
            referenced_tables.push(table_val.clone());
        }
    }
    for table in &schema_summary.table_details {
        if doc_text.to_lowercase().contains(&table.name.to_lowercase())
            && !referenced_tables.iter().any(|t| t.eq_ignore_ascii_case(&table.name))
        {
            referenced_tables.push(table.name.clone());
        }
    }

    let mut relevant_columns = Vec::new();
    let mut fk_neighbors = Vec::new();

    for table_name in &referenced_tables {
        if let Some(table) = schema_summary
            .table_details
            .iter()
            .find(|t| t.name.eq_ignore_ascii_case(table_name))
        {
            for col in &table.columns {
                relevant_columns.push(format!("{}.{} ({})", table.name, col.name, col.data_type));
            }
            for fk in &table.foreign_keys {
                fk_neighbors.push(format!(
                    "{} ({}) -> {}.{} ({})",
                    table.name,
                    fk.from_columns.join(", "),
                    fk.to_schema,
                    fk.to_table,
                    fk.to_columns.join(", ")
                ));
            }
        }
    }

    AiSqlContext {
        sql_before_cursor: before_cursor.to_owned(),
        sql_after_cursor: after_cursor.to_owned(),
        current_statement: String::new(),
        active_schema: active_schema.to_owned(),
        referenced_tables,
        relevant_columns,
        fk_neighbors,
    }
}

fn rank_items(items: &mut [CompletionItem], prefix: &str) {
    items.sort_by(|a, b| {
        let a_exact = a.label.to_lowercase().starts_with(prefix);
        let b_exact = b.label.to_lowercase().starts_with(prefix);
        match (a_exact, b_exact) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                let kind_score = |k: &CompletionItemKind| match k {
                    CompletionItemKind::Column => 1,
                    CompletionItemKind::Table => 2,
                    CompletionItemKind::View => 3,
                    CompletionItemKind::Function => 4,
                    CompletionItemKind::Keyword => 5,
                    CompletionItemKind::Schema => 6,
                    CompletionItemKind::Snippet => 7,
                };
                let score_a = kind_score(&a.kind);
                let score_b = kind_score(&b.kind);
                score_a.cmp(&score_b).then_with(|| a.label.cmp(&b.label))
            }
        }
    });
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
            .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
            .map(|idx| idx + 1)
            .unwrap_or(0);
        let qual = &without_dot[start..];
        if !qual.is_empty() {
            return Some(qual);
        }
    }
    None
}

pub fn extract_table_aliases(text: &str) -> HashMap<String, String> {
    let mut aliases = HashMap::new();
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut i = 0;
    while i < words.len() {
        let w = words[i];
        if w.eq_ignore_ascii_case("WITH") && i + 2 < words.len() {
            let cte_name = words[i + 1].trim_matches(|c| {
                c == '"' || c == '`' || c == '[' || c == ']' || c == '(' || c == ')' || c == ';' || c == ','
            });
            if words[i + 2].eq_ignore_ascii_case("AS") {
                // Look inside CTE body for source table
                let mut j = i + 3;
                let mut inner_table = None;
                while j < words.len() && j < i + 20 {
                    if words[j].eq_ignore_ascii_case("FROM") && j + 1 < words.len() {
                        let t = words[j + 1].trim_matches(|c| {
                            c == '"' || c == '`' || c == '[' || c == ']' || c == '(' || c == ')' || c == ';' || c == ','
                        });
                        if !t.is_empty() {
                            inner_table = Some(t);
                            break;
                        }
                    }
                    j += 1;
                }
                if let Some(target) = inner_table {
                    let target_name = if let Some(dot_pos) = target.rfind('.') {
                        &target[dot_pos + 1..]
                    } else {
                        target
                    };
                    aliases.insert(cte_name.to_lowercase(), target_name.to_lowercase());
                } else {
                    aliases.insert(cte_name.to_lowercase(), cte_name.to_lowercase());
                }
            }
        } else if (w.eq_ignore_ascii_case("FROM")
            || w.eq_ignore_ascii_case("JOIN")
            || w.eq_ignore_ascii_case("INTO")
            || w.eq_ignore_ascii_case("UPDATE"))
            && i + 1 < words.len()
        {
            let raw_table = words[i + 1].trim_matches(|c| {
                c == '"' || c == '`' || c == '[' || c == ']' || c == '(' || c == ')' || c == ';' || c == ','
            });
            if !raw_table.is_empty() && !raw_table.starts_with('(') {
                let table_name = if let Some(dot_pos) = raw_table.rfind('.') {
                    &raw_table[dot_pos + 1..]
                } else {
                    raw_table
                };

                if i + 2 < words.len() {
                    if words[i + 2].eq_ignore_ascii_case("AS") {
                        if i + 3 < words.len() {
                            let alias = words[i + 3].trim_matches(|c| {
                                c == '"'
                                    || c == '`'
                                    || c == '['
                                    || c == ']'
                                    || c == '('
                                    || c == ')'
                                    || c == ';'
                                    || c == ','
                            });
                            if is_valid_alias_token(alias) {
                                aliases.insert(alias.to_lowercase(), table_name.to_lowercase());
                                i += 3;
                            }
                        }
                    } else {
                        let alias = words[i + 2].trim_matches(|c| {
                            c == '"' || c == '`' || c == '[' || c == ']' || c == '(' || c == ')' || c == ';' || c == ','
                        });
                        if is_valid_alias_token(alias) {
                            aliases.insert(alias.to_lowercase(), table_name.to_lowercase());
                            i += 2;
                        }
                    }
                }
            }
        }
        i += 1;
    }
    aliases
}

fn is_valid_alias_token(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    const NON_ALIAS_KEYWORDS: &[&str] = &[
        "WHERE", "ON", "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "FULL", "CROSS", "NATURAL", "GROUP", "ORDER",
        "HAVING", "LIMIT", "OFFSET", "UNION", "SET", "VALUES", "SELECT", "FROM", "AND", "OR", "USING", "AS",
    ];
    !NON_ALIAS_KEYWORDS.iter().any(|kw| kw.eq_ignore_ascii_case(token))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{UiSchemaColumn, UiTableSummary};

    #[test]
    fn test_extract_table_aliases() {
        let sql = "SELECT u.id, u.name FROM users AS u JOIN orders o ON u.id = o.user_id";
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
            cached_tokens: None,
            is_sqlite: false,
            is_manual_trigger: false,
        };

        let (prefix, items) = SchemaCompletionProvider::provide(&ctx);
        assert_eq!(prefix, "em");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "email");
        assert_eq!(items[0].kind, CompletionItemKind::Column);
        assert_eq!(items[0].replacement_range, (9, 11)); // "em" replaced
    }

    #[test]
    fn test_completion_suppression_inside_comment() {
        let summary = UiSchemaSummary::default();
        let mut tokens = CachedSqlTokens::new();
        let buf = crate::editor::buffer::TextBuffer::from_string("-- SELECT * FROM users\nSELECT 1;");
        tokens.get_or_recompute(&buf, crate::editor::syntax::SqlDialect::Postgres);

        let ctx = CompletionContext {
            text_before_cursor: "-- SELECT * FROM us",
            text_after_cursor: "\nSELECT 1;",
            cursor_offset: 19,
            active_schema: "public",
            schema_summary: &summary,
            cached_tokens: Some(&tokens),
            is_sqlite: false,
            is_manual_trigger: false,
        };

        let (prefix, items) = SchemaCompletionProvider::provide(&ctx);
        assert!(items.is_empty(), "Automatic completion must be suppressed in comments");
        assert_eq!(prefix, "");
    }

    #[test]
    fn test_from_clause_suggests_tables_first() {
        let summary = UiSchemaSummary {
            table_details: vec![UiTableSummary {
                schema: "public".to_owned(),
                name: "users".to_owned(),
                row_count: Some(10),
                columns: vec![],
                foreign_keys: vec![],
            }],
            ..Default::default()
        };

        let ctx = CompletionContext {
            text_before_cursor: "SELECT * FROM us",
            text_after_cursor: " WHERE id = 1",
            cursor_offset: 16,
            active_schema: "public",
            schema_summary: &summary,
            cached_tokens: None,
            is_sqlite: false,
            is_manual_trigger: false,
        };

        let (prefix, items) = SchemaCompletionProvider::provide(&ctx);
        assert_eq!(prefix, "us");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "users");
        assert_eq!(items[0].kind, CompletionItemKind::Table);
        assert_eq!(items[0].replacement_range, (14, 16));
    }

    #[test]
    fn test_schema_qualified_table_completion() {
        let summary = UiSchemaSummary {
            table_details: vec![UiTableSummary {
                schema: "public".to_owned(),
                name: "orders".to_owned(),
                row_count: Some(10),
                columns: vec![],
                foreign_keys: vec![],
            }],
            ..Default::default()
        };

        let ctx = CompletionContext {
            text_before_cursor: "SELECT * FROM public.ord",
            text_after_cursor: ";",
            cursor_offset: 24,
            active_schema: "public",
            schema_summary: &summary,
            cached_tokens: None,
            is_sqlite: false,
            is_manual_trigger: false,
        };

        let (prefix, items) = SchemaCompletionProvider::provide(&ctx);
        assert_eq!(prefix, "ord");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "orders");
        assert_eq!(items[0].kind, CompletionItemKind::Table);
        assert_eq!(items[0].replacement_range, (21, 24));
    }

    #[test]
    fn test_cte_alias_resolution_and_column_completion() {
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
                        data_type: "text".to_owned(),
                        nullable: true,
                        is_primary_key: false,
                    },
                ],
                foreign_keys: vec![],
            }],
            ..Default::default()
        };

        let sql = "WITH active_users AS (SELECT * FROM users) SELECT active_users.em";
        let ctx = CompletionContext {
            text_before_cursor: sql,
            text_after_cursor: " FROM active_users",
            cursor_offset: sql.len(),
            active_schema: "public",
            schema_summary: &summary,
            cached_tokens: None,
            is_sqlite: false,
            is_manual_trigger: false,
        };

        let (prefix, items) = SchemaCompletionProvider::provide(&ctx);
        assert_eq!(prefix, "em");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "email");
        assert_eq!(items[0].kind, CompletionItemKind::Column);
        assert_eq!(items[0].detail, Some("Column · text".to_owned()));
        assert_eq!(items[0].replacement_range, (63, 65));
    }

    #[test]
    fn test_multi_segment_schema_table_column_completion() {
        let summary = UiSchemaSummary {
            table_details: vec![UiTableSummary {
                schema: "analytics".to_owned(),
                name: "events".to_owned(),
                row_count: Some(500),
                columns: vec![
                    UiSchemaColumn {
                        name: "event_id".to_owned(),
                        data_type: "uuid".to_owned(),
                        nullable: false,
                        is_primary_key: true,
                    },
                    UiSchemaColumn {
                        name: "payload".to_owned(),
                        data_type: "jsonb".to_owned(),
                        nullable: true,
                        is_primary_key: false,
                    },
                ],
                foreign_keys: vec![],
            }],
            ..Default::default()
        };

        let sql = "SELECT analytics.events.pay";
        let ctx = CompletionContext {
            text_before_cursor: sql,
            text_after_cursor: " FROM analytics.events",
            cursor_offset: sql.len(),
            active_schema: "public",
            schema_summary: &summary,
            cached_tokens: None,
            is_sqlite: false,
            is_manual_trigger: false,
        };

        let (prefix, items) = SchemaCompletionProvider::provide(&ctx);
        assert_eq!(prefix, "pay");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "payload");
        assert_eq!(items[0].kind, CompletionItemKind::Column);
        assert_eq!(items[0].replacement_range, (24, 27));
    }

    #[test]
    fn test_build_ai_sql_context_extracts_references() {
        let summary = UiSchemaSummary {
            table_details: vec![
                UiTableSummary {
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
                            name: "name".to_owned(),
                            data_type: "text".to_owned(),
                            nullable: false,
                            is_primary_key: false,
                        },
                    ],
                    foreign_keys: vec![],
                },
                UiTableSummary {
                    schema: "public".to_owned(),
                    name: "orders".to_owned(),
                    row_count: Some(20),
                    columns: vec![
                        UiSchemaColumn {
                            name: "order_id".to_owned(),
                            data_type: "integer".to_owned(),
                            nullable: false,
                            is_primary_key: true,
                        },
                        UiSchemaColumn {
                            name: "user_id".to_owned(),
                            data_type: "integer".to_owned(),
                            nullable: false,
                            is_primary_key: false,
                        },
                    ],
                    foreign_keys: vec![crate::runtime::UiSchemaForeignKey {
                        name: "fk_orders_users".to_owned(),
                        from_columns: vec!["user_id".to_owned()],
                        to_schema: "public".to_owned(),
                        to_table: "users".to_owned(),
                        to_columns: vec!["id".to_owned()],
                    }],
                },
            ],
            ..Default::default()
        };

        let sql = "SELECT u.name, o.order_id FROM users u JOIN orders o ON u.id = o.user_id WHERE u.";
        let ai_ctx = build_ai_sql_context(sql, sql.len(), "public", &summary);

        assert_eq!(ai_ctx.active_schema, "public");
        assert!(ai_ctx.referenced_tables.contains(&"users".to_owned()));
        assert!(ai_ctx.referenced_tables.contains(&"orders".to_owned()));
        assert!(ai_ctx.relevant_columns.iter().any(|c| c.starts_with("users.name")));
        assert!(ai_ctx.relevant_columns.iter().any(|c| c.starts_with("orders.order_id")));
        assert_eq!(ai_ctx.fk_neighbors.len(), 1);
        assert!(ai_ctx.fk_neighbors[0].contains("orders (user_id) -> public.users (id)"));
    }
}
