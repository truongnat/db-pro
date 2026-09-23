use crate::domain::safety::has_top_level_sql_keyword;

pub(super) enum StatementClass {
    Read,
    Write,
}

pub(super) fn classify_statement(sql: &str) -> StatementClass {
    let keyword = effective_keyword(sql);
    match keyword {
        Some(k) if is_read_keyword(&k) => StatementClass::Read,
        Some(k) if is_row_producing_mutation(&k, sql) => StatementClass::Read,
        _ => StatementClass::Write,
    }
}

fn is_row_producing_mutation(keyword: &str, sql: &str) -> bool {
    matches!(keyword, "INSERT" | "UPDATE" | "DELETE" | "MERGE") && has_top_level_sql_keyword(sql, "RETURNING")
}

fn is_read_keyword(word: &str) -> bool {
    matches!(word, "SELECT" | "SHOW" | "EXPLAIN" | "TABLE")
}

/// Extracts the effective first keyword of a SQL statement, handling
/// leading comments and WITH...CTE chains. For `WITH cte AS (...) UPDATE ...`,
/// returns "UPDATE" (not "WITH").
fn effective_keyword(sql: &str) -> Option<String> {
    let trimmed = strip_leading_comments(sql).trim_start();
    let upper = trimmed.to_ascii_uppercase();
    let first = upper.split_whitespace().next()?;
    if first != "WITH" && !first.starts_with("WITH") {
        return Some(first.to_string());
    }
    if !is_with_keyword(trimmed) {
        return Some(first.to_string());
    }
    Some(scan_with_statement(trimmed).unwrap_or_else(|| "WITH".to_string()))
}

fn is_with_keyword(sql: &str) -> bool {
    !sql[4..]
        .chars()
        .next()
        .is_some_and(|character| character.is_alphanumeric())
}

fn scan_with_statement(sql: &str) -> Option<String> {
    let chars: Vec<char> = sql.chars().collect();
    let len = chars.len();
    let mut i = 4; // skip "WITH"
    let mut depth: i32 = 0;
    let mut in_string = false;

    while i < len {
        if in_string {
            if chars[i] == '\'' {
                if i + 1 < len && chars[i + 1] == '\'' {
                    i += 1; // skip escaped quote
                } else {
                    in_string = false;
                }
            }
            i += 1;
            continue;
        }

        match chars[i] {
            '\'' => in_string = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    // Skip whitespace after closing paren
                    i += 1;
                    while i < len && chars[i].is_whitespace() {
                        i += 1;
                    }
                    if i < len && chars[i] == ',' {
                        // Another CTE follows — skip name + AS
                        i += 1;
                        continue;
                    }
                    // No comma: next word is the main statement keyword
                    let remaining: String = chars[i..].iter().collect();
                    return remaining.split_whitespace().next().map(|s| s.to_string());
                }
            }
            _ => {}
        }
        i += 1;
    }

    None
}

fn strip_leading_comments(sql: &str) -> &str {
    let mut s = sql.trim_start();
    loop {
        if s.starts_with("--") {
            // Line comment: skip to end of line
            s = s.find('\n').map(|i| &s[i + 1..]).unwrap_or("").trim_start();
        } else if s.starts_with("/*") {
            // Block comment: skip to */
            s = s.find("*/").map(|i| &s[i + 2..]).unwrap_or("").trim_start();
        } else {
            break;
        }
    }
    s
}
