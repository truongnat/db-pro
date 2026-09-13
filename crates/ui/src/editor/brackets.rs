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
    let mut unmatched = Vec::new();
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
                let expected = matching_open(character);
                if stack.last().is_some_and(|(opening, _)| Some(*opening) == expected) {
                    stack.pop();
                } else {
                    unmatched.push((offset, character));
                }
            }
            _ => {}
        }
        offset += character.len_utf8();
    }
    unmatched.extend(stack.into_iter().map(|(character, offset)| (offset, character)));
    unmatched.sort_unstable_by_key(|(offset, _)| *offset);
    unmatched
}

fn is_delimiter(character: char) -> bool {
    matches!(character, '(' | ')' | '[' | ']' | '\'' | '"')
}

fn matching_delimiter(text: &str, offset: usize) -> Option<usize> {
    let character = text[offset..].chars().next()?;
    match character {
        '(' | '[' => scan_forward(text, offset, character),
        ')' | ']' => scan_backward(text, offset, character),
        '\'' | '"' => matching_quote(text, offset, character),
        _ => None,
    }
}

fn scan_forward(text: &str, start: usize, opening: char) -> Option<usize> {
    let closing = opening_pair(opening)?;
    let mut depth = 0usize;
    let mut offset = start;
    while offset < text.len() {
        if let Some(end) = protected_end(text, offset) {
            offset = end;
            continue;
        }
        let character = text[offset..].chars().next()?;
        if character == opening {
            depth = depth.saturating_add(1);
        } else if character == closing {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(offset);
            }
        }
        offset += character.len_utf8();
    }
    None
}

fn scan_backward(text: &str, start: usize, closing: char) -> Option<usize> {
    let opening = matching_open(closing)?;
    let mut stack = Vec::new();
    let mut offset = 0;
    while offset <= start && offset < text.len() {
        if let Some(end) = protected_end(text, offset) {
            offset = end;
            continue;
        }
        let character = text[offset..].chars().next()?;
        if character == opening {
            stack.push((opening, offset));
        } else if character == closing {
            let Some((candidate, candidate_offset)) = stack.pop() else {
                offset += character.len_utf8();
                continue;
            };
            if offset == start {
                return (candidate == opening).then_some(candidate_offset);
            }
        }
        offset += character.len_utf8();
    }
    None
}

fn matching_open(closing: char) -> Option<char> {
    match closing {
        ')' => Some('('),
        ']' => Some('['),
        _ => None,
    }
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
        if character != quote {
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
}
