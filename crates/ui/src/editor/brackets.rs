//! Lightweight delimiter matching for the SQL editor.
//!
//! This is deliberately independent from the renderer so pairing and matching
//! stay testable without an egui context. It skips SQL strings/comments while
//! looking for structural parentheses and brackets.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DelimiterMatch {
    pub delimiter: usize,
    pub matching: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DelimiterIssue {
    pub offset: usize,
    pub character: char,
    pub expected: Option<char>,
}

pub fn opening_pair(character: char) -> Option<char> {
    match character {
        '(' => Some(')'),
        '[' => Some(']'),
        '\'' => Some('\''),
        '"' => Some('"'),
        _ => None,
    }
}

pub fn closing_pair(character: char) -> bool {
    matches!(character, ')' | ']' | '\'' | '"')
}

pub fn auto_pair_range(text: &str, cursor: usize) -> Option<(usize, usize)> {
    let mut cursor = cursor.min(text.len());
    while cursor > 0 && !text.is_char_boundary(cursor) {
        cursor -= 1;
    }
    let (opening_offset, opening) = text[..cursor].char_indices().next_back()?;
    let closing = text[cursor..].chars().next()?;
    (opening_pair(opening) == Some(closing)).then_some((opening_offset, cursor + closing.len_utf8()))
}

/// Finds a delimiter immediately before or at the caret.
pub fn delimiter_near_cursor(text: &str, cursor: usize) -> Option<DelimiterMatch> {
    let mut cursor = cursor.min(text.len());
    while cursor > 0 && !text.is_char_boundary(cursor) {
        cursor -= 1;
    }
    let previous = text[..cursor].char_indices().next_back();
    if let Some((offset, character)) = previous {
        if is_delimiter(character) {
            return Some(DelimiterMatch {
                delimiter: offset,
                matching: matching_delimiter(text, offset),
            });
        }
    }

    let current = text[cursor..].chars().next().map(|character| (cursor, character));
    current.and_then(|(offset, character)| {
        is_delimiter(character).then(|| DelimiterMatch {
            delimiter: offset,
            matching: matching_delimiter(text, offset),
        })
    })
}

pub fn is_unmatched_structural_delimiter(text: &str, offset: usize) -> bool {
    let Some(character) = text[offset..].chars().next() else {
        return false;
    };
    matches!(character, '(' | ')' | '[' | ']') && matching_delimiter(text, offset).is_none()
}

pub fn unmatched_structural_delimiters(text: &str) -> Vec<(usize, char)> {
    structural_delimiter_issues(text)
        .into_iter()
        .map(|issue| (issue.offset, issue.character))
        .collect()
}

pub fn structural_delimiter_issues(text: &str) -> Vec<DelimiterIssue> {
    let mut issues = Vec::new();
    let mut stack = Vec::new();
    let mut offset = 0;
    while offset < text.len() {
        if let Some(end) = protected_end(text, offset) {
            offset = end;
            continue;
        }
        let Some(character) = text[offset..].chars().next() else {
            break;
        };
        match character {
            '(' | '[' => stack.push((character, offset)),
            ')' | ']' => {
                let expected = stack.last().and_then(|(opening, _)| opening_pair(*opening));
                if expected == Some(character) {
                    stack.pop();
                } else {
                    issues.push(DelimiterIssue {
                        offset,
                        character,
                        expected,
                    });
                }
            }
            _ => {}
        }
        offset += character.len_utf8();
    }
    issues.extend(stack.into_iter().map(|(character, offset)| DelimiterIssue {
        offset,
        character,
        expected: None,
    }));
    issues.sort_unstable_by_key(|issue| issue.offset);
    issues
}

fn is_delimiter(character: char) -> bool {
    matches!(character, '(' | ')' | '[' | ']' | '\'' | '"')
}

fn matching_delimiter(text: &str, offset: usize) -> Option<usize> {
    let character = text[offset..].chars().next()?;
    match character {
        '(' | '[' => scan_forward(text, offset, character),
        ')' | ']' => scan_backward(text, offset),
        '\'' | '"' => matching_quote(text, offset, character),
        _ => None,
    }
}

fn scan_forward(text: &str, start: usize, opening: char) -> Option<usize> {
    let mut stack = vec![(opening, start)];
    let mut offset = start + opening.len_utf8();
    while offset < text.len() {
        if let Some(end) = protected_end(text, offset) {
            offset = end;
            continue;
        }
        let character = text[offset..].chars().next()?;
        match character {
            '(' | '[' => stack.push((character, offset)),
            ')' | ']' => {
                let expected = stack.last().and_then(|(opening, _)| opening_pair(*opening));
                if expected != Some(character) {
                    return None;
                }
                stack.pop();
                if stack.is_empty() {
                    return Some(offset);
                }
            }
            _ => {}
        }
        offset += character.len_utf8();
    }
    None
}

fn scan_backward(text: &str, start: usize) -> Option<usize> {
    let mut stack = Vec::new();
    let mut offset = 0;
    while offset <= start && offset < text.len() {
        if let Some(end) = protected_end(text, offset) {
            offset = end;
            continue;
        }
        let character = text[offset..].chars().next()?;
        match character {
            '(' | '[' => stack.push((character, offset)),
            ')' | ']' => {
                let expected = stack.last().and_then(|(opening, _)| opening_pair(*opening));
                if offset == start {
                    return (expected == Some(character))
                        .then(|| stack.pop().map(|(_, offset)| offset))
                        .flatten();
                }
                if expected == Some(character) {
                    stack.pop();
                }
            }
            _ => {}
        }
        offset += character.len_utf8();
    }
    None
}

fn matching_quote(text: &str, target: usize, quote: char) -> Option<usize> {
    let mut opening = None;
    let mut offset = 0;
    while offset < text.len() {
        if let Some(end) = non_quote_protected_end(text, offset) {
            offset = end;
            continue;
        }
        let character = text[offset..].chars().next()?;
        if character != quote || is_escaped(text, offset) {
            offset += character.len_utf8();
            continue;
        }
        let next_offset = offset + character.len_utf8();
        if text[next_offset..].starts_with(quote) {
            offset = next_offset + quote.len_utf8();
            continue;
        }
        if opening == Some(offset) {
            return Some(offset);
        }
        if offset == target {
            if let Some(opening) = opening {
                return Some(opening);
            }
            return find_quote_after(text, next_offset, quote);
        }
        opening = Some(offset);
        offset = next_offset;
    }
    None
}

fn find_quote_after(text: &str, mut offset: usize, quote: char) -> Option<usize> {
    while offset < text.len() {
        if let Some(end) = non_quote_protected_end(text, offset) {
            offset = end;
            continue;
        }
        let character = text[offset..].chars().next()?;
        if character == quote {
            if is_escaped(text, offset) {
                offset += character.len_utf8();
                continue;
            }
            let next = offset + character.len_utf8();
            if text[next..].starts_with(quote) {
                offset = next + quote.len_utf8();
                continue;
            }
            return Some(offset);
        }
        offset += character.len_utf8();
    }
    None
}

fn protected_end(text: &str, offset: usize) -> Option<usize> {
    let rest = &text[offset..];
    if rest.starts_with("--") {
        return Some(text[offset..].find('\n').map_or(text.len(), |end| offset + end));
    }
    if rest.starts_with("/*") {
        return Some(text[offset..].find("*/").map_or(text.len(), |end| offset + end + 2));
    }
    if rest.starts_with('\'') || rest.starts_with('"') || rest.starts_with('`') {
        let quote = rest.chars().next()?;
        let mut cursor = offset + quote.len_utf8();
        while cursor < text.len() {
            let character = text[cursor..].chars().next()?;
            cursor += character.len_utf8();
            if character == '\\' && quote == '\'' {
                if let Some(escaped) = text[cursor..].chars().next() {
                    cursor += escaped.len_utf8();
                }
                continue;
            }
            if character == quote {
                if text[cursor..].starts_with(quote) {
                    cursor += quote.len_utf8();
                } else {
                    break;
                }
            }
        }
        return Some(cursor);
    }
    if rest.starts_with('$') {
        let tag_end = text[offset + 1..]
            .char_indices()
            .find(|(_, character)| *character == '$')
            .map(|(index, _)| offset + 1 + index)?;
        let tag = &text[offset..=tag_end];
        if tag[1..tag.len() - 1]
            .chars()
            .all(|character| character.is_alphanumeric() || character == '_')
        {
            let body_start = tag_end + 1;
            return Some(
                text[body_start..]
                    .find(tag)
                    .map_or(text.len(), |end| body_start + end + tag.len()),
            );
        }
    }
    None
}

fn is_escaped(text: &str, offset: usize) -> bool {
    let mut backslashes = 0usize;
    let mut cursor = offset;
    while cursor > 0 {
        cursor = text[..cursor].char_indices().next_back().map_or(0, |(index, _)| index);
        if text[cursor..].starts_with('\\') {
            backslashes += 1;
        } else {
            break;
        }
    }
    backslashes % 2 == 1
}

fn non_quote_protected_end(text: &str, offset: usize) -> Option<usize> {
    let rest = &text[offset..];
    if rest.starts_with("--") {
        return Some(text[offset..].find('\n').map_or(text.len(), |end| offset + end));
    }
    if rest.starts_with("/*") {
        return Some(text[offset..].find("*/").map_or(text.len(), |end| offset + end + 2));
    }
    if rest.starts_with('$') {
        let tag_end = text[offset + 1..]
            .char_indices()
            .find(|(_, character)| *character == '$')
            .map(|(index, _)| offset + 1 + index)?;
        let tag = &text[offset..=tag_end];
        if tag[1..tag.len() - 1]
            .chars()
            .all(|character| character.is_alphanumeric() || character == '_')
        {
            let body_start = tag_end + 1;
            return Some(
                text[body_start..]
                    .find(tag)
                    .map_or(text.len(), |end| body_start + end + tag.len()),
            );
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_nested_parentheses_and_brackets() {
        let sql = "SELECT fn(items[1], (value))";
        let opening = sql.find('(').expect("opening parenthesis");
        let matching = matching_delimiter(sql, opening).expect("matching parenthesis");
        assert_eq!(&sql[matching..matching + 1], ")");
        let bracket = sql.find('[').expect("opening bracket");
        assert_eq!(matching_delimiter(sql, bracket), sql.find(']'));
    }

    #[test]
    fn ignores_literals_comments_and_dollar_quotes() {
        let sql = "SELECT '(', $$ [not a bracket] $$, /* ( */ (value)";
        let opening = sql.rfind('(').expect("structural opening");
        assert_eq!(matching_delimiter(sql, opening), sql.rfind(')'));
        assert!(matching_delimiter(sql, sql.find('(').expect("literal opening")).is_none());
    }

    #[test]
    fn cursor_matching_prefers_delimiter_before_caret() {
        let sql = "SELECT (value)";
        let cursor = sql.find(')').expect("closing delimiter");
        let result = delimiter_near_cursor(sql, cursor).expect("delimiter near cursor");
        assert_eq!(result.delimiter, cursor);
        assert_eq!(result.matching, sql.find('('));
    }

    #[test]
    fn reports_mixed_delimiter_mismatch() {
        let sql = "SELECT ([)]";
        let mismatch = structural_delimiter_issues(sql)
            .into_iter()
            .find(|issue| issue.character == ')')
            .expect("mismatch diagnostic");
        assert_eq!(mismatch.expected, Some(']'));
        assert!(matching_delimiter(sql, sql.find('(').expect("opening parenthesis")).is_none());
    }

    #[test]
    fn handles_escaped_e_strings_and_quoted_identifiers() {
        let sql = r#"SELECT E'it\'s (safe)', "name" FROM users"#;
        let string_start = sql.find('\'').expect("string quote");
        let string_end = sql.rfind('\'').expect("string end quote");
        assert_eq!(matching_delimiter(sql, string_start), Some(string_end));
        let identifier_start = sql.find('"').expect("identifier quote");
        let identifier_end = sql.rfind('"').expect("identifier end quote");
        assert_eq!(matching_delimiter(sql, identifier_start), Some(identifier_end));
    }

    #[test]
    fn detects_the_range_to_delete_for_an_auto_pair() {
        let sql = "SELECT ()";
        let cursor = sql.find(')').expect("caret between pair");
        assert_eq!(auto_pair_range(sql, cursor), Some((7, 9)));
    }
}
