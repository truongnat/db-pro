//! Named / numbered / positional SQL parameter discovery (#225).
//!
//! Detection is intentionally conservative: tokens inside single-quoted strings
//! or `--` / `/* */` comments are ignored so false positives stay rare.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredParameter {
    pub name: String,
    pub kind: ParameterKind,
    pub offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterKind {
    Numbered,
    Named,
    Positional,
}

/// How rewritten bind placeholders are emitted for the active provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceholderStyle {
    /// PostgreSQL-style `$1`, `$2`, …
    NumberedDollar,
    /// SQLite / MySQL-style `?`
    QuestionMark,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedSql {
    pub sql: String,
    /// Bound values in left-to-right placeholder order (as text; UI maps to `QueryParam`).
    pub values: Vec<String>,
}

/// Discover bindable parameters in `sql` while skipping quotes and comments.
pub fn discover_sql_parameters(sql: &str) -> Vec<DiscoveredParameter> {
    let bytes = sql.as_bytes();
    let mut out = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let mut i = 0usize;
    let mut in_single = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while i < bytes.len() {
        let ch = bytes[i] as char;
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            if ch == '*' && bytes.get(i + 1) == Some(&b'/') {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if in_single {
            if ch == '\'' {
                if bytes.get(i + 1) == Some(&b'\'') {
                    i += 2;
                    continue;
                }
                in_single = false;
            }
            i += 1;
            continue;
        }
        if ch == '-' && bytes.get(i + 1) == Some(&b'-') {
            in_line_comment = true;
            i += 2;
            continue;
        }
        if ch == '/' && bytes.get(i + 1) == Some(&b'*') {
            in_block_comment = true;
            i += 2;
            continue;
        }
        if ch == '\'' {
            in_single = true;
            i += 1;
            continue;
        }

        if ch == '$' {
            let start = i;
            i += 1;
            let mut digits = String::new();
            while i < bytes.len() && (bytes[i] as char).is_ascii_digit() {
                digits.push(bytes[i] as char);
                i += 1;
            }
            if !digits.is_empty() {
                let name = format!("${digits}");
                if seen.insert(name.clone()) {
                    out.push(DiscoveredParameter {
                        name,
                        kind: ParameterKind::Numbered,
                        offset: start,
                    });
                }
            }
            continue;
        }

        if ch == ':' {
            let start = i;
            let next = bytes.get(i + 1).map(|b| *b as char);
            if next.is_some_and(|c| c.is_ascii_alphabetic() || c == '_') {
                i += 1;
                let mut name = String::from(":");
                while i < bytes.len() {
                    let c = bytes[i] as char;
                    if c.is_ascii_alphanumeric() || c == '_' {
                        name.push(c);
                        i += 1;
                    } else {
                        break;
                    }
                }
                if seen.insert(name.clone()) {
                    out.push(DiscoveredParameter {
                        name,
                        kind: ParameterKind::Named,
                        offset: start,
                    });
                }
                continue;
            }
        }

        if ch == '?' {
            let name = format!(
                "?{}",
                out.iter().filter(|p| p.kind == ParameterKind::Positional).count() + 1
            );
            // Positional placeholders are ordered and may repeat; keep every occurrence
            // identity by synthesizing ?1, ?2, … for the panel.
            out.push(DiscoveredParameter {
                name,
                kind: ParameterKind::Positional,
                offset: i,
            });
            i += 1;
            continue;
        }

        i += 1;
    }

    out
}

/// Rewrite discovered parameters into provider placeholders and collect bind values.
///
/// Missing panel values return `Err` with the parameter name that still needs input.
pub fn prepare_bound_sql(
    sql: &str,
    values: &HashMap<String, String>,
    style: PlaceholderStyle,
) -> Result<PreparedSql, String> {
    let discovered = discover_sql_parameters(sql);
    if discovered.is_empty() {
        return Ok(PreparedSql {
            sql: sql.to_owned(),
            values: Vec::new(),
        });
    }

    for param in &discovered {
        if !values.contains_key(&param.name) {
            return Err(param.name.clone());
        }
    }

    let bytes = sql.as_bytes();
    let mut out = String::with_capacity(sql.len());
    let mut bound = Vec::new();
    let mut i = 0usize;
    let mut in_single = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut positional_index = 0usize;

    while i < bytes.len() {
        let ch = bytes[i] as char;
        if in_line_comment {
            out.push(ch);
            if ch == '\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            out.push(ch);
            if ch == '*' && bytes.get(i + 1) == Some(&b'/') {
                out.push('/');
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if in_single {
            out.push(ch);
            if ch == '\'' {
                if bytes.get(i + 1) == Some(&b'\'') {
                    out.push('\'');
                    i += 2;
                    continue;
                }
                in_single = false;
            }
            i += 1;
            continue;
        }
        if ch == '-' && bytes.get(i + 1) == Some(&b'-') {
            out.push('-');
            out.push('-');
            in_line_comment = true;
            i += 2;
            continue;
        }
        if ch == '/' && bytes.get(i + 1) == Some(&b'*') {
            out.push('/');
            out.push('*');
            in_block_comment = true;
            i += 2;
            continue;
        }
        if ch == '\'' {
            out.push('\'');
            in_single = true;
            i += 1;
            continue;
        }

        if ch == '$' {
            let start = i;
            i += 1;
            let mut digits = String::new();
            while i < bytes.len() && (bytes[i] as char).is_ascii_digit() {
                digits.push(bytes[i] as char);
                i += 1;
            }
            if !digits.is_empty() {
                let name = format!("${digits}");
                let value = values.get(&name).cloned().unwrap_or_default();
                bound.push(value);
                push_placeholder(&mut out, style, bound.len());
                continue;
            }
            out.push('$');
            // Restart from the character after `$` (already advanced).
            let _ = start;
            continue;
        }

        if ch == ':' {
            let next = bytes.get(i + 1).map(|b| *b as char);
            if next.is_some_and(|c| c.is_ascii_alphabetic() || c == '_') {
                i += 1;
                let mut name = String::from(":");
                while i < bytes.len() {
                    let c = bytes[i] as char;
                    if c.is_ascii_alphanumeric() || c == '_' {
                        name.push(c);
                        i += 1;
                    } else {
                        break;
                    }
                }
                let value = values.get(&name).cloned().unwrap_or_default();
                bound.push(value);
                push_placeholder(&mut out, style, bound.len());
                continue;
            }
        }

        if ch == '?' {
            positional_index += 1;
            let name = format!("?{positional_index}");
            let value = values.get(&name).cloned().unwrap_or_default();
            bound.push(value);
            push_placeholder(&mut out, style, bound.len());
            i += 1;
            continue;
        }

        out.push(ch);
        i += 1;
    }

    Ok(PreparedSql {
        sql: out,
        values: bound,
    })
}

fn push_placeholder(out: &mut String, style: PlaceholderStyle, index: usize) {
    match style {
        PlaceholderStyle::NumberedDollar => {
            out.push('$');
            out.push_str(&index.to_string());
        }
        PlaceholderStyle::QuestionMark => out.push('?'),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_numbered_and_named_outside_literals() {
        let found = discover_sql_parameters(
            "SELECT * FROM t WHERE id = $1 AND name = :customer -- $2\nAND note = ':skip' /* :also */",
        );
        let names: Vec<_> = found.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["$1", ":customer"]);
    }

    #[test]
    fn discovers_positional_placeholders_in_order() {
        let found = discover_sql_parameters("SELECT ? FROM t WHERE a = ? AND b = ?");
        assert_eq!(found.len(), 3);
        assert!(found.iter().all(|p| p.kind == ParameterKind::Positional));
        assert_eq!(found[0].name, "?1");
        assert_eq!(found[2].name, "?3");
    }

    #[test]
    fn prepare_rewrites_named_to_postgres_placeholders() {
        let mut values = HashMap::new();
        values.insert(":id".to_owned(), "7".to_owned());
        values.insert(":name".to_owned(), "Ada".to_owned());
        let prepared = prepare_bound_sql(
            "SELECT * FROM t WHERE id = :id AND name = :name",
            &values,
            PlaceholderStyle::NumberedDollar,
        )
        .expect("prepared");
        assert_eq!(prepared.sql, "SELECT * FROM t WHERE id = $1 AND name = $2");
        assert_eq!(prepared.values, vec!["7".to_owned(), "Ada".to_owned()]);
    }

    #[test]
    fn prepare_rewrites_to_question_marks_for_sqlite() {
        let mut values = HashMap::new();
        values.insert("$1".to_owned(), "1".to_owned());
        values.insert(":name".to_owned(), "x".to_owned());
        let prepared = prepare_bound_sql(
            "SELECT $1, :name FROM t -- :skip\n",
            &values,
            PlaceholderStyle::QuestionMark,
        )
        .expect("prepared");
        assert_eq!(prepared.sql, "SELECT ?, ? FROM t -- :skip\n");
        assert_eq!(prepared.values, vec!["1".to_owned(), "x".to_owned()]);
    }

    #[test]
    fn prepare_errors_when_value_missing() {
        let err =
            prepare_bound_sql("SELECT :id", &HashMap::new(), PlaceholderStyle::QuestionMark).expect_err("missing");
        assert_eq!(err, ":id");
    }
}
