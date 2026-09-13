use crate::editor::SqlDialect;

const CLAUSE_KEYWORDS: &[&str] = &[
    "INSERT INTO",
    "DELETE FROM",
    "LEFT JOIN",
    "RIGHT JOIN",
    "FULL JOIN",
    "INNER JOIN",
    "CROSS JOIN",
    "GROUP BY",
    "ORDER BY",
    "SELECT",
    "RETURNING",
    "FROM",
    "WHERE",
    "HAVING",
    "JOIN",
    "SET",
    "VALUES",
    "LIMIT",
    "OFFSET",
];

/// Formats common SQL clause boundaries without rewriting protected SQL text.
///
/// This intentionally remains a conservative formatter for incomplete editor input:
/// comments, quoted identifiers, string literals, and PostgreSQL dollar-quoted bodies
/// are copied byte-for-byte. The dialect is accepted at this boundary so callers do not
/// accidentally format a document using a different provider context as the formatter grows.
pub fn format_sql(sql: &str) -> String {
    format_sql_for_dialect(sql, SqlDialect::Postgres)
}

pub fn format_sql_for_dialect(sql: &str, _dialect: SqlDialect) -> String {
    let mut formatted = String::with_capacity(sql.len() + 32);
    let mut offset = 0;
    let mut parenthesis_depth = 0usize;

    while offset < sql.len() {
        if let Some(end) = protected_sql_end(sql, offset) {
            formatted.push_str(&sql[offset..end]);
            offset = end;
            continue;
        }

        let Some(character) = sql[offset..].chars().next() else {
            break;
        };
        if character == '(' {
            parenthesis_depth = parenthesis_depth.saturating_add(1);
            formatted.push(character);
            offset += character.len_utf8();
            continue;
        }
        if character == ')' {
            parenthesis_depth = parenthesis_depth.saturating_sub(1);
            formatted.push(character);
            offset += character.len_utf8();
            continue;
        }

        if let Some(keyword) = matching_clause_keyword(sql, offset) {
            if parenthesis_depth == 0 && needs_clause_break(&formatted) {
                trim_trailing_horizontal_whitespace(&mut formatted);
                if !formatted.ends_with('\n') {
                    formatted.push('\n');
                }
            }
            formatted.push_str(keyword);
            offset += keyword.len();
            continue;
        }

        formatted.push(character);
        offset += character.len_utf8();
    }

    formatted.trim().to_owned()
}

fn protected_sql_end(sql: &str, offset: usize) -> Option<usize> {
    let rest = &sql[offset..];
    if rest.starts_with("--") {
        return Some(sql[offset..].find('\n').map_or(sql.len(), |line_end| offset + line_end));
    }
    if rest.starts_with("/*") {
        return Some(
            sql[offset..]
                .find("*/")
                .map_or(sql.len(), |comment_end| offset + comment_end + 2),
        );
    }

    let quote = rest.chars().next()?;
    if quote == '\'' || quote == '"' || quote == '`' {
        let mut cursor = offset + quote.len_utf8();
        while cursor < sql.len() {
            let character = sql[cursor..].chars().next()?;
            cursor += character.len_utf8();
            if character == quote {
                if sql[cursor..].starts_with(quote) {
                    cursor += quote.len_utf8();
                } else {
                    break;
                }
            }
        }
        return Some(cursor);
    }

    if quote == '$' {
        let tag_end = sql[offset + 1..]
            .char_indices()
            .find(|(_, character)| *character == '$')
            .map(|(index, _)| offset + 1 + index)?;
        let tag = &sql[offset..=tag_end];
        let body_start = tag_end + 1;
        return sql[body_start..]
            .find(tag)
            .map_or(Some(sql.len()), |end| Some(body_start + end + tag.len()));
    }

    None
}

fn matching_clause_keyword(sql: &str, offset: usize) -> Option<&'static str> {
    CLAUSE_KEYWORDS.iter().copied().find(|keyword| {
        let end = offset + keyword.len();
        end <= sql.len()
            && sql[offset..end].eq_ignore_ascii_case(keyword)
            && is_left_word_boundary(sql, offset)
            && is_right_word_boundary(sql, end)
    })
}

fn is_left_word_boundary(sql: &str, offset: usize) -> bool {
    sql[..offset]
        .chars()
        .next_back()
        .is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_')
}

fn is_right_word_boundary(sql: &str, offset: usize) -> bool {
    sql[offset..]
        .chars()
        .next()
        .is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_')
}

fn needs_clause_break(formatted: &str) -> bool {
    !formatted.is_empty() && !formatted.ends_with('\n')
}

fn trim_trailing_horizontal_whitespace(formatted: &mut String) {
    while formatted.ends_with([' ', '\t']) {
        formatted.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_clause_boundaries_without_rewriting_literals_or_comments() {
        let sql = "select 'select from', note from users -- where stays\nwhere id = 1";
        let formatted = format_sql_for_dialect(sql, SqlDialect::Postgres);

        assert!(formatted.contains("'select from'"));
        assert!(formatted.contains("-- where stays"));
        assert!(formatted.contains("SELECT 'select from', note\nFROM users"));
        assert!(formatted.ends_with("\nWHERE id = 1"));
    }

    #[test]
    fn preserves_dollar_quoted_postgres_body() {
        let sql = "select $tag$from where select$tag$ from functions";
        let formatted = format_sql_for_dialect(sql, SqlDialect::Postgres);

        assert!(formatted.contains("$tag$from where select$tag$"));
        assert!(formatted.ends_with("\nFROM functions"));
    }

    #[test]
    fn formats_incomplete_sql_without_panicking() {
        assert_eq!(format_sql("select * from"), "SELECT *\nFROM");
    }
}
