//! Statement slice, clause, and table references for SQL completion.
//!
//! Keywords count only at the parenthesis depth that encloses the cursor.
//! `(` increments depth and `)` decrements it. Quoted text and comments are
//! not keywords and do not change depth. Table names come from FROM / JOIN /
//! UPDATE / INTO tokens in the statement around the cursor, not from a
//! substring search of the script.

use super::schema_completion::SqlClause;
use crate::runtime::UiSchemaSummary;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
struct Tok<'a> {
    kind: TokKind,
    text: &'a str,
    /// Parenthesis depth before this token.
    depth: i32,
    quoted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokKind {
    Word,
    Comma,
    Dot,
    Open,
    Close,
}

#[derive(Debug, Clone)]
struct TableUse {
    name: String,
    alias: Option<String>,
}

#[derive(Debug)]
pub struct CompletionScope {
    pub clause: SqlClause,
    pub aliases: HashMap<String, String>,
    pub referenced_tables: Vec<String>,
    pub statement_start: usize,
    pub statement_end: usize,
    tables: Vec<TableUse>,
}

#[derive(Debug)]
pub struct FkSnippet {
    pub label: String,
    pub insert_text: String,
    pub detail: String,
    pub documentation: String,
}

pub fn analyze(full_text: &str, cursor: usize, prefix_len: usize) -> CompletionScope {
    let cursor = floor_char_boundary(full_text, cursor);
    let (statement_start, statement_end) = statement_bounds(full_text, cursor);
    let cursor = cursor.clamp(statement_start, statement_end);
    let clause_end =
        floor_char_boundary(full_text, cursor.saturating_sub(prefix_len)).clamp(statement_start, statement_end);
    let cursor_depth = paren_depth(full_text, statement_start, cursor);
    let clause = clause_at(full_text, statement_start, clause_end, cursor_depth);
    let table_tokens = tokenize(full_text, statement_start, statement_end);
    let tables = collect_tables(&table_tokens, cursor_depth);

    let mut aliases = HashMap::new();
    let mut referenced_tables = Vec::new();
    for table in &tables {
        if let Some(alias) = &table.alias {
            aliases.entry(alias.clone()).or_insert_with(|| table.name.clone());
        }
        if !referenced_tables
            .iter()
            .any(|name: &String| name.eq_ignore_ascii_case(&table.name))
        {
            referenced_tables.push(table.name.clone());
        }
    }

    CompletionScope {
        clause,
        aliases,
        referenced_tables,
        statement_start,
        statement_end,
        tables,
    }
}

impl CompletionScope {
    /// FK predicates between tables already named in this statement.
    /// `prepend_on` is set when the user has not typed `ON` yet.
    /// A side with no alias is qualified by its table name.
    pub fn foreign_key_join_snippets(&self, summary: &UiSchemaSummary, prepend_on: bool) -> Vec<FkSnippet> {
        let mut snippets = Vec::new();
        let mut seen = HashSet::new();
        for source in &summary.table_details {
            if !self
                .tables
                .iter()
                .any(|table| table.name.eq_ignore_ascii_case(&source.name))
            {
                continue;
            }
            let source_qual = self.qualifier(&source.name);
            for foreign_key in &source.foreign_keys {
                if !self
                    .tables
                    .iter()
                    .any(|table| table.name.eq_ignore_ascii_case(&foreign_key.to_table))
                {
                    continue;
                }
                let target_qual = self.qualifier(&foreign_key.to_table);
                if foreign_key.from_columns.len() != foreign_key.to_columns.len()
                    || foreign_key.from_columns.is_empty()
                    || source_qual.eq_ignore_ascii_case(&target_qual)
                {
                    continue;
                }
                let condition = foreign_key
                    .from_columns
                    .iter()
                    .zip(&foreign_key.to_columns)
                    .map(|(source_column, target_column)| {
                        format!("{source_qual}.{source_column} = {target_qual}.{target_column}")
                    })
                    .collect::<Vec<_>>()
                    .join(" AND ");
                let insert_text = if prepend_on {
                    format!("ON {condition}")
                } else {
                    condition
                };
                if !seen.insert(insert_text.to_lowercase()) {
                    continue;
                }
                snippets.push(FkSnippet {
                    label: insert_text.clone(),
                    insert_text,
                    detail: format!("FK · {}", foreign_key.name),
                    documentation: format!(
                        "Join {} to {} using the declared foreign key",
                        source.name, foreign_key.to_table
                    ),
                });
            }
        }
        snippets
    }

    /// Other side of a foreign key that is not already in this statement.
    /// The insert text names that table and the `ON` predicate.
    pub fn foreign_key_join_targets(&self, summary: &UiSchemaSummary) -> Vec<FkSnippet> {
        let mut snippets = Vec::new();
        let mut seen = HashSet::new();
        let present = |name: &str| self.tables.iter().any(|table| table.name.eq_ignore_ascii_case(name));
        for source in &summary.table_details {
            let source_here = present(&source.name);
            for foreign_key in &source.foreign_keys {
                let target_here = present(&foreign_key.to_table);
                let (here_name, other_name, here_cols, other_cols) = if source_here && !target_here {
                    (
                        source.name.as_str(),
                        foreign_key.to_table.as_str(),
                        &foreign_key.from_columns,
                        &foreign_key.to_columns,
                    )
                } else if target_here && !source_here {
                    (
                        foreign_key.to_table.as_str(),
                        source.name.as_str(),
                        &foreign_key.to_columns,
                        &foreign_key.from_columns,
                    )
                } else {
                    continue;
                };
                if here_cols.len() != other_cols.len() || here_cols.is_empty() {
                    continue;
                }
                let here_qual = self.qualifier(here_name);
                let condition = here_cols
                    .iter()
                    .zip(other_cols)
                    .map(|(here_column, other_column)| {
                        format!("{here_qual}.{here_column} = {other_name}.{other_column}")
                    })
                    .collect::<Vec<_>>()
                    .join(" AND ");
                let insert_text = format!("{other_name} ON {condition}");
                if !seen.insert(insert_text.to_lowercase()) {
                    continue;
                }
                snippets.push(FkSnippet {
                    label: other_name.to_owned(),
                    insert_text,
                    detail: format!("FK · {}", foreign_key.name),
                    documentation: format!("Join {here_name} to {other_name} using the declared foreign key"),
                });
            }
        }
        snippets
    }

    fn qualifier(&self, table_name: &str) -> String {
        self.tables
            .iter()
            .find(|table| table.name.eq_ignore_ascii_case(table_name))
            .and_then(|table| table.alias.clone())
            .unwrap_or_else(|| table_name.to_lowercase())
    }
}

fn clause_at(text: &str, start: usize, end: usize, cursor_depth: i32) -> SqlClause {
    let mut best_depth = i32::MIN;
    let mut best = SqlClause::Unknown;
    for token in tokenize(text, start, end) {
        if token.kind != TokKind::Word || token.quoted {
            continue;
        }
        let Some(clause) = keyword_clause(token.text) else {
            continue;
        };
        // Inner scopes win over an outer keyword when the cursor is inside them.
        // A parent keyword still applies when this paren group has none (INSERT lists).
        if token.depth <= cursor_depth && token.depth >= best_depth {
            best_depth = token.depth;
            best = clause;
        }
    }
    best
}

fn keyword_clause(word: &str) -> Option<SqlClause> {
    Some(match word {
        word if word.eq_ignore_ascii_case("FROM") => SqlClause::From,
        word if word.eq_ignore_ascii_case("JOIN")
            || word.eq_ignore_ascii_case("CROSS")
            || word.eq_ignore_ascii_case("INNER")
            || word.eq_ignore_ascii_case("LEFT")
            || word.eq_ignore_ascii_case("RIGHT")
            || word.eq_ignore_ascii_case("OUTER") =>
        {
            SqlClause::Join
        }
        word if word.eq_ignore_ascii_case("ON") => SqlClause::JoinOn,
        word if word.eq_ignore_ascii_case("WHERE") => SqlClause::Where,
        word if word.eq_ignore_ascii_case("SET") => SqlClause::Set,
        word if word.eq_ignore_ascii_case("UPDATE") => SqlClause::Update,
        word if word.eq_ignore_ascii_case("VALUES") => SqlClause::Values,
        word if word.eq_ignore_ascii_case("INTO") => SqlClause::InsertInto,
        word if word.eq_ignore_ascii_case("DELETE") => SqlClause::DeleteFrom,
        word if word.eq_ignore_ascii_case("HAVING") => SqlClause::Having,
        word if word.eq_ignore_ascii_case("ORDER") => SqlClause::OrderBy,
        word if word.eq_ignore_ascii_case("GROUP") => SqlClause::GroupBy,
        word if word.eq_ignore_ascii_case("RETURNING") => SqlClause::Returning,
        word if word.eq_ignore_ascii_case("SELECT") => SqlClause::Select,
        _ => return None,
    })
}

fn collect_tables(tokens: &[Tok<'_>], cursor_depth: i32) -> Vec<TableUse> {
    let mut tables = Vec::new();
    walk_tables(tokens, 0, tokens.len(), cursor_depth, &mut tables);
    tables
}

fn walk_tables(tokens: &[Tok<'_>], start: usize, end: usize, cursor_depth: i32, tables: &mut Vec<TableUse>) {
    let mut index = start;
    while index < end {
        let token = &tokens[index];
        if token.kind == TokKind::Open {
            if let Some(close_at) = matching_close(tokens, index) {
                walk_tables(tokens, index + 1, close_at, cursor_depth, tables);
                index = close_at + 1;
            } else {
                walk_tables(tokens, index + 1, tokens.len(), cursor_depth, tables);
                break;
            }
            continue;
        }
        if token.kind == TokKind::Word && is_table_introducer(token.text) && token.depth <= cursor_depth {
            let commas = token.text.eq_ignore_ascii_case("FROM");
            let depth = token.depth;
            let next = take_tables(tokens, index + 1, end, depth, commas, cursor_depth, tables);
            index = if next > index { next } else { index + 1 };
            continue;
        }
        index += 1;
    }
}

fn take_tables(
    tokens: &[Tok<'_>],
    mut index: usize,
    end: usize,
    list_depth: i32,
    commas: bool,
    cursor_depth: i32,
    tables: &mut Vec<TableUse>,
) -> usize {
    while index < end {
        if tokens[index].depth < list_depth {
            return index;
        }
        if tokens[index].kind == TokKind::Open {
            let after_group = if let Some(close_at) = matching_close(tokens, index) {
                walk_tables(tokens, index + 1, close_at, cursor_depth, tables);
                close_at + 1
            } else {
                walk_tables(tokens, index + 1, tokens.len(), cursor_depth, tables);
                tokens.len()
            };
            let (_, next) = consume_alias(tokens, after_group.min(end), list_depth);
            index = next;
            if !commas {
                return index;
            }
            if index < end && tokens[index].kind == TokKind::Comma && tokens[index].depth == list_depth {
                index += 1;
                continue;
            }
            return index;
        }
        if tokens[index].depth > list_depth {
            index += 1;
            continue;
        }
        if tokens[index].kind == TokKind::Close || tokens[index].kind == TokKind::Comma && !commas {
            return index;
        }
        if tokens[index].kind == TokKind::Comma {
            index += 1;
            continue;
        }
        if !is_name_token(&tokens[index], list_depth) {
            return index;
        }
        let Some((name, next)) = consume_qualified(tokens, index, list_depth) else {
            return index;
        };
        let (alias, next) = consume_alias(tokens, next, list_depth);
        tables.push(TableUse {
            name: name.to_lowercase(),
            alias,
        });
        index = next;
        if !commas {
            return index;
        }
        if index < end && tokens[index].kind == TokKind::Comma && tokens[index].depth == list_depth {
            index += 1;
            continue;
        }
        return index;
    }
    index
}

fn is_table_introducer(word: &str) -> bool {
    word.eq_ignore_ascii_case("FROM")
        || word.eq_ignore_ascii_case("JOIN")
        || word.eq_ignore_ascii_case("UPDATE")
        || word.eq_ignore_ascii_case("INTO")
}

fn is_name_token(token: &Tok<'_>, depth: i32) -> bool {
    token.kind == TokKind::Word && token.depth == depth && (token.quoted || !is_reserved(token.text))
}

fn is_alias_token(token: &Tok<'_>, depth: i32) -> bool {
    is_name_token(token, depth)
}

fn consume_qualified(tokens: &[Tok<'_>], index: usize, depth: i32) -> Option<(String, usize)> {
    if index >= tokens.len() || !is_name_token(&tokens[index], depth) {
        return None;
    }
    let mut name = tokens[index].text.to_owned();
    let mut next = index + 1;
    while next + 1 < tokens.len() && tokens[next].kind == TokKind::Dot && is_name_token(&tokens[next + 1], depth) {
        name = tokens[next + 1].text.to_owned();
        next += 2;
    }
    Some((name, next))
}

fn consume_alias(tokens: &[Tok<'_>], index: usize, depth: i32) -> (Option<String>, usize) {
    if index >= tokens.len() {
        return (None, index);
    }
    let token = &tokens[index];
    if token.kind != TokKind::Word || token.depth != depth {
        return (None, index);
    }
    if !token.quoted && token.text.eq_ignore_ascii_case("AS") {
        let name_at = index + 1;
        if name_at < tokens.len() && is_alias_token(&tokens[name_at], depth) {
            return (Some(tokens[name_at].text.to_lowercase()), name_at + 1);
        }
        return (None, index + 1);
    }
    if is_alias_token(token, depth) {
        return (Some(token.text.to_lowercase()), index + 1);
    }
    (None, index)
}

fn matching_close(tokens: &[Tok<'_>], open_at: usize) -> Option<usize> {
    let mut depth = 0i32;
    for (offset, token) in tokens.iter().enumerate().skip(open_at) {
        match token.kind {
            TokKind::Open => depth += 1,
            TokKind::Close => {
                depth -= 1;
                if depth == 0 {
                    return Some(offset);
                }
            }
            _ => {}
        }
    }
    None
}

fn is_reserved(word: &str) -> bool {
    const WORDS: &[&str] = &[
        "AND",
        "AS",
        "CROSS",
        "DELETE",
        "EXCEPT",
        "FETCH",
        "FROM",
        "FULL",
        "GROUP",
        "HAVING",
        "INNER",
        "INSERT",
        "INTERSECT",
        "INTO",
        "JOIN",
        "LEFT",
        "LIMIT",
        "NATURAL",
        "OFFSET",
        "ON",
        "OR",
        "ORDER",
        "OUTER",
        "RETURNING",
        "RIGHT",
        "SELECT",
        "SET",
        "UNION",
        "UPDATE",
        "USING",
        "VALUES",
        "WHERE",
        "WINDOW",
    ];
    WORDS.iter().any(|keyword| keyword.eq_ignore_ascii_case(word))
}

fn statement_bounds(text: &str, cursor: usize) -> (usize, usize) {
    let bytes = text.as_bytes();
    let mut index = 0;
    let mut depth = 0i32;
    let mut start = 0usize;
    while index < bytes.len() {
        if let Some(next) = skip_comment(text, index) {
            index = next;
            continue;
        }
        let byte = bytes[index];
        if byte == b'\'' || byte == b'"' || byte == b'`' {
            index = skip_quoted(text, index, byte).0;
            continue;
        }
        if byte == b'(' {
            depth += 1;
        } else if byte == b')' {
            depth = depth.saturating_sub(1);
        } else if byte == b';' && depth == 0 {
            if index < cursor {
                start = index + 1;
            } else {
                return (start, index);
            }
        }
        index += 1;
    }
    (start, text.len())
}

fn paren_depth(text: &str, start: usize, end: usize) -> i32 {
    let bytes = text.as_bytes();
    let end = end.min(bytes.len());
    let mut index = start.min(end);
    let mut depth = 0i32;
    while index < end {
        if let Some(next) = skip_comment(text, index) {
            index = next.min(end);
            continue;
        }
        let byte = bytes[index];
        if byte == b'\'' || byte == b'"' || byte == b'`' {
            index = skip_quoted(text, index, byte).0.min(end);
            continue;
        }
        if byte == b'(' {
            depth += 1;
        } else if byte == b')' {
            depth = depth.saturating_sub(1);
        }
        index += 1;
    }
    depth
}

fn tokenize<'a>(text: &'a str, start: usize, end: usize) -> Vec<Tok<'a>> {
    let bytes = text.as_bytes();
    let end = end.min(text.len());
    let mut index = start.min(end);
    let mut depth = 0i32;
    let mut tokens = Vec::new();
    while index < end {
        if let Some(next) = skip_comment(text, index) {
            index = next.min(end);
            continue;
        }
        let byte = bytes[index];
        if byte.is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if byte == b'\'' {
            index = skip_quoted(text, index, byte).0.min(end);
            continue;
        }
        if byte == b'"' || byte == b'`' {
            let (next, inner) = skip_quoted(text, index, byte);
            tokens.push(Tok {
                kind: TokKind::Word,
                text: inner,
                depth,
                quoted: true,
            });
            let next = next.min(end);
            index = if next > index { next } else { index + 1 };
            continue;
        }
        if byte == b'(' {
            tokens.push(Tok {
                kind: TokKind::Open,
                text: "(",
                depth,
                quoted: false,
            });
            depth += 1;
            index += 1;
            continue;
        }
        if byte == b')' {
            depth = depth.saturating_sub(1);
            tokens.push(Tok {
                kind: TokKind::Close,
                text: ")",
                depth,
                quoted: false,
            });
            index += 1;
            continue;
        }
        if byte == b',' {
            tokens.push(Tok {
                kind: TokKind::Comma,
                text: ",",
                depth,
                quoted: false,
            });
            index += 1;
            continue;
        }
        if byte == b'.' {
            tokens.push(Tok {
                kind: TokKind::Dot,
                text: ".",
                depth,
                quoted: false,
            });
            index += 1;
            continue;
        }
        if is_ident_byte(byte) {
            let word_start = index;
            index += 1;
            while index < end && is_ident_byte(bytes[index]) {
                index += 1;
            }
            tokens.push(Tok {
                kind: TokKind::Word,
                text: &text[word_start..index],
                depth,
                quoted: false,
            });
            continue;
        }
        index += 1;
    }
    tokens
}

fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn skip_comment(text: &str, index: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    if index + 1 >= bytes.len() {
        return None;
    }
    if bytes[index] == b'-' && bytes[index + 1] == b'-' {
        let mut next = index + 2;
        while next < bytes.len() && bytes[next] != b'\n' {
            next += 1;
        }
        return Some(next);
    }
    if bytes[index] == b'/' && bytes[index + 1] == b'*' {
        let mut next = index + 2;
        while next + 1 < bytes.len() && !(bytes[next] == b'*' && bytes[next + 1] == b'/') {
            next += 1;
        }
        return Some(if next + 1 < bytes.len() { next + 2 } else { bytes.len() });
    }
    None
}

/// Returns the index after the closing quote and the raw interior.
fn skip_quoted(text: &str, start: usize, quote: u8) -> (usize, &str) {
    let bytes = text.as_bytes();
    let mut index = start + 1;
    let content_start = index.min(bytes.len());
    while index < bytes.len() {
        if bytes[index] == quote {
            if index + 1 < bytes.len() && bytes[index + 1] == quote {
                index += 2;
                continue;
            }
            return (index + 1, &text[content_start..index]);
        }
        if quote == b'\'' && bytes[index] == b'\\' {
            index += 1;
            if index < bytes.len() {
                index += 1;
            }
            continue;
        }
        index += 1;
    }
    (bytes.len(), &text[content_start..bytes.len()])
}

fn floor_char_boundary(text: &str, mut index: usize) -> usize {
    if index > text.len() {
        index = text.len();
    }
    while index > 0 && !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::completion::CompletionItemKind;
    use crate::query::schema_completion::{CompletionContext, SchemaCompletionProvider};
    use crate::runtime::{UiSchemaColumn, UiSchemaForeignKey, UiSchemaSummary, UiTableSummary};

    fn at_end(sql: &str) -> CompletionScope {
        analyze(sql, sql.len(), 0)
    }

    #[test]
    fn outer_where_ignores_from_inside_subquery() {
        let sql = "SELECT * FROM orders WHERE id IN (SELECT id FROM other) AND ";
        let scope = at_end(sql);
        assert_eq!(scope.clause, SqlClause::Where);
        assert_eq!(scope.referenced_tables, vec!["orders".to_string()]);
    }

    #[test]
    fn cursor_inside_subquery_uses_inner_from() {
        let sql = "SELECT * FROM orders WHERE id IN (SELECT id FROM other WHERE ";
        let scope = at_end(sql);
        assert_eq!(scope.clause, SqlClause::Where);
        assert!(scope.referenced_tables.iter().any(|table| table == "other"));
    }

    #[test]
    fn comma_from_maps_both_aliases() {
        let sql = "SELECT * FROM orders o, customers c WHERE ";
        let scope = at_end(sql);
        assert_eq!(scope.aliases.get("o").map(String::as_str), Some("orders"));
        assert_eq!(scope.aliases.get("c").map(String::as_str), Some("customers"));
        assert_eq!(
            scope.referenced_tables,
            vec!["orders".to_string(), "customers".to_string()]
        );

        let bare = at_end("SELECT * FROM a, b b2 ");
        assert_eq!(bare.aliases.get("b2").map(String::as_str), Some("b"));
        assert!(!bare.aliases.contains_key("a"));
        assert_eq!(bare.referenced_tables, vec!["a".to_string(), "b".to_string()]);

        let with_as = at_end("SELECT * FROM public.orders AS o, customers AS c ");
        assert_eq!(with_as.aliases.get("o").map(String::as_str), Some("orders"));
        assert_eq!(with_as.aliases.get("c").map(String::as_str), Some("customers"));
        assert!(!with_as.referenced_tables.iter().any(|table| table == "public"));
    }

    #[test]
    fn references_stay_inside_the_statement_at_the_cursor() {
        let sql = "SELECT * FROM other o; SELECT * FROM orders o, customers c WHERE ";
        let scope = at_end(sql);
        assert_eq!(scope.aliases.get("o").map(String::as_str), Some("orders"));
        assert_eq!(scope.aliases.get("c").map(String::as_str), Some("customers"));
        assert!(!scope.referenced_tables.iter().any(|table| table == "other"));
    }

    #[test]
    fn table_inside_another_word_is_not_referenced() {
        let sql = "SELECT status, created_at FROM orders WHERE ";
        let scope = at_end(sql);
        assert_eq!(scope.referenced_tables, vec!["orders".to_string()]);
        assert!(!scope.referenced_tables.iter().any(|table| table == "at"));
    }

    #[test]
    fn strings_and_comments_are_not_keywords() {
        let quoted = at_end("SELECT * FROM orders WHERE note = 'FROM (other)' /* JOIN */ AND ");
        assert_eq!(quoted.clause, SqlClause::Where);
        assert_eq!(quoted.referenced_tables, vec!["orders".to_string()]);

        let comment = at_end("SELECT * FROM orders -- WHERE other\n");
        assert_eq!(comment.clause, SqlClause::From);
        assert_eq!(comment.referenced_tables, vec!["orders".to_string()]);
    }

    #[test]
    fn insert_column_list_keeps_insert_clause() {
        let scope = analyze("INSERT INTO users (id, em", "INSERT INTO users (id, em".len(), 2);
        assert_eq!(scope.clause, SqlClause::InsertInto);
        assert_eq!(scope.referenced_tables, vec!["users".to_string()]);

        let updated = analyze("UPDATE users SET em", "UPDATE users SET em".len(), 2);
        assert_eq!(updated.clause, SqlClause::Set);
        assert_eq!(updated.referenced_tables, vec!["users".to_string()]);
    }

    fn column(name: &str, primary: bool) -> UiSchemaColumn {
        UiSchemaColumn {
            name: name.to_owned(),
            data_type: "integer".to_owned(),
            nullable: !primary,
            is_primary_key: primary,
        }
    }

    fn provide(before: &str, after: &str, summary: &UiSchemaSummary) -> Vec<crate::editor::completion::CompletionItem> {
        let context = CompletionContext {
            text_before_cursor: before,
            text_after_cursor: after,
            cursor_offset: before.len(),
            active_schema: "public",
            schema_summary: summary,
            cached_tokens: None,
            is_sqlite: false,
            is_manual_trigger: true,
        };
        SchemaCompletionProvider::provide(&context).1
    }

    #[test]
    fn substring_table_name_is_not_a_referenced_completion() {
        let summary = UiSchemaSummary {
            table_details: vec![
                UiTableSummary {
                    schema: "public".to_owned(),
                    name: "orders".to_owned(),
                    row_count: None,
                    columns: vec![column("email", false)],
                    foreign_keys: vec![],
                },
                UiTableSummary {
                    schema: "public".to_owned(),
                    name: "at".to_owned(),
                    row_count: None,
                    columns: vec![column("email_secret", false)],
                    foreign_keys: vec![],
                },
            ],
            ..Default::default()
        };
        let items = provide("SELECT status, created_at FROM orders WHERE em", "", &summary);
        assert!(items.iter().any(|item| item.label == "email"));
        assert!(!items.iter().any(|item| item.label.contains("email_secret")));
    }

    #[test]
    fn join_suggests_the_missing_foreign_key_table_with_on() {
        let summary = UiSchemaSummary {
            table_details: vec![
                UiTableSummary {
                    schema: "public".to_owned(),
                    name: "orders".to_owned(),
                    row_count: None,
                    columns: vec![column("customer_id", false)],
                    foreign_keys: vec![UiSchemaForeignKey {
                        name: "orders_customer".to_owned(),
                        from_columns: vec!["customer_id".to_owned()],
                        to_schema: "public".to_owned(),
                        to_table: "customers".to_owned(),
                        to_columns: vec!["id".to_owned()],
                    }],
                },
                UiTableSummary {
                    schema: "public".to_owned(),
                    name: "customers".to_owned(),
                    row_count: None,
                    columns: vec![column("id", true)],
                    foreign_keys: vec![],
                },
            ],
            ..Default::default()
        };
        let items = provide("SELECT * FROM orders JOIN cu", "", &summary);
        let snippet = items
            .iter()
            .find(|item| item.insert_text.contains("customers ON"))
            .expect("missing FK join target");
        assert!(snippet.insert_text.contains("orders.customer_id = customers.id"));
    }

    #[test]
    fn ambiguous_columns_use_alias_or_table_qualifier() {
        let summary = UiSchemaSummary {
            table_details: vec![
                UiTableSummary {
                    schema: "public".to_owned(),
                    name: "orders".to_owned(),
                    row_count: None,
                    columns: vec![column("id", true)],
                    foreign_keys: vec![],
                },
                UiTableSummary {
                    schema: "public".to_owned(),
                    name: "customers".to_owned(),
                    row_count: None,
                    columns: vec![column("id", true)],
                    foreign_keys: vec![],
                },
            ],
            ..Default::default()
        };
        let aliased = provide("SELECT i", " FROM orders o, customers c", &summary);
        assert!(aliased.iter().any(|item| item.insert_text == "o.id"));
        assert!(aliased.iter().any(|item| item.insert_text == "c.id"));

        let bare = provide("SELECT i", " FROM orders, customers", &summary);
        assert!(bare.iter().any(|item| item.insert_text == "orders.id"));
        assert!(bare.iter().any(|item| item.insert_text == "customers.id"));
    }

    #[test]
    fn join_suggestion_insert_text_contains_on() {
        let summary = UiSchemaSummary {
            table_details: vec![
                UiTableSummary {
                    schema: "public".to_owned(),
                    name: "customers".to_owned(),
                    row_count: None,
                    columns: vec![column("id", true)],
                    foreign_keys: vec![],
                },
                UiTableSummary {
                    schema: "public".to_owned(),
                    name: "orders".to_owned(),
                    row_count: None,
                    columns: vec![column("customer_id", false)],
                    foreign_keys: vec![UiSchemaForeignKey {
                        name: "orders_customer_id_fkey".to_owned(),
                        from_columns: vec!["customer_id".to_owned()],
                        to_schema: "public".to_owned(),
                        to_table: "customers".to_owned(),
                        to_columns: vec!["id".to_owned()],
                    }],
                },
            ],
            ..Default::default()
        };
        let items = provide("SELECT * FROM orders JOIN customers ON", "", &summary);
        assert!(items.iter().any(|item| {
            item.kind == CompletionItemKind::Snippet
                && item.insert_text.contains("ON")
                && item.insert_text.contains('=')
                && item.insert_text.contains("orders.customer_id")
                && item.insert_text.contains("customers.id")
        }));
    }
}
