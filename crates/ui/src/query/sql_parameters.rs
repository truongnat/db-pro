//! Named / numbered / positional SQL parameter discovery (#225).
//!
//! Detection is intentionally conservative: tokens inside single-quoted strings
//! or `--` / `/* */` comments are ignored so false positives stay rare.

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
}
