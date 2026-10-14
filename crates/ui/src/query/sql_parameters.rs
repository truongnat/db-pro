// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
//! Named / numbered / positional SQL parameter discovery (#225).
//!
//! Tokens inside single-quoted strings, `--` / `/* */` comments, and dollar-quoted
//! bodies (`$$` / `$tag$`) are ignored. `?` is a placeholder only for drivers that
//! bind it (MySQL, SQLite). PostgreSQL jsonb and SQL Server keep `?` as syntax.

use std::collections::{BTreeSet, HashMap};

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
    /// SQL Server `@p1`, `@p2`, … (1-based).
    AtPn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedSql {
    pub sql: String,
    /// Bound values in left-to-right placeholder order (as text; UI maps to `QueryParam`).
    pub values: Vec<String>,
}

struct ParameterHit {
    name: String,
    kind: ParameterKind,
    start: usize,
    end: usize,
}

/// Discover bindable parameters, recording `?` as a positional placeholder.
///
/// Callers that know the driver should use [`discover_sql_parameters_for_driver`].
pub fn discover_sql_parameters(sql: &str) -> Vec<DiscoveredParameter> {
    discover_scanned(sql, true)
}

/// Discover parameters using the driver's placeholder rules.
///
/// PostgreSQL and SQL Server do not treat `?` as a placeholder. MySQL, SQLite,
/// and unknown drivers still do. `:name` and `$n` are recognized for every driver.
pub fn discover_sql_parameters_for_driver(sql: &str, driver: &str) -> Vec<DiscoveredParameter> {
    discover_scanned(sql, question_mark_is_placeholder(driver))
}

/// Placeholder spelling for `driver` (`postgresql`/`postgres`, `sql server`/`sqlserver`/`mssql`, else `?`).
pub fn placeholder_style_for_driver(driver: &str) -> PlaceholderStyle {
    if is_postgres_driver(driver) {
        PlaceholderStyle::NumberedDollar
    } else if is_sql_server_driver(driver) {
        PlaceholderStyle::AtPn
    } else {
        PlaceholderStyle::QuestionMark
    }
}

fn discover_scanned(sql: &str, record_question_mark: bool) -> Vec<DiscoveredParameter> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for hit in scan_parameter_hits(sql, record_question_mark) {
        if hit.kind != ParameterKind::Positional && !seen.insert(hit.name.clone()) {
            continue;
        }
        out.push(DiscoveredParameter {
            name: hit.name,
            kind: hit.kind,
            offset: hit.start,
        });
    }
    out
}

/// Rewrite discovered parameters into provider placeholders and collect bind values.
///
/// `?` is rewritten only for [`PlaceholderStyle::QuestionMark`]. Other styles leave
/// it in the SQL text (PostgreSQL jsonb, SQL Server). Missing panel values return
/// `Err` with the parameter name that still needs input. A present key, including
/// `""`, is a filled value.
pub fn prepare_bound_sql(
    sql: &str,
    values: &HashMap<String, String>,
    style: PlaceholderStyle,
) -> Result<PreparedSql, String> {
    let hits = scan_parameter_hits(sql, matches!(style, PlaceholderStyle::QuestionMark));
    if hits.is_empty() {
        return Ok(PreparedSql {
            sql: sql.to_owned(),
            values: Vec::new(),
        });
    }

    let mut seen = BTreeSet::new();
    for hit in &hits {
        if !seen.insert(hit.name.as_str()) {
            continue;
        }
        if !values.contains_key(&hit.name) {
            return Err(hit.name.clone());
        }
    }

    let mut out = String::with_capacity(sql.len());
    let mut bound = Vec::with_capacity(hits.len());
    let mut cursor = 0usize;
    for hit in &hits {
        out.push_str(&sql[cursor..hit.start]);
        bound.push(values.get(&hit.name).cloned().unwrap_or_default());
        push_placeholder(&mut out, style, bound.len());
        cursor = hit.end;
    }
    out.push_str(&sql[cursor..]);

    Ok(PreparedSql {
        sql: out,
        values: bound,
    })
}

fn scan_parameter_hits(sql: &str, record_question_mark: bool) -> Vec<ParameterHit> {
    let bytes = sql.as_bytes();
    let mut hits = Vec::new();
    let mut i = 0usize;
    let mut in_single = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut positional = 0usize;

    while i < bytes.len() {
        let ch = bytes[i];
        if in_line_comment {
            if ch == b'\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            if ch == b'*' && bytes.get(i + 1) == Some(&b'/') {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if in_single {
            if ch == b'\'' {
                if bytes.get(i + 1) == Some(&b'\'') {
                    i += 2;
                    continue;
                }
                in_single = false;
            }
            i += 1;
            continue;
        }
        if ch == b'-' && bytes.get(i + 1) == Some(&b'-') {
            in_line_comment = true;
            i += 2;
            continue;
        }
        if ch == b'/' && bytes.get(i + 1) == Some(&b'*') {
            in_block_comment = true;
            i += 2;
            continue;
        }
        if ch == b'\'' {
            in_single = true;
            i += 1;
            continue;
        }

        if ch == b'$' {
            if let Some(end) = dollar_quote_end(bytes, i) {
                i = end;
                continue;
            }
            let start = i;
            i += 1;
            let digits_start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            if i > digits_start {
                let digits = std::str::from_utf8(&bytes[digits_start..i]).unwrap_or("");
                hits.push(ParameterHit {
                    name: format!("${digits}"),
                    kind: ParameterKind::Numbered,
                    start,
                    end: i,
                });
            }
            continue;
        }

        if ch == b':' {
            let next = bytes.get(i + 1).copied();
            if next.is_some_and(|c| c.is_ascii_alphabetic() || c == b'_') {
                let start = i;
                i += 1;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                hits.push(ParameterHit {
                    name: sql[start..i].to_owned(),
                    kind: ParameterKind::Named,
                    start,
                    end: i,
                });
                continue;
            }
        }

        if ch == b'?' && record_question_mark {
            positional += 1;
            let start = i;
            i += 1;
            hits.push(ParameterHit {
                name: format!("?{positional}"),
                kind: ParameterKind::Positional,
                start,
                end: i,
            });
            continue;
        }

        i += 1;
    }

    hits
}

/// End offset (exclusive) of a dollar-quoted literal starting at `start`, when
/// `bytes[start]` opens `$$` or `$tag$`. An unclosed opener consumes the rest:
/// the tail is still quoted, so `$1` / `:name` / `?` inside it are not binds.
fn dollar_quote_end(bytes: &[u8], start: usize) -> Option<usize> {
    let tag_start = start + 1;
    let tag_end = if bytes.get(tag_start) == Some(&b'$') {
        tag_start
    } else if bytes.get(tag_start).is_some_and(|byte| is_dollar_tag_start(*byte)) {
        let mut index = tag_start + 1;
        while index < bytes.len() && is_dollar_tag_continue(bytes[index]) {
            index += 1;
        }
        if bytes.get(index) != Some(&b'$') {
            return None;
        }
        index
    } else {
        return None;
    };
    let tag = &bytes[tag_start..tag_end];
    let mut index = tag_end + 1;
    while index + tag.len() + 2 <= bytes.len() {
        let closing_dollar = index + 1 + tag.len();
        if bytes[index] == b'$' && &bytes[index + 1..closing_dollar] == tag && bytes[closing_dollar] == b'$' {
            return Some(closing_dollar + 1);
        }
        index += 1;
    }
    Some(bytes.len())
}

fn is_dollar_tag_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_dollar_tag_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn question_mark_is_placeholder(driver: &str) -> bool {
    !is_postgres_driver(driver) && !is_sql_server_driver(driver)
}

fn is_postgres_driver(driver: &str) -> bool {
    driver.eq_ignore_ascii_case("postgresql") || driver.eq_ignore_ascii_case("postgres")
}

fn is_sql_server_driver(driver: &str) -> bool {
    driver.eq_ignore_ascii_case("sql server")
        || driver.eq_ignore_ascii_case("sqlserver")
        || driver.eq_ignore_ascii_case("mssql")
}

fn push_placeholder(out: &mut String, style: PlaceholderStyle, index: usize) {
    match style {
        PlaceholderStyle::NumberedDollar => {
            out.push('$');
            out.push_str(&index.to_string());
        }
        PlaceholderStyle::QuestionMark => out.push('?'),
        PlaceholderStyle::AtPn => {
            out.push_str("@p");
            out.push_str(&index.to_string());
        }
    }
}

#[cfg(test)]
#[path = "tests/sql_parameters_tests.rs"]
mod tests;
