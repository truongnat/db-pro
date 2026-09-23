use crate::editor::{CachedSqlTokens, SqlDialect, SyntaxTokenKind};
use crate::runtime::{UiFunctionSummary, UiSchemaColumn, UiSchemaForeignKey, UiSchemaSummary, UiTableSummary};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlSymbolHelp {
    pub title: String,
    pub kind: String,
    pub detail: String,
    pub documentation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlSignatureHelp {
    pub label: String,
    pub parameters: Vec<String>,
    pub active_parameter: usize,
    pub documentation: String,
}

/// Column summary as shown in the table hover card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverColumn {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_primary_key: bool,
    pub is_foreign_key: bool,
}

/// FK reference as shown in the table hover card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverForeignKey {
    pub name: String,
    pub from_columns: Vec<String>,
    pub to_table: String,
    pub to_columns: Vec<String>,
}

/// Rich structured hover content, rendered by `draw_hover_popup`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RichHoverHelp {
    /// A table or view: name, row count, columns, FK list.
    Table {
        qualified_name: String,
        kind: String, // "Table" | "View"
        row_count: Option<u64>,
        columns: Vec<HoverColumn>,
        foreign_keys: Vec<HoverForeignKey>,
    },
    /// A single column: name, type, flags.
    Column {
        qualified_name: String,
        data_type: String,
        nullable: bool,
        is_primary_key: bool,
        parent_table: String,
        foreign_key_target: Option<String>,
    },
    /// A function or procedure with its signature.
    Function {
        label: String,
        parameters: Vec<String>,
        active_parameter: usize,
        return_type: String,
        documentation: String,
    },
    /// A SQL keyword with reference documentation.
    Keyword {
        keyword: String,
        dialect_note: String,
        documentation: String,
        example: Option<String>,
    },
    /// Fallback: plain symbol info (used for unrecognized identifiers).
    Symbol(SqlSymbolHelp),
}

#[derive(Debug, Clone, Default)]
pub struct SchemaSymbolIndex {
    tables: HashMap<String, Vec<SqlSymbolHelp>>,
    views: HashMap<String, Vec<SqlSymbolHelp>>,
    columns: HashMap<String, Vec<SqlSymbolHelp>>,
    functions: HashMap<String, Vec<SqlSignatureHelp>>,
}

impl SchemaSymbolIndex {
    pub fn build(schema: &UiSchemaSummary) -> Self {
        let mut index = Self::default();
        for table in &schema.table_details {
            let qualified_name = format!("{}.{}", table.schema, table.name);
            index.insert_table(
                &table.name,
                &qualified_name,
                SqlSymbolHelp {
                    title: qualified_name.clone(),
                    kind: "Table".to_owned(),
                    detail: format!("{} columns", table.columns.len()),
                    documentation: table.row_count.map_or_else(
                        || "Database table".to_owned(),
                        |rows| format!("Database table · approximately {rows} rows"),
                    ),
                },
            );
            for column in &table.columns {
                let help = SqlSymbolHelp {
                    title: format!("{}.{}.{}", table.schema, table.name, column.name),
                    kind: "Column".to_owned(),
                    detail: format!(
                        "{}{}{}",
                        column.data_type,
                        if column.nullable {
                            " · nullable"
                        } else {
                            " · not null"
                        },
                        if column.is_primary_key { " · primary key" } else { "" }
                    ),
                    documentation: format!("Column of {qualified_name}"),
                };
                index
                    .columns
                    .entry(normalize(&column.name))
                    .or_default()
                    .push(help.clone());
                index
                    .columns
                    .entry(normalize(&format!("{}.{}", table.name, column.name)))
                    .or_default()
                    .push(help.clone());
                index
                    .columns
                    .entry(normalize(&format!("{}.{}.{}", table.schema, table.name, column.name)))
                    .or_default()
                    .push(help);
            }
        }
        for view in &schema.views {
            let qualified_name = format!("{}.{}", view.schema, view.name);
            let help = SqlSymbolHelp {
                title: qualified_name.clone(),
                kind: "View".to_owned(),
                detail: "Database view".to_owned(),
                documentation: summarize_definition(&view.definition),
            };
            index.views.entry(normalize(&view.name)).or_default().push(help.clone());
            index.views.entry(normalize(&qualified_name)).or_default().push(help);
        }
        for function in &schema.functions {
            index.insert_function(function);
        }
        index
    }

    pub fn resolve_symbol(
        &self,
        sql: &str,
        token_range: (usize, usize),
        active_schema: &str,
        dialect: SqlDialect,
    ) -> Option<SqlSymbolHelp> {
        let token = sql.get(token_range.0..token_range.1)?;
        if let Some(help) = keyword_help(token, dialect) {
            return Some(help);
        }

        let qualified = qualified_name_at(sql, token_range);
        self.resolve_named(&qualified, active_schema)
            .or_else(|| builtin_signature(token, dialect).map(|signature| signature_symbol_help(&signature)))
    }

    pub fn signature_help(
        &self,
        sql: &str,
        cursor_offset: usize,
        active_schema: &str,
        dialect: SqlDialect,
        tokens: &CachedSqlTokens,
    ) -> Option<SqlSignatureHelp> {
        let call = function_call_context(sql, cursor_offset, tokens)?;
        let normalized_call = normalize(&call.name);
        let mut candidates = self.functions.get(&normalized_call).cloned().unwrap_or_default();
        if !call.name.contains('.') && !active_schema.is_empty() {
            let qualified = normalize(&format!("{active_schema}.{}", call.name));
            if let Some(active_schema_candidates) = self.functions.get(&qualified) {
                candidates.splice(0..0, active_schema_candidates.iter().cloned());
            }
        }
        if let Some(builtin) = builtin_signature(&call.name, dialect) {
            candidates.push(builtin);
        }
        candidates.sort_by(|left, right| left.label.cmp(&right.label));
        candidates.dedup_by(|left, right| left.label == right.label);

        candidates
            .into_iter()
            .find(|candidate| candidate.parameters.len() > call.active_argument)
            .or_else(|| builtin_signature(&call.name, dialect))
            .map(|mut signature| {
                signature.active_parameter = call.active_argument.min(signature.parameters.len().saturating_sub(1));
                signature
            })
    }

    fn insert_table(&mut self, name: &str, qualified_name: &str, help: SqlSymbolHelp) {
        self.tables.entry(normalize(name)).or_default().push(help.clone());
        self.tables.entry(normalize(qualified_name)).or_default().push(help);
    }

    fn insert_function(&mut self, function: &UiFunctionSummary) {
        let signature = function_signature(function);
        self.functions
            .entry(normalize(&function.name))
            .or_default()
            .push(signature.clone());
        self.functions
            .entry(normalize(&format!("{}.{}", function.schema, function.name)))
            .or_default()
            .push(signature);
    }

    fn resolve_named(&self, name: &str, active_schema: &str) -> Option<SqlSymbolHelp> {
        let normalized = normalize(name);
        for collection in [&self.columns, &self.tables, &self.views] {
            if let Some(help) = resolve_unambiguous(collection.get(&normalized), active_schema) {
                return Some(help);
            }
        }
        self.functions
            .get(&normalized)
            .and_then(|signatures| signatures.first())
            .map(signature_symbol_help)
    }

    /// Rich hover that returns structured content instead of a flat `SqlSymbolHelp`.
    /// This is used by the hover popup renderer for DBeaver/Zed-style cards.
    pub fn rich_hover(
        &self,
        sql: &str,
        token_range: (usize, usize),
        active_schema: &str,
        dialect: SqlDialect,
        schema: &UiSchemaSummary,
    ) -> Option<RichHoverHelp> {
        let token = sql.get(token_range.0..token_range.1)?;

        // 1. Keyword check first (no schema lookup needed).
        if let Some(kw) = rich_keyword_help(token, dialect) {
            return Some(kw);
        }

        let qualified = qualified_name_at(sql, token_range);
        let qualified_lower = normalize(&qualified);

        // 2. Try to match a table (prefer active schema).
        for table in &schema.table_details {
            let table_key = normalize(&format!("{}.{}", table.schema, table.name));
            let short_key = normalize(&table.name);
            if table_key == qualified_lower
                || (short_key == qualified_lower && table.schema.eq_ignore_ascii_case(active_schema))
            {
                return Some(rich_table_hover(table, "Table", schema));
            }
        }

        // 3. Try to match a view.
        for view in &schema.views {
            let view_key = normalize(&format!("{}.{}", view.schema, view.name));
            let short_key = normalize(&view.name);
            if view_key == qualified_lower
                || (short_key == qualified_lower && view.schema.eq_ignore_ascii_case(active_schema))
            {
                return Some(RichHoverHelp::Table {
                    qualified_name: format!("{}.{}", view.schema, view.name),
                    kind: "View".to_owned(),
                    row_count: None,
                    columns: Vec::new(),
                    foreign_keys: Vec::new(),
                });
            }
        }

        // 4. Try to match a column (qualified: table.col or schema.table.col).
        if let Some(hover) = rich_column_hover(&qualified, active_schema, schema) {
            return Some(hover);
        }

        // 5. Try function signature.
        if let Some(sig) = self.functions.get(&qualified_lower).and_then(|s| s.first()) {
            let params: Vec<String> = sig.parameters.clone();
            return Some(RichHoverHelp::Function {
                label: sig.label.clone(),
                parameters: params,
                active_parameter: 0,
                return_type: String::new(),
                documentation: sig.documentation.clone(),
            });
        }

        // 6. Builtin function signature.
        if let Some(sig) = builtin_signature(token, dialect) {
            return Some(RichHoverHelp::Function {
                label: sig.label.clone(),
                parameters: sig.parameters.clone(),
                active_parameter: 0,
                return_type: String::new(),
                documentation: sig.documentation.clone(),
            });
        }

        // 7. Fall back to flat symbol help.
        self.resolve_symbol(sql, token_range, active_schema, dialect)
            .map(RichHoverHelp::Symbol)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FunctionCallContext {
    name: String,
    active_argument: usize,
}

fn function_call_context(sql: &str, cursor_offset: usize, tokens: &CachedSqlTokens) -> Option<FunctionCallContext> {
    if cursor_offset > sql.len() || !sql.is_char_boundary(cursor_offset) {
        return None;
    }
    if cursor_offset < sql.len() && is_literal_or_comment(tokens, cursor_offset) {
        return None;
    }

    let mut depth = 0usize;
    let mut open_paren = None;
    for (offset, character) in sql[..cursor_offset].char_indices().rev() {
        if is_literal_or_comment(tokens, offset) {
            continue;
        }
        match character {
            ')' => depth = depth.saturating_add(1),
            '(' if depth == 0 => {
                open_paren = Some(offset);
                break;
            }
            '(' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    let open_paren = open_paren?;
    let name = sql_identifier_before(sql, open_paren)?;
    if keyword_help(&name, SqlDialect::Generic).is_some() {
        return None;
    }

    let mut nested_depth = 0usize;
    let mut active_argument = 0usize;
    for (relative_offset, character) in sql[open_paren + 1..cursor_offset].char_indices() {
        let offset = open_paren + 1 + relative_offset;
        if is_literal_or_comment(tokens, offset) {
            continue;
        }
        match character {
            '(' | '[' => nested_depth = nested_depth.saturating_add(1),
            ')' | ']' => nested_depth = nested_depth.saturating_sub(1),
            ',' if nested_depth == 0 => active_argument = active_argument.saturating_add(1),
            _ => {}
        }
    }

    Some(FunctionCallContext { name, active_argument })
}

fn is_literal_or_comment(tokens: &CachedSqlTokens, offset: usize) -> bool {
    tokens.token_at(offset).is_some_and(|token| {
        matches!(
            token.kind,
            SyntaxTokenKind::String | SyntaxTokenKind::DollarQuote | SyntaxTokenKind::Comment
        )
    })
}

fn sql_identifier_before(sql: &str, offset: usize) -> Option<String> {
    let prefix = sql.get(..offset)?.trim_end();
    let start = prefix
        .char_indices()
        .rev()
        .find(|(_, character)| !is_identifier_character(*character))
        .map_or(0, |(index, character)| index + character.len_utf8());
    let name = prefix.get(start..)?.trim_matches('"');
    (!name.is_empty()).then(|| name.to_owned())
}

fn qualified_name_at(sql: &str, token_range: (usize, usize)) -> String {
    let token = sql.get(token_range.0..token_range.1).unwrap_or_default();
    let prefix = sql.get(..token_range.0).unwrap_or_default();
    let qualifier = prefix
        .strip_suffix('.')
        .and_then(|before_dot| sql_identifier_before(before_dot, before_dot.len()));
    qualifier.map_or_else(|| token.to_owned(), |qualifier| format!("{qualifier}.{token}"))
}

fn is_identifier_character(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '_' | '.' | '$' | '"')
}

fn resolve_unambiguous(candidates: Option<&Vec<SqlSymbolHelp>>, active_schema: &str) -> Option<SqlSymbolHelp> {
    let candidates = candidates?;
    if candidates.len() == 1 {
        return candidates.first().cloned();
    }
    let schema_prefix = format!("{active_schema}.");
    let active_candidates: Vec<_> = candidates
        .iter()
        .filter(|candidate| candidate.title.starts_with(&schema_prefix))
        .collect();
    (active_candidates.len() == 1).then(|| active_candidates[0].clone())
}

fn function_signature(function: &UiFunctionSummary) -> SqlSignatureHelp {
    let parameters: Vec<String> = function
        .parameters
        .iter()
        .filter(|parameter| {
            parameter.mode.is_empty()
                || matches!(
                    parameter.mode.to_ascii_uppercase().as_str(),
                    "IN" | "INOUT" | "VARIADIC"
                )
        })
        .map(|parameter| {
            let name = if parameter.name.is_empty() {
                parameter.data_type.clone()
            } else {
                format!("{} {}", parameter.name, parameter.data_type)
            };
            if parameter.has_default && !parameter.default_expr.is_empty() {
                format!("{name} = {}", parameter.default_expr)
            } else {
                name
            }
        })
        .collect();
    let label = format!(
        "{}.{}({}) → {}",
        function.schema,
        function.name,
        parameters.join(", "),
        function.data_type
    );
    SqlSignatureHelp {
        label,
        parameters,
        active_parameter: 0,
        documentation: format!(
            "{} · language {} · volatility {}{}",
            function.routine_type,
            function.language,
            function.volatility,
            if function.security_definer {
                " · security definer"
            } else {
                ""
            }
        ),
    }
}

fn signature_symbol_help(signature: &SqlSignatureHelp) -> SqlSymbolHelp {
    SqlSymbolHelp {
        title: signature.label.clone(),
        kind: "Function".to_owned(),
        detail: signature.documentation.clone(),
        documentation: "SQL function signature".to_owned(),
    }
}

fn builtin_signature(name: &str, dialect: SqlDialect) -> Option<SqlSignatureHelp> {
    let normalized = normalize(name.rsplit('.').next().unwrap_or(name));
    let (parameters, documentation) = match (dialect, normalized.as_str()) {
        (_, "count") => (vec!["expression"], "Count input rows or non-null values"),
        (_, "sum") => (vec!["expression"], "Sum numeric input values"),
        (_, "avg") => (vec!["expression"], "Average numeric input values"),
        (_, "lower") => (vec!["text"], "Convert text to lower case"),
        (_, "upper") => (vec!["text"], "Convert text to upper case"),
        (_, "length") => (vec!["value"], "Return the length of a value"),
        (_, "coalesce") => (vec!["value", "fallback", "…"], "Return the first non-null argument"),
        (SqlDialect::SQLite, "substr") => (vec!["text", "start", "length"], "Extract part of a string"),
        (SqlDialect::SQLite, "group_concat") => (vec!["value", "separator"], "Concatenate grouped values"),
        (SqlDialect::Postgres, "date_trunc") => (vec!["precision", "source"], "Truncate a temporal value"),
        (SqlDialect::Postgres, "string_agg") => (vec!["value", "delimiter"], "Concatenate grouped text"),
        _ => return None,
    };
    let parameters: Vec<String> = parameters.into_iter().map(str::to_owned).collect();
    Some(SqlSignatureHelp {
        label: format!("{}({})", name.rsplit('.').next().unwrap_or(name), parameters.join(", ")),
        parameters,
        active_parameter: 0,
        documentation: documentation.to_owned(),
    })
}

fn keyword_help(token: &str, dialect: SqlDialect) -> Option<SqlSymbolHelp> {
    let keyword = token.to_ascii_uppercase();
    let documentation = match keyword.as_str() {
        "SELECT" => "Read rows or computed expressions from a query source.",
        "FROM" => "Declare the table, view, CTE, or subquery used by the statement.",
        "WHERE" => "Filter rows before grouping and projection.",
        "JOIN" => "Combine rows from another relation using an ON or USING condition.",
        "GROUP" | "GROUP BY" => "Group rows before aggregate expressions are evaluated.",
        "HAVING" => "Filter grouped rows after aggregate evaluation.",
        "ORDER" | "ORDER BY" => "Sort the result set by one or more expressions.",
        "LIMIT" => "Limit the maximum number of returned rows.",
        "RETURNING" if dialect == SqlDialect::Postgres => "Return values from rows changed by a mutation.",
        "PRAGMA" if dialect == SqlDialect::SQLite => "Read or change a SQLite engine setting.",
        _ => return None,
    };
    Some(SqlSymbolHelp {
        title: keyword,
        kind: "SQL keyword".to_owned(),
        detail: match dialect {
            SqlDialect::Postgres => "PostgreSQL syntax",
            SqlDialect::SQLite => "SQLite syntax",
            SqlDialect::Generic => "SQL syntax",
        }
        .to_owned(),
        documentation: documentation.to_owned(),
    })
}

fn summarize_definition(definition: &str) -> String {
    const MAX_PREVIEW_CHARS: usize = 180;
    let compact = definition.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= MAX_PREVIEW_CHARS {
        compact
    } else {
        format!("{}…", compact.chars().take(MAX_PREVIEW_CHARS).collect::<String>())
    }
}

fn normalize(value: &str) -> String {
    value.trim_matches('"').to_ascii_lowercase()
}

/// Build `RichHoverHelp::Table` from a `UiTableSummary`.
fn rich_table_hover(table: &UiTableSummary, kind: &str, schema: &UiSchemaSummary) -> RichHoverHelp {
    // Collect columns and mark FK columns.
    let fk_columns: std::collections::HashSet<&str> = table
        .foreign_keys
        .iter()
        .flat_map(|fk: &UiSchemaForeignKey| fk.from_columns.iter().map(String::as_str))
        .collect();

    let columns: Vec<HoverColumn> = table
        .columns
        .iter()
        .map(|col: &UiSchemaColumn| HoverColumn {
            name: col.name.clone(),
            data_type: col.data_type.clone(),
            nullable: col.nullable,
            is_primary_key: col.is_primary_key,
            is_foreign_key: fk_columns.contains(col.name.as_str()),
        })
        .collect();

    let foreign_keys: Vec<HoverForeignKey> = table
        .foreign_keys
        .iter()
        .map(|fk: &UiSchemaForeignKey| HoverForeignKey {
            name: fk.name.clone(),
            from_columns: fk.from_columns.clone(),
            to_table: if fk.to_schema.is_empty() || fk.to_schema == table.schema {
                fk.to_table.clone()
            } else {
                format!("{}.{}", fk.to_schema, fk.to_table)
            },
            to_columns: fk.to_columns.clone(),
        })
        .collect();

    let _ = schema; // Reserved for future enrichment (index count, etc.)

    RichHoverHelp::Table {
        qualified_name: format!("{}.{}", table.schema, table.name),
        kind: kind.to_owned(),
        row_count: table.row_count,
        columns,
        foreign_keys,
    }
}

/// Try to resolve a column hover from a possibly-qualified name like `table.col` or
/// `schema.table.col`.
fn rich_column_hover(qualified: &str, active_schema: &str, schema: &UiSchemaSummary) -> Option<RichHoverHelp> {
    // We only handle dot-qualified names (bare column names are ambiguous).
    let parts: Vec<&str> = qualified.split('.').collect();
    let (table_name, col_name) = match parts.as_slice() {
        [t, c] => (*t, *c),
        [_s, t, c] => (*t, *c),
        _ => return None,
    };

    for table in &schema.table_details {
        if !table.name.eq_ignore_ascii_case(table_name)
            && !format!("{}.{}", table.schema, table.name).eq_ignore_ascii_case(qualified)
        {
            // Allow schema prefix to also match
            if parts.len() < 3 && !table.schema.eq_ignore_ascii_case(active_schema) {
                continue;
            }
        }
        for col in &table.columns {
            if col.name.eq_ignore_ascii_case(col_name) {
                // Find any FK pointing from this column.
                let fk_target = table.foreign_keys.iter().find_map(|fk| {
                    if fk.from_columns.iter().any(|c| c.eq_ignore_ascii_case(col_name)) {
                        Some(format!(
                            "{}.{}({})",
                            if fk.to_schema.is_empty() {
                                &table.schema
                            } else {
                                &fk.to_schema
                            },
                            fk.to_table,
                            fk.to_columns.join(", ")
                        ))
                    } else {
                        None
                    }
                });
                return Some(RichHoverHelp::Column {
                    qualified_name: format!("{}.{}.{}", table.schema, table.name, col.name),
                    data_type: col.data_type.clone(),
                    nullable: col.nullable,
                    is_primary_key: col.is_primary_key,
                    parent_table: format!("{}.{}", table.schema, table.name),
                    foreign_key_target: fk_target,
                });
            }
        }
    }
    None
}

/// Expanded keyword help table (~50 keywords) returning `RichHoverHelp::Keyword`.
#[allow(clippy::too_many_lines)]
fn rich_keyword_help(token: &str, dialect: SqlDialect) -> Option<RichHoverHelp> {
    let kw = token.to_ascii_uppercase();
    let is_pg = dialect == SqlDialect::Postgres;
    let is_sqlite = dialect == SqlDialect::SQLite;
    let dialect_note = match dialect {
        SqlDialect::Postgres => "PostgreSQL",
        SqlDialect::SQLite => "SQLite",
        SqlDialect::Generic => "SQL",
    }
    .to_owned();

    let (doc, example): (&str, Option<&str>) = match kw.as_str() {
        // ── DML ──────────────────────────────────────────────────────────────
        "SELECT" => (
            "Read rows or computed expressions from one or more relations.",
            Some("SELECT id, name FROM users WHERE active = true;"),
        ),
        "FROM" => (
            "Specify the primary relation (table, view, subquery, CTE) for the statement.",
            Some("SELECT * FROM orders o JOIN customers c ON o.customer_id = c.id;"),
        ),
        "WHERE" => (
            "Filter rows based on a boolean predicate before grouping and projection.",
            Some("SELECT * FROM orders WHERE status = 'pending' AND total > 100;"),
        ),
        "INSERT" => (
            "Insert one or more rows into a table.",
            Some("INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com');"),
        ),
        "INTO" => (
            "Specify the target table for INSERT or SELECT INTO.",
            None,
        ),
        "UPDATE" => (
            "Modify existing rows in a table.",
            Some("UPDATE products SET price = price * 1.1 WHERE category = 'premium';"),
        ),
        "SET" => (
            "Assign new values to columns in an UPDATE statement.",
            None,
        ),
        "DELETE" => (
            "Remove rows from a table.",
            Some("DELETE FROM sessions WHERE expired_at < now();"),
        ),
        "MERGE" if is_pg => (
            "Conditionally insert, update, or delete rows based on a join condition (PostgreSQL 15+).",
            None,
        ),
        // ── Query clauses ─────────────────────────────────────────────────────
        "JOIN" => (
            "Combine rows from another relation. INNER JOIN (default) returns only matched rows.",
            Some("SELECT o.id, c.name FROM orders o JOIN customers c ON o.customer_id = c.id;"),
        ),
        "INNER" => ("Qualifier for JOIN: return only rows matched in both relations.", None),
        "LEFT" => (
            "LEFT JOIN: return all rows from the left relation; NULL-fill unmatched right rows.",
            Some("SELECT u.id, p.bio FROM users u LEFT JOIN profiles p ON u.id = p.user_id;"),
        ),
        "RIGHT" => (
            "RIGHT JOIN: return all rows from the right relation; NULL-fill unmatched left rows.",
            None,
        ),
        "FULL" => (
            "FULL OUTER JOIN: return all rows from both relations; NULL-fill where unmatched.",
            None,
        ),
        "CROSS" => (
            "CROSS JOIN: cartesian product of both relations (every row × every row).",
            None,
        ),
        "ON" => ("Specify the join predicate.", None),
        "USING" => (
            "Specify shared column names for a JOIN condition (implicit equality).",
            Some("SELECT * FROM orders JOIN customers USING (customer_id);"),
        ),
        "GROUP" | "GROUP BY" => (
            "Group rows with the same values in the listed expressions before applying aggregates.",
            Some("SELECT status, count(*) FROM orders GROUP BY status;"),
        ),
        "HAVING" => (
            "Filter grouped rows after aggregate evaluation — analogous to WHERE for groups.",
            Some("SELECT customer_id, sum(total) FROM orders GROUP BY customer_id HAVING sum(total) > 500;"),
        ),
        "ORDER" | "ORDER BY" => (
            "Sort the result set by one or more expressions. Default direction is ASC.",
            Some("SELECT * FROM products ORDER BY price DESC, name ASC;"),
        ),
        "ASC" => ("Sort in ascending order (smallest first). This is the default.", None),
        "DESC" => ("Sort in descending order (largest first).", None),
        "NULLS" => ("NULLS FIRST / NULLS LAST — control where NULL values appear in the sort.", None),
        "LIMIT" => (
            "Restrict the number of rows returned.",
            Some("SELECT * FROM events ORDER BY created_at DESC LIMIT 20;"),
        ),
        "OFFSET" => (
            "Skip a specified number of rows before returning results. Combine with LIMIT for pagination.",
            Some("SELECT * FROM products ORDER BY id LIMIT 10 OFFSET 20;"),
        ),
        "DISTINCT" => (
            "Eliminate duplicate rows from the result set.",
            Some("SELECT DISTINCT country FROM customers;"),
        ),
        "ALL" => ("Include duplicate rows (the default for UNION ALL, or explicit in SELECT ALL).", None),
        "UNION" => (
            "Combine result sets of two queries; duplicate rows are removed unless ALL is specified.",
            Some("SELECT id FROM active_users UNION SELECT id FROM trial_users;"),
        ),
        "INTERSECT" => (
            "Return only rows that appear in both result sets.",
            None,
        ),
        "EXCEPT" if is_pg => (
            "Return rows from the first result set that do not appear in the second.",
            None,
        ),
        "MINUS" if is_sqlite => (
            "Return rows from the first result set that do not appear in the second (EXCEPT synonym in some dialects).",
            None,
        ),
        // ── Subquery / CTE ───────────────────────────────────────────────────
        "WITH" => (
            "Introduce a Common Table Expression (CTE). Chain multiple CTEs with commas.",
            Some("WITH active AS (SELECT * FROM users WHERE active) SELECT * FROM active;"),
        ),
        "RECURSIVE" => (
            "Allow a CTE to reference itself, enabling tree or graph traversal.",
            Some("WITH RECURSIVE tree AS (SELECT id, parent_id FROM categories WHERE parent_id IS NULL\nUNION ALL\nSELECT c.id, c.parent_id FROM categories c JOIN tree t ON c.parent_id = t.id)\nSELECT * FROM tree;"),
        ),
        "EXISTS" => (
            "Test whether a subquery returns at least one row.",
            Some("SELECT * FROM orders o WHERE EXISTS (SELECT 1 FROM items WHERE order_id = o.id);"),
        ),
        "IN" => (
            "Test whether a value is equal to any member of a list or subquery result.",
            Some("SELECT * FROM orders WHERE status IN ('pending', 'processing');"),
        ),
        "NOT" => ("Negate the following boolean expression.", None),
        "ANY" | "SOME" => (
            "Return true if the comparison holds for at least one value in the subquery/array.",
            None,
        ),
        // ── Predicates ───────────────────────────────────────────────────────
        "LIKE" => (
            "Pattern match using % (any sequence) and _ (any single character).",
            Some("SELECT * FROM products WHERE name LIKE 'Pro%';"),
        ),
        "ILIKE" if is_pg => (
            "Case-insensitive LIKE (PostgreSQL extension).",
            Some("SELECT * FROM users WHERE email ILIKE '%@example.com';"),
        ),
        "GLOB" if is_sqlite => (
            "Case-sensitive glob pattern matching (* and ?) — SQLite extension.",
            Some("SELECT * FROM files WHERE name GLOB '*.sql';"),
        ),
        "BETWEEN" => (
            "Test whether a value falls within a range (inclusive of both bounds).",
            Some("SELECT * FROM orders WHERE total BETWEEN 50 AND 200;"),
        ),
        "IS" => ("Test for NULL equality using IS NULL / IS NOT NULL.", None),
        "NULL" => ("The absence of a value. Use IS NULL / IS NOT NULL to test.", None),
        "TRUE" | "FALSE" => ("Boolean literal.", None),
        // ── DDL ──────────────────────────────────────────────────────────────
        "CREATE" => ("Create a database object (table, index, view, function, etc.).", None),
        "ALTER" => ("Modify the structure of an existing database object.", None),
        "DROP" => (
            "Permanently remove a database object. Use IF EXISTS to avoid errors when the object is absent.",
            Some("DROP TABLE IF EXISTS temp_results;"),
        ),
        "TABLE" => ("Qualify a DDL statement target as a table.", None),
        "INDEX" => ("Create or reference a database index.", None),
        "VIEW" => ("Create or reference a database view.", None),
        "CONSTRAINT" => ("Name or reference a table constraint (PK, FK, UNIQUE, CHECK).", None),
        "PRIMARY" | "PRIMARY KEY" => ("Designate a column or column group as the unique row identifier.", None),
        "FOREIGN" | "FOREIGN KEY" => ("Declare a referential integrity constraint to another table.", None),
        "REFERENCES" => ("Specify the referenced table and column(s) for a foreign key.", None),
        "UNIQUE" => ("Enforce that all values in the column(s) are distinct.", None),
        "CHECK" => ("Validate a boolean expression at the row level.", None),
        "DEFAULT" => ("Supply a value when none is given during INSERT.", None),
        "NOT NULL" => ("Prevent NULL values in the column.", None),
        // ── Transaction ──────────────────────────────────────────────────────
        "BEGIN" => (
            "Start a transaction block. Statements after BEGIN are executed atomically.",
            None,
        ),
        "COMMIT" => ("Persist all changes made within the current transaction.", None),
        "ROLLBACK" => ("Discard all changes made within the current transaction.", None),
        "SAVEPOINT" => ("Mark a point within a transaction to which you can roll back partially.", None),
        // ── Provider-specific ─────────────────────────────────────────────────
        "RETURNING" if is_pg => (
            "Return column values from rows affected by INSERT, UPDATE, or DELETE.",
            Some("INSERT INTO users (name) VALUES ('Bob') RETURNING id, created_at;"),
        ),
        "EXPLAIN" => (
            "Show the query execution plan without running the query.\nEXPLAIN (ANALYZE, BUFFERS) also executes and shows runtime statistics.",
            Some("EXPLAIN (ANALYZE, BUFFERS) SELECT * FROM orders WHERE customer_id = 42;"),
        ),
        "ANALYZE" => ("In EXPLAIN ANALYZE: actually execute the query to collect runtime statistics.", None),
        "VACUUM" if is_pg => ("Reclaim storage occupied by dead tuples. VACUUM ANALYZE also updates statistics.", None),
        "PRAGMA" if is_sqlite => (
            "Read or modify a SQLite engine setting.",
            Some("PRAGMA table_info(users);\nPRAGMA journal_mode = WAL;"),
        ),
        "ATTACH" if is_sqlite => (
            "Attach another SQLite database file to the current connection.",
            Some("ATTACH DATABASE 'archive.db' AS archive;"),
        ),
        "WINDOW" if is_pg => (
            "Define a named window specification for window functions.",
            Some("SELECT row_number() OVER w FROM orders WINDOW w AS (PARTITION BY customer_id ORDER BY created_at);"),
        ),
        "OVER" => (
            "Apply a window function over a partition of rows without collapsing them into groups.",
            Some("SELECT name, salary, rank() OVER (PARTITION BY dept ORDER BY salary DESC) FROM employees;"),
        ),
        "PARTITION" => ("Define partitions within a window function OVER clause.", None),
        "CASE" => (
            "Conditional expression: CASE WHEN … THEN … ELSE … END.",
            Some("SELECT CASE WHEN score >= 90 THEN 'A' WHEN score >= 80 THEN 'B' ELSE 'C' END FROM results;"),
        ),
        "WHEN" => ("Branch condition inside a CASE expression.", None),
        "THEN" => ("Result expression for a matching WHEN condition.", None),
        "ELSE" => ("Default result for a CASE expression when no WHEN matches.", None),
        "END" => ("Close a CASE expression or a BEGIN block.", None),
        "CAST" => (
            "Convert a value to a different data type.",
            Some("SELECT CAST(price AS numeric(10,2)) FROM products;"),
        ),
        "COALESCE" => (
            "Return the first non-NULL argument. Equivalent to a series of CASE WHEN IS NOT NULL.",
            Some("SELECT COALESCE(display_name, username, 'anonymous') FROM users;"),
        ),
        "NULLIF" => (
            "Return NULL if the two arguments are equal, otherwise return the first argument.",
            Some("SELECT NULLIF(discount, 0) FROM orders;"),
        ),
        _ => return None,
    };

    Some(RichHoverHelp::Keyword {
        keyword: kw,
        dialect_note,
        documentation: doc.to_owned(),
        example: example.map(str::to_owned),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::TextBuffer;
    use crate::runtime::{UiRoutineParameter, UiSchemaColumn, UiTableSummary};

    fn tokens(sql: &str, dialect: SqlDialect) -> CachedSqlTokens {
        let mut tokens = CachedSqlTokens::new();
        tokens.get_or_recompute(&TextBuffer::from_string(sql), dialect);
        tokens
    }

    #[test]
    fn index_resolves_active_schema_and_rejects_ambiguous_relations() {
        let schema = UiSchemaSummary {
            table_details: vec![
                UiTableSummary {
                    schema: "public".to_owned(),
                    name: "users".to_owned(),
                    row_count: Some(4),
                    columns: vec![],
                    foreign_keys: vec![],
                },
                UiTableSummary {
                    schema: "audit".to_owned(),
                    name: "users".to_owned(),
                    row_count: None,
                    columns: vec![],
                    foreign_keys: vec![],
                },
            ],
            ..Default::default()
        };
        let index = SchemaSymbolIndex::build(&schema);
        assert_eq!(
            index
                .resolve_symbol("users", (0, 5), "public", SqlDialect::Postgres)
                .unwrap()
                .title,
            "public.users"
        );
        assert!(index
            .resolve_symbol("users", (0, 5), "", SqlDialect::Postgres)
            .is_none());
    }

    #[test]
    fn qualified_column_hover_includes_type_and_constraints() {
        let schema = UiSchemaSummary {
            table_details: vec![UiTableSummary {
                schema: "public".to_owned(),
                name: "users".to_owned(),
                row_count: None,
                columns: vec![UiSchemaColumn {
                    name: "id".to_owned(),
                    data_type: "bigint".to_owned(),
                    nullable: false,
                    is_primary_key: true,
                }],
                foreign_keys: vec![],
            }],
            ..Default::default()
        };
        let index = SchemaSymbolIndex::build(&schema);
        let sql = "SELECT users.id";
        let help = index
            .resolve_symbol(sql, (13, 15), "public", SqlDialect::Postgres)
            .unwrap();
        assert_eq!(help.title, "public.users.id");
        assert!(help.detail.contains("primary key"));
    }

    #[test]
    fn nested_function_call_tracks_only_same_level_commas() {
        let sql = "SELECT coalesce(round(total, 2), fallback, 'a,b'";
        let cached = tokens(sql, SqlDialect::Postgres);
        let call = function_call_context(sql, sql.len(), &cached).unwrap();
        assert_eq!(call.name, "coalesce");
        assert_eq!(call.active_argument, 2);
    }

    #[test]
    fn signature_help_prefers_introspected_active_schema_function() {
        let schema = UiSchemaSummary {
            functions: vec![UiFunctionSummary {
                schema: "public".to_owned(),
                name: "calculate_tax".to_owned(),
                routine_type: "FUNCTION".to_owned(),
                data_type: "numeric".to_owned(),
                definition: String::new(),
                identity_arguments: "amount numeric".to_owned(),
                language: "sql".to_owned(),
                volatility: "stable".to_owned(),
                security_definer: false,
                parameters: vec![UiRoutineParameter {
                    name: "amount".to_owned(),
                    data_type: "numeric".to_owned(),
                    mode: "IN".to_owned(),
                    has_default: false,
                    default_expr: String::new(),
                }],
            }],
            ..Default::default()
        };
        let index = SchemaSymbolIndex::build(&schema);
        let sql = "SELECT calculate_tax(";
        let cached = tokens(sql, SqlDialect::Postgres);
        let signature = index
            .signature_help(sql, sql.len(), "public", SqlDialect::Postgres, &cached)
            .unwrap();
        assert!(signature.label.starts_with("public.calculate_tax"));
        assert_eq!(signature.active_parameter, 0);
    }

    #[test]
    fn sqlite_and_postgres_builtins_remain_provider_specific() {
        let index = SchemaSymbolIndex::default();
        let sqlite_sql = "SELECT substr(value, ";
        let sqlite_tokens = tokens(sqlite_sql, SqlDialect::SQLite);
        assert!(index
            .signature_help(sqlite_sql, sqlite_sql.len(), "main", SqlDialect::SQLite, &sqlite_tokens,)
            .is_some());

        let postgres_tokens = tokens(sqlite_sql, SqlDialect::Postgres);
        assert!(index
            .signature_help(
                sqlite_sql,
                sqlite_sql.len(),
                "public",
                SqlDialect::Postgres,
                &postgres_tokens,
            )
            .is_none());
    }

    #[test]
    fn rich_hover_resolves_table_columns_and_foreign_keys() {
        let schema = UiSchemaSummary {
            table_details: vec![UiTableSummary {
                schema: "public".to_owned(),
                name: "orders".to_owned(),
                row_count: Some(150),
                columns: vec![
                    UiSchemaColumn {
                        name: "id".to_owned(),
                        data_type: "bigint".to_owned(),
                        nullable: false,
                        is_primary_key: true,
                    },
                    UiSchemaColumn {
                        name: "user_id".to_owned(),
                        data_type: "bigint".to_owned(),
                        nullable: false,
                        is_primary_key: false,
                    },
                ],
                foreign_keys: vec![UiSchemaForeignKey {
                    name: "fk_orders_user".to_owned(),
                    from_columns: vec!["user_id".to_owned()],
                    to_schema: "public".to_owned(),
                    to_table: "users".to_owned(),
                    to_columns: vec!["id".to_owned()],
                }],
            }],
            ..Default::default()
        };
        let index = SchemaSymbolIndex::build(&schema);
        let sql = "SELECT * FROM orders";
        let hover = index
            .rich_hover(sql, (14, 20), "public", SqlDialect::Postgres, &schema)
            .expect("should resolve orders table hover");

        match hover {
            RichHoverHelp::Table {
                qualified_name,
                row_count,
                columns,
                foreign_keys,
                ..
            } => {
                assert_eq!(qualified_name, "public.orders");
                assert_eq!(row_count, Some(150));
                assert_eq!(columns.len(), 2);
                assert!(columns[0].is_primary_key);
                assert!(columns[1].is_foreign_key);
                assert_eq!(foreign_keys.len(), 1);
                assert_eq!(foreign_keys[0].to_table, "users");
            }
            other => panic!("expected RichHoverHelp::Table, got {other:?}"),
        }
    }

    #[test]
    fn rich_hover_resolves_qualified_column_with_fk_target() {
        let schema = UiSchemaSummary {
            table_details: vec![UiTableSummary {
                schema: "public".to_owned(),
                name: "orders".to_owned(),
                row_count: None,
                columns: vec![UiSchemaColumn {
                    name: "user_id".to_owned(),
                    data_type: "bigint".to_owned(),
                    nullable: false,
                    is_primary_key: false,
                }],
                foreign_keys: vec![UiSchemaForeignKey {
                    name: "fk_orders_user".to_owned(),
                    from_columns: vec!["user_id".to_owned()],
                    to_schema: "public".to_owned(),
                    to_table: "users".to_owned(),
                    to_columns: vec!["id".to_owned()],
                }],
            }],
            ..Default::default()
        };
        let index = SchemaSymbolIndex::build(&schema);
        let sql = "SELECT orders.user_id FROM orders";
        let hover = index
            .rich_hover(sql, (7, 21), "public", SqlDialect::Postgres, &schema)
            .expect("should resolve column hover");

        match hover {
            RichHoverHelp::Column {
                qualified_name,
                data_type,
                foreign_key_target,
                ..
            } => {
                assert_eq!(qualified_name, "public.orders.user_id");
                assert_eq!(data_type, "bigint");
                assert_eq!(foreign_key_target, Some("public.users(id)".to_owned()));
            }
            other => panic!("expected RichHoverHelp::Column, got {other:?}"),
        }
    }

    #[test]
    fn rich_hover_keyword_returns_documentation_and_dialect_note() {
        let index = SchemaSymbolIndex::default();
        let schema = UiSchemaSummary::default();
        let sql = "SELECT * FROM orders RETURNING id";
        let hover = index
            .rich_hover(sql, (0, 6), "public", SqlDialect::Postgres, &schema)
            .expect("should resolve SELECT keyword");

        match hover {
            RichHoverHelp::Keyword {
                keyword,
                dialect_note,
                documentation,
                example,
            } => {
                assert_eq!(keyword, "SELECT");
                assert_eq!(dialect_note, "PostgreSQL");
                assert!(!documentation.is_empty());
                assert!(example.is_some());
            }
            other => panic!("expected RichHoverHelp::Keyword, got {other:?}"),
        }
    }
}
