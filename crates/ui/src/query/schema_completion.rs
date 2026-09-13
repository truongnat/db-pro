use crate::editor::completion::{CompletionItem, CompletionItemKind};
use crate::editor::prediction::{
    AiSqlContext, MAX_COLUMNS_PER_TABLE, MAX_CTES, MAX_FK_NEIGHBORS, MAX_REFERENCED_TABLES, MAX_SQL_AFTER_CHARS,
    MAX_SQL_BEFORE_CHARS,
};
use crate::editor::syntax::CachedSqlTokens;
use crate::runtime::UiSchemaSummary;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlClause {
    Select,
    From,
    Join,
    Where,
    GroupBy,
    OrderBy,
    Having,
    InsertInto,
    Values,
    Update,
    Set,
    DeleteFrom,
    Returning,
    Unknown,
}

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

#[derive(Debug, Clone)]
pub struct CteDefinition {
    pub name: String,
    pub target_table: Option<String>,
    pub explicit_columns: Vec<String>,
    pub inferred_columns: Vec<String>,
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

        // Check for dot qualifier right before the prefix
        let before_prefix = &text_before[..text_before.len() - prefix.len()];
        let qualifier = extract_qualifier(before_prefix);

        let mut full_doc = String::with_capacity(ctx.text_before_cursor.len() + ctx.text_after_cursor.len() + 1);
        full_doc.push_str(ctx.text_before_cursor);
        full_doc.push(' ');
        full_doc.push_str(ctx.text_after_cursor);

        let ctes = extract_cte_definitions(&full_doc);
        let aliases = extract_table_aliases(&full_doc);
        let subquery_aliases = extract_subquery_aliases(&full_doc);
        let clause = detect_clause_context(before_prefix);
        let mutation_target = extract_mutation_target(&full_doc);

        let mut items = Vec::new();

        if let Some(qualifier) = qualifier {
            let qual_lower = qualifier.to_lowercase();

            // 1. Multi-part qualifier: schema.table. (e.g. "public.users.")
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
                                        if col.nullable { " (NULL)" } else { " (NOT NULL)" }
                                    )),
                                    replacement_range,
                                    sort_score: 950,
                                });
                            }
                        }
                    }
                }
                rank_items(&mut items, &prefix_lower);
                return (prefix.to_owned(), items);
            }

            // 2. Check if qualifier is a schema name (e.g. "public.")
            let is_known_schema = ctx
                .schema_summary
                .schemas
                .iter()
                .any(|s| s.eq_ignore_ascii_case(&qual_lower))
                || ctx
                    .schema_summary
                    .table_details
                    .iter()
                    .any(|t| t.schema.eq_ignore_ascii_case(&qual_lower));

            if is_known_schema {
                for table in &ctx.schema_summary.table_details {
                    if table.schema.eq_ignore_ascii_case(&qual_lower)
                        && table.name.to_lowercase().contains(&prefix_lower)
                    {
                        items.push(CompletionItem {
                            label: table.name.clone(),
                            insert_text: table.name.clone(),
                            kind: CompletionItemKind::Table,
                            detail: Some(format!("Table · {}.{}", table.schema, table.name)),
                            documentation: Some(format!("Table with {} columns", table.columns.len())),
                            replacement_range,
                            sort_score: 920,
                        });
                    }
                }
                for view in &ctx.schema_summary.views {
                    if view.schema.eq_ignore_ascii_case(&qual_lower) && view.name.to_lowercase().contains(&prefix_lower)
                    {
                        items.push(CompletionItem {
                            label: view.name.clone(),
                            insert_text: view.name.clone(),
                            kind: CompletionItemKind::View,
                            detail: Some(format!("View · {}.{}", view.schema, view.name)),
                            documentation: Some("Database View".to_owned()),
                            replacement_range,
                            sort_score: 900,
                        });
                    }
                }
                for func in &ctx.schema_summary.functions {
                    if func.schema.eq_ignore_ascii_case(&qual_lower) && func.name.to_lowercase().contains(&prefix_lower)
                    {
                        items.push(CompletionItem {
                            label: format!("{}()", func.name),
                            insert_text: format!("{}(", func.name),
                            kind: CompletionItemKind::Function,
                            detail: Some(format!("Function · {}.{}", func.schema, func.name)),
                            documentation: (!func.data_type.is_empty()).then(|| format!("Returns: {}", func.data_type)),
                            replacement_range,
                            sort_score: 850,
                        });
                    }
                }
                if !items.is_empty() {
                    rank_items(&mut items, &prefix_lower);
                    return (prefix.to_owned(), items);
                }
            }

            // 3. Check if qualifier is a defined CTE
            if let Some(cte) = ctes.get(&qual_lower) {
                // If CTE has explicit column names, suggest them first
                if !cte.explicit_columns.is_empty() {
                    for col in &cte.explicit_columns {
                        if col.to_lowercase().contains(&prefix_lower) {
                            items.push(CompletionItem {
                                label: col.clone(),
                                insert_text: col.clone(),
                                kind: CompletionItemKind::Column,
                                detail: Some(format!("Column · CTE {}", cte.name)),
                                documentation: None,
                                replacement_range,
                                sort_score: 950,
                            });
                        }
                    }
                }
                for col in &cte.inferred_columns {
                    if col.to_lowercase().contains(&prefix_lower)
                        && !items.iter().any(|item| item.label.eq_ignore_ascii_case(col))
                    {
                        items.push(CompletionItem {
                            label: col.clone(),
                            insert_text: col.clone(),
                            kind: CompletionItemKind::Column,
                            detail: Some(format!("Column · CTE {}", cte.name)),
                            documentation: Some("Inferred from the CTE SELECT list".to_owned()),
                            replacement_range,
                            sort_score: 940,
                        });
                    }
                }
                // If CTE targets an underlying table, suggest table columns
                if let Some(target) = &cte.target_table {
                    for table in &ctx.schema_summary.table_details {
                        if table.name.eq_ignore_ascii_case(target) {
                            for col in &table.columns {
                                if col.name.to_lowercase().contains(&prefix_lower)
                                    && !items.iter().any(|i| i.label == col.name)
                                {
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
                                            "CTE {} -> {}.{}",
                                            cte.name, table.schema, table.name
                                        )),
                                        replacement_range,
                                        sort_score: 930,
                                    });
                                }
                            }
                        }
                    }
                }
                if !items.is_empty() {
                    rank_items(&mut items, &prefix_lower);
                    return (prefix.to_owned(), items);
                }
            }

            if let Some(columns) = subquery_aliases.get(&qual_lower) {
                for column in columns {
                    if column.to_lowercase().contains(&prefix_lower) {
                        items.push(CompletionItem {
                            label: column.clone(),
                            insert_text: column.clone(),
                            kind: CompletionItemKind::Column,
                            detail: Some(format!("Column · subquery {qualifier}")),
                            documentation: Some("Inferred from the subquery SELECT list".to_owned()),
                            replacement_range,
                            sort_score: 940,
                        });
                    }
                }
                if !items.is_empty() {
                    rank_items(&mut items, &prefix_lower);
                    return (prefix.to_owned(), items);
                }
            }

            // 4. Check if qualifier is a table name or an alias in the document
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
                                    if col.nullable { " (NULL)" } else { " (NOT NULL)" }
                                )),
                                replacement_range,
                                sort_score: 950,
                            });
                        }
                    }
                }
            }

            rank_items(&mut items, &prefix_lower);
            return (prefix.to_owned(), items);
        }

        // Clause-driven completions when no dot qualifier is present:

        // 1. FROM / JOIN clauses: suggest Tables, Views, CTEs, Schemas
        if matches!(clause, SqlClause::From | SqlClause::Join) {
            // Suggest CTE names
            for cte in ctes.values() {
                if cte.name.to_lowercase().contains(&prefix_lower) {
                    items.push(CompletionItem {
                        label: cte.name.clone(),
                        insert_text: cte.name.clone(),
                        kind: CompletionItemKind::Cte,
                        detail: Some("Common Table Expression".to_owned()),
                        documentation: None,
                        replacement_range,
                        sort_score: 960,
                    });
                }
            }

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
                        sort_score: 920,
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
                            sort_score: 900,
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
                        sort_score: 880,
                    });
                }
            }
            // Suggest Schemas
            for schema in &ctx.schema_summary.schemas {
                if schema.to_lowercase().contains(&prefix_lower) {
                    items.push(CompletionItem {
                        label: format!("{schema}."),
                        insert_text: format!("{schema}."),
                        kind: CompletionItemKind::Schema,
                        detail: Some("Database Schema".to_owned()),
                        documentation: None,
                        replacement_range,
                        sort_score: 850,
                    });
                }
            }
            // Keywords for FROM / JOIN
            add_clause_keywords(&mut items, clause, &prefix_lower, replacement_range);
            rank_items(&mut items, &prefix_lower);
            return (prefix.to_owned(), items);
        }

        // 2. SELECT / WHERE / GROUP BY / ORDER BY / HAVING / RETURNING clauses:
        // Prioritize columns from referenced tables/aliases and detect ambiguous columns
        let mut referenced_table_names = Vec::new();
        if let Some(target) = mutation_target {
            referenced_table_names.push(target);
        }
        let mut alias_tables: Vec<String> = aliases.values().cloned().collect();
        alias_tables.sort_unstable();
        alias_tables.dedup();
        for t in &alias_tables {
            if !referenced_table_names.contains(t) {
                referenced_table_names.push(t.clone());
            }
        }
        for table in &ctx.schema_summary.table_details {
            if full_doc.to_lowercase().contains(&table.name.to_lowercase())
                && !referenced_table_names
                    .iter()
                    .any(|t| t.eq_ignore_ascii_case(&table.name))
            {
                referenced_table_names.push(table.name.clone());
            }
        }

        // Detect column name frequencies across referenced tables for ambiguity check
        let mut col_frequencies: HashMap<String, usize> = HashMap::new();
        for t_name in &referenced_table_names {
            if let Some(t) = ctx
                .schema_summary
                .table_details
                .iter()
                .find(|t| t.name.eq_ignore_ascii_case(t_name))
            {
                for c in &t.columns {
                    *col_frequencies.entry(c.name.to_lowercase()).or_insert(0) += 1;
                }
            }
        }

        // Suggest columns from referenced tables
        for t_name in &referenced_table_names {
            if let Some(table) = ctx
                .schema_summary
                .table_details
                .iter()
                .find(|t| t.name.eq_ignore_ascii_case(t_name))
            {
                // Find alias if exists
                let alias = aliases
                    .iter()
                    .find(|(_, val)| val.eq_ignore_ascii_case(&table.name))
                    .map(|(k, _)| k.as_str());
                for col in &table.columns {
                    if col.name.to_lowercase().contains(&prefix_lower) {
                        let is_ambiguous = col_frequencies.get(&col.name.to_lowercase()).copied().unwrap_or(0) > 1;
                        if let Some(a) = alias {
                            // Suggest qualified alias.col
                            items.push(CompletionItem {
                                label: format!("{a}.{}", col.name),
                                insert_text: format!("{a}.{}", col.name),
                                kind: CompletionItemKind::Column,
                                detail: Some(format!(
                                    "Column · {} ({}){}",
                                    col.data_type,
                                    table.name,
                                    if col.is_primary_key { " [PK]" } else { "" }
                                )),
                                documentation: None,
                                replacement_range,
                                sort_score: if is_ambiguous { 960 } else { 920 },
                            });
                        }
                        // Suggest bare col
                        items.push(CompletionItem {
                            label: col.name.clone(),
                            insert_text: col.name.clone(),
                            kind: CompletionItemKind::Column,
                            detail: Some(format!(
                                "Column · {} ({}){}",
                                col.data_type,
                                table.name,
                                if col.is_primary_key { " [PK]" } else { "" }
                            )),
                            documentation: None,
                            replacement_range,
                            sort_score: if is_ambiguous { 820 } else { 940 },
                        });
                    }
                }
            }
        }

        // Suggest CTE columns if any defined
        for cte in ctes.values() {
            for col in cte.explicit_columns.iter().chain(cte.inferred_columns.iter()) {
                if col.to_lowercase().contains(&prefix_lower) {
                    items.push(CompletionItem {
                        label: format!("{}.{col}", cte.name),
                        insert_text: format!("{}.{col}", cte.name),
                        kind: CompletionItemKind::Column,
                        detail: Some(format!("Column · CTE {}", cte.name)),
                        documentation: None,
                        replacement_range,
                        sort_score: 930,
                    });
                }
            }
        }

        // Fallback: suggest all table columns in active schema
        if items.is_empty() {
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
                            sort_score: 800,
                        });
                    }
                }
            }
        }

        // Suggest Tables and Views
        for table in &ctx.schema_summary.table_details {
            if table.name.to_lowercase().contains(&prefix_lower) {
                items.push(CompletionItem {
                    label: table.name.clone(),
                    insert_text: table.name.clone(),
                    kind: CompletionItemKind::Table,
                    detail: Some(format!("Table · {}.{}", table.schema, table.name)),
                    documentation: None,
                    replacement_range,
                    sort_score: 700,
                });
            }
        }

        // Suggest Functions
        for func in &ctx.schema_summary.functions {
            if func.name.to_lowercase().contains(&prefix_lower) {
                items.push(CompletionItem {
                    label: format!("{}()", func.name),
                    insert_text: format!("{}(", func.name),
                    kind: CompletionItemKind::Function,
                    detail: Some(format!("Function · {}.{}", func.schema, func.name)),
                    documentation: (!func.data_type.is_empty()).then(|| format!("Returns: {}", func.data_type)),
                    replacement_range,
                    sort_score: 650,
                });
            }
        }

        // Keywords
        add_clause_keywords(&mut items, clause, &prefix_lower, replacement_range);

        rank_items(&mut items, &prefix_lower);
        (prefix.to_owned(), items)
    }

    pub fn build_ai_sql_context(
        before_cursor: &str,
        after_cursor: &str,
        cursor_offset: usize,
        active_schema: &str,
        schema_summary: &UiSchemaSummary,
        is_sqlite: bool,
    ) -> AiSqlContext {
        let full_text = format!("{before_cursor}{after_cursor}");
        let mut ctx = self::build_ai_sql_context(&full_text, cursor_offset, active_schema, schema_summary);
        ctx.dialect = if is_sqlite {
            "SQLite".to_owned()
        } else {
            "PostgreSQL".to_owned()
        };
        ctx
    }
}

pub fn detect_clause_context(text_before: &str) -> SqlClause {
    let upper = text_before.to_ascii_uppercase();
    let tokens: Vec<&str> = upper.split_whitespace().collect();
    if tokens.is_empty() {
        return SqlClause::Unknown;
    }

    // Scan backwards from last token
    for &w in tokens.iter().rev() {
        match w {
            "FROM" => return SqlClause::From,
            "JOIN" | "CROSS" | "INNER" | "LEFT" | "RIGHT" | "OUTER" => return SqlClause::Join,
            "WHERE" => return SqlClause::Where,
            "SET" => return SqlClause::Set,
            "UPDATE" => return SqlClause::Update,
            "VALUES" => return SqlClause::Values,
            "INTO" => return SqlClause::InsertInto,
            "DELETE" => return SqlClause::DeleteFrom,
            "HAVING" => return SqlClause::Having,
            "ORDER" => return SqlClause::OrderBy,
            "GROUP" => return SqlClause::GroupBy,
            "RETURNING" => return SqlClause::Returning,
            "SELECT" => return SqlClause::Select,
            _ => {}
        }
    }
    SqlClause::Unknown
}

fn add_clause_keywords(
    items: &mut Vec<CompletionItem>,
    clause: SqlClause,
    prefix_lower: &str,
    replacement_range: (usize, usize),
) {
    let keywords: &[&str] = match clause {
        SqlClause::Select => &[
            "DISTINCT", "FROM", "AS", "CASE", "WHEN", "THEN", "ELSE", "END", "COUNT", "SUM", "AVG", "MIN", "MAX",
            "COALESCE", "NOW()",
        ],
        SqlClause::From | SqlClause::Join => &[
            "ON",
            "USING",
            "LEFT JOIN",
            "INNER JOIN",
            "RIGHT JOIN",
            "FULL JOIN",
            "CROSS JOIN",
            "WHERE",
            "AS",
            "GROUP BY",
            "ORDER BY",
            "LIMIT",
        ],
        SqlClause::Where | SqlClause::Having => &[
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
            "GROUP BY",
            "ORDER BY",
            "LIMIT",
        ],
        SqlClause::GroupBy => &["HAVING", "ORDER BY", "LIMIT"],
        SqlClause::OrderBy => &["ASC", "DESC", "NULLS FIRST", "NULLS LAST", "LIMIT", "OFFSET"],
        SqlClause::Update => &["SET", "WHERE"],
        SqlClause::Set => &["WHERE", "RETURNING"],
        SqlClause::InsertInto => &["VALUES", "SELECT", "RETURNING"],
        SqlClause::DeleteFrom => &["WHERE", "RETURNING"],
        _ => &[
            "SELECT",
            "FROM",
            "WHERE",
            "JOIN",
            "LEFT JOIN",
            "INNER JOIN",
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
            "INSERT INTO",
            "VALUES",
            "UPDATE",
            "SET",
            "DELETE FROM",
            "COUNT",
            "SUM",
            "AVG",
            "COALESCE",
            "NOW()",
        ],
    };

    for kw in keywords {
        if kw.to_lowercase().contains(prefix_lower) {
            items.push(CompletionItem {
                label: (*kw).to_owned(),
                insert_text: (*kw).to_owned(),
                kind: CompletionItemKind::Keyword,
                detail: Some("SQL Keyword".to_owned()),
                documentation: None,
                replacement_range,
                sort_score: 100,
            });
        }
    }
}

pub fn build_ai_sql_context(
    doc_text: &str,
    cursor_offset: usize,
    active_schema: &str,
    schema_summary: &UiSchemaSummary,
) -> AiSqlContext {
    let cursor = cursor_offset.min(doc_text.len());
    let (before_cursor, after_cursor) = doc_text.split_at(cursor);
    let statement_start = before_cursor.rfind(';').map_or(0, |offset| offset + 1);
    let statement_end = after_cursor
        .find(';')
        .map_or(doc_text.len(), |offset| cursor + offset + 1);
    let current_statement = doc_text[statement_start..statement_end].trim().to_owned();

    // Truncate SQL before cursor deterministically
    let truncated_before = if before_cursor.len() > MAX_SQL_BEFORE_CHARS {
        let start = before_cursor.len() - MAX_SQL_BEFORE_CHARS;
        &before_cursor[start..]
    } else {
        before_cursor
    };

    // Small suffix after cursor
    let truncated_after = if after_cursor.len() > MAX_SQL_AFTER_CHARS {
        &after_cursor[..MAX_SQL_AFTER_CHARS]
    } else {
        after_cursor
    };

    let normalized_sql = doc_text.to_lowercase();
    let ctes = extract_cte_definitions(doc_text);
    let mut cte_names: Vec<String> = ctes.keys().cloned().collect();
    cte_names.sort_unstable();
    cte_names.truncate(MAX_CTES);

    let aliases = extract_table_aliases(doc_text);
    let mut referenced_tables = Vec::new();
    let mut alias_tables: Vec<String> = aliases.values().cloned().collect();
    alias_tables.sort_unstable();
    alias_tables.dedup();
    for table_val in &alias_tables {
        if !referenced_tables.contains(table_val) {
            referenced_tables.push(table_val.clone());
        }
    }
    for table in &schema_summary.table_details {
        if normalized_sql.contains(&table.name.to_lowercase())
            && !referenced_tables.iter().any(|t| t.eq_ignore_ascii_case(&table.name))
        {
            referenced_tables.push(table.name.clone());
        }
    }
    referenced_tables.truncate(MAX_REFERENCED_TABLES);

    let mut relevant_columns = Vec::new();
    let mut fk_neighbors = Vec::new();

    for table_name in &referenced_tables {
        if let Some(table) = schema_summary
            .table_details
            .iter()
            .find(|t| t.name.eq_ignore_ascii_case(table_name))
        {
            for col in table.columns.iter().take(MAX_COLUMNS_PER_TABLE) {
                relevant_columns.push(format!("{}.{} ({})", table.name, col.name, col.data_type));
            }
            for fk in table.foreign_keys.iter().take(MAX_FK_NEIGHBORS) {
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
    fk_neighbors.truncate(MAX_FK_NEIGHBORS);

    AiSqlContext {
        sql_before_cursor: truncated_before.to_owned(),
        sql_after_cursor: truncated_after.to_owned(),
        current_statement,
        active_schema: active_schema.to_owned(),
        dialect: "PostgreSQL".to_owned(),
        referenced_tables,
        table_aliases: aliases,
        relevant_columns,
        fk_neighbors,
        cte_names,
    }
}

fn rank_items(items: &mut [CompletionItem], prefix: &str) {
    items.sort_by(|a, b| {
        let a_exact = a.label.to_lowercase() == prefix;
        let b_exact = b.label.to_lowercase() == prefix;
        if a_exact != b_exact {
            return b_exact.cmp(&a_exact);
        }

        let a_starts = a.label.to_lowercase().starts_with(prefix);
        let b_starts = b.label.to_lowercase().starts_with(prefix);
        if a_starts != b_starts {
            return b_starts.cmp(&a_starts);
        }

        b.sort_score.cmp(&a.sort_score).then_with(|| a.label.cmp(&b.label))
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

pub fn extract_cte_definitions(text: &str) -> HashMap<String, CteDefinition> {
    let mut ctes = HashMap::new();
    let normalized = text.replace('(', " ( ").replace(')', " ) ").replace(',', " , ");
    let words: Vec<&str> = normalized.split_whitespace().collect();
    let mut i = 0;
    while i < words.len() {
        if !words[i].eq_ignore_ascii_case("WITH") {
            i += 1;
            continue;
        }
        let mut cursor = i + 1;
        if words
            .get(cursor)
            .is_some_and(|word| word.eq_ignore_ascii_case("RECURSIVE"))
        {
            cursor += 1;
        }
        while let Some(raw_name) = words.get(cursor) {
            let name = clean_cte_token(raw_name);
            if name.is_empty() {
                break;
            }
            cursor += 1;
            let mut explicit_columns = Vec::new();
            if words.get(cursor) == Some(&"(") {
                cursor += 1;
                while let Some(word) = words.get(cursor) {
                    cursor += 1;
                    if *word == ")" {
                        break;
                    }
                    if *word != "," {
                        explicit_columns.push(clean_cte_token(word));
                    }
                }
            }
            if words.get(cursor).is_none_or(|word| !word.eq_ignore_ascii_case("AS")) {
                break;
            }
            cursor += 1;
            if words.get(cursor) != Some(&"(") {
                break;
            }
            cursor += 1;
            let body_start = cursor;
            let mut depth = 1usize;
            while let Some(word) = words.get(cursor) {
                if *word == "(" {
                    depth += 1;
                } else if *word == ")" {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        break;
                    }
                }
                cursor += 1;
            }
            let body = &words[body_start..cursor.min(words.len())];
            let target_table = find_cte_source_table(body);
            let inferred_columns = infer_cte_columns(body);
            ctes.insert(
                name.to_lowercase(),
                CteDefinition {
                    name,
                    target_table,
                    explicit_columns,
                    inferred_columns,
                },
            );
            if words.get(cursor) == Some(&")") {
                cursor += 1;
            }
            if words.get(cursor) != Some(&",") {
                break;
            }
            cursor += 1;
        }
        i = cursor;
    }
    ctes
}

fn clean_cte_token(token: &str) -> String {
    token
        .trim_matches(|character| matches!(character, '"' | '`' | '[' | ']' | ';'))
        .to_owned()
}

fn find_cte_source_table(body: &[&str]) -> Option<String> {
    body.windows(2)
        .find(|window| window[0].eq_ignore_ascii_case("FROM") && window[1] != "(")
        .map(|window| clean_cte_token(window[1]))
        .filter(|table| !table.is_empty())
        .map(|table| table.rsplit('.').next().unwrap_or(&table).to_owned())
}

fn infer_cte_columns(body: &[&str]) -> Vec<String> {
    let Some(select_index) = body.iter().position(|word| word.eq_ignore_ascii_case("SELECT")) else {
        return Vec::new();
    };
    let Some(from_index) = body[select_index + 1..]
        .iter()
        .position(|word| word.eq_ignore_ascii_case("FROM"))
        .map(|index| select_index + 1 + index)
    else {
        return Vec::new();
    };
    body[select_index + 1..from_index]
        .iter()
        .filter(|word| **word != "," && **word != "*")
        .filter_map(|word| {
            let cleaned = clean_cte_token(word);
            let column = cleaned.rsplit('.').next().unwrap_or(&cleaned);
            (!column.is_empty() && !column.eq_ignore_ascii_case("AS")).then(|| column.to_owned())
        })
        .collect()
}

pub fn extract_table_aliases(text: &str) -> HashMap<String, String> {
    let mut aliases = HashMap::new();
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut i = 0;
    while i < words.len() {
        let w = words[i];
        if (w.eq_ignore_ascii_case("FROM")
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

fn extract_mutation_target(text: &str) -> Option<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    for (index, word) in words.iter().enumerate() {
        let is_target_keyword = word.eq_ignore_ascii_case("UPDATE") || word.eq_ignore_ascii_case("INTO");
        if is_target_keyword {
            let target = words
                .get(index + 1)?
                .trim_matches(|character| matches!(character, '"' | '`' | '[' | ']' | '(' | ')' | ',' | ';'));
            if !target.is_empty() && !target.eq_ignore_ascii_case("SELECT") {
                return Some(target.rsplit('.').next().unwrap_or(target).to_lowercase());
            }
        }
    }
    None
}

fn extract_subquery_aliases(text: &str) -> HashMap<String, Vec<String>> {
    let normalized = text.replace('(', " ( ").replace(')', " ) ").replace(',', " , ");
    let words: Vec<&str> = normalized.split_whitespace().collect();
    let mut aliases = HashMap::new();
    for index in 0..words.len().saturating_sub(1) {
        if !matches!(words[index].to_ascii_uppercase().as_str(), "FROM" | "JOIN") || words.get(index + 1) != Some(&"(")
        {
            continue;
        }
        let body_start = index + 2;
        let mut cursor = body_start;
        let mut depth = 1usize;
        while let Some(word) = words.get(cursor) {
            if *word == "(" {
                depth += 1;
            } else if *word == ")" {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    break;
                }
            }
            cursor += 1;
        }
        let Some(alias_token) = words.get(cursor + 1) else {
            continue;
        };
        let alias = if alias_token.eq_ignore_ascii_case("AS") {
            words.get(cursor + 2).copied().unwrap_or_default()
        } else {
            alias_token
        };
        let alias = clean_cte_token(alias);
        if !alias.is_empty() && is_valid_alias_token(&alias) {
            let columns = infer_cte_columns(&words[body_start..cursor.min(words.len())]);
            if !columns.is_empty() {
                aliases.insert(alias.to_lowercase(), columns);
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
        assert!(!items.is_empty());
        assert_eq!(items[0].label, "users");
        assert_eq!(items[0].kind, CompletionItemKind::Table);
        assert_eq!(items[0].replacement_range, (14, 16));
        assert!(items.iter().any(|i| i.label == "USING"));
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
    fn explicit_cte_columns_are_parsed_across_whitespace() {
        let ctes = extract_cte_definitions(
            "WITH visible (id, display_name) AS (SELECT id, name FROM users) SELECT * FROM visible",
        );
        let cte = ctes.get("visible").expect("CTE should be indexed");
        assert_eq!(cte.explicit_columns, ["id", "display_name"]);
        assert_eq!(cte.target_table.as_deref(), Some("users"));
    }

    #[test]
    fn simple_cte_select_list_is_available_for_qualified_completion() {
        let summary = UiSchemaSummary::default();
        let sql = "WITH x AS (SELECT id, email FROM users) SELECT x.";
        let ctx = CompletionContext {
            text_before_cursor: sql,
            text_after_cursor: "",
            cursor_offset: sql.len(),
            active_schema: "public",
            schema_summary: &summary,
            cached_tokens: None,
            is_sqlite: false,
            is_manual_trigger: true,
        };
        let (_, items) = SchemaCompletionProvider::provide(&ctx);
        let labels: Vec<_> = items.iter().map(|item| item.label.as_str()).collect();
        assert!(labels.contains(&"id"));
        assert!(labels.contains(&"email"));
    }

    #[test]
    fn update_set_completion_prioritizes_update_target_columns() {
        let summary = UiSchemaSummary {
            table_details: vec![UiTableSummary {
                schema: "public".to_owned(),
                name: "users".to_owned(),
                row_count: None,
                columns: vec![UiSchemaColumn {
                    name: "email".to_owned(),
                    data_type: "text".to_owned(),
                    nullable: true,
                    is_primary_key: false,
                }],
                foreign_keys: vec![],
            }],
            ..Default::default()
        };
        let sql = "UPDATE users SET em";
        let ctx = CompletionContext {
            text_before_cursor: sql,
            text_after_cursor: "",
            cursor_offset: sql.len(),
            active_schema: "public",
            schema_summary: &summary,
            cached_tokens: None,
            is_sqlite: false,
            is_manual_trigger: true,
        };
        let (_, items) = SchemaCompletionProvider::provide(&ctx);
        assert_eq!(items.first().map(|item| item.label.as_str()), Some("email"));
    }

    #[test]
    fn subquery_alias_exposes_simple_select_columns() {
        let summary = UiSchemaSummary::default();
        let sql = "SELECT u.em";
        let ctx = CompletionContext {
            text_before_cursor: sql,
            text_after_cursor: " FROM (SELECT id, email FROM users) u WHERE ",
            cursor_offset: 11,
            active_schema: "public",
            schema_summary: &summary,
            cached_tokens: None,
            is_sqlite: false,
            is_manual_trigger: true,
        };
        let (_, items) = SchemaCompletionProvider::provide(&ctx);
        assert_eq!(items.first().map(|item| item.label.as_str()), Some("email"));
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

    #[test]
    fn ai_context_contains_only_the_current_statement() {
        let sql = "SELECT 1; SELECT u.id FROM users u WHERE u.";
        let ai_ctx = build_ai_sql_context(sql, sql.len(), "public", &UiSchemaSummary::default());
        assert_eq!(ai_ctx.current_statement, "SELECT u.id FROM users u WHERE u.");
        assert_eq!(ai_ctx.sql_before_cursor, sql);
    }
}
