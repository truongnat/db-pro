use super::safety_lexer::{token_is_word, tokenize_sql, SqlToken, SqlTokenKind};
use super::{classify_statement_safety, StatementSafety};

/// For WITH (CTE) statements, find the main keyword after the CTE definitions.
/// Data-modifying CTE bodies execute as side effects even when the outer query
/// returns rows, so their mutations must participate in safety classification.
pub(super) fn classify_cte_safety(sql: &str) -> Option<StatementSafety> {
    CteSafetyAnalyzer::new(sql)?.classify()
}

struct CteSafetyAnalyzer<'a> {
    sql: &'a str,
    tokens: Vec<SqlToken>,
    cursor: usize,
    has_mutation: bool,
    has_destructive_delete: bool,
}

impl<'a> CteSafetyAnalyzer<'a> {
    fn new(sql: &'a str) -> Option<Self> {
        let tokens = tokenize_sql(sql);
        let with_index = tokens.iter().position(|token| token_is_word(sql, *token, "WITH"))?;
        let mut cursor = with_index + 1;
        if tokens
            .get(cursor)
            .is_some_and(|token| token_is_word(sql, *token, "RECURSIVE"))
        {
            cursor += 1;
        }
        Some(Self {
            sql,
            tokens,
            cursor,
            has_mutation: false,
            has_destructive_delete: false,
        })
    }

    fn classify(mut self) -> Option<StatementSafety> {
        let outer_index = loop {
            let (next_cursor, body_mutation, body_destructive_delete) = match self.inspect_body() {
                Err(safety) => return Some(safety),
                Ok(Some(body)) => body,
                Ok(None) => return None,
            };
            self.has_mutation |= body_mutation;
            self.has_destructive_delete |= body_destructive_delete;
            self.cursor = next_cursor;
            if self.tokens.get(next_cursor - 1).map(|token| token.kind) != Some(SqlTokenKind::Comma) {
                break next_cursor;
            }
        };

        let outer_safety = self
            .tokens
            .get(outer_index)
            .and_then(|token| classify_statement_safety(&self.sql[token.start..]));
        combine_cte_safety(outer_safety, self.has_mutation, self.has_destructive_delete)
    }

    fn inspect_body(&self) -> Result<Option<(usize, bool, bool)>, StatementSafety> {
        let Some(as_index) = find_cte_as_token(self.sql, &self.tokens, self.cursor) else {
            return Ok(None);
        };
        let Some(body_index) = self.body_start(as_index + 1) else {
            return Err(StatementSafety::Destructive);
        };
        let Some(close_index) = matching_token_parenthesis(&self.tokens, body_index) else {
            return Ok(None);
        };
        let body = CteBodyScanner {
            tokens: &self.tokens,
            sql: self.sql,
            start: body_index + 1,
            end: close_index,
        };
        let (body_mutation, body_destructive_delete) = body.classify();
        let next_index = close_index + 1;
        let next_cursor = if self.tokens.get(next_index).map(|token| token.kind) == Some(SqlTokenKind::Comma) {
            next_index + 1
        } else {
            next_index
        };
        Ok(Some((next_cursor, body_mutation, body_destructive_delete)))
    }

    fn body_start(&self, mut index: usize) -> Option<usize> {
        if self
            .tokens
            .get(index)
            .is_some_and(|token| token_is_word(self.sql, *token, "NOT"))
            && self
                .tokens
                .get(index + 1)
                .is_some_and(|token| token_is_word(self.sql, *token, "MATERIALIZED"))
        {
            index += 2;
        } else if self
            .tokens
            .get(index)
            .is_some_and(|token| token_is_word(self.sql, *token, "MATERIALIZED"))
        {
            index += 1;
        }
        (self.tokens.get(index).map(|token| token.kind) == Some(SqlTokenKind::OpenParen)).then_some(index)
    }
}

fn find_cte_as_token(sql: &str, tokens: &[SqlToken], start: usize) -> Option<usize> {
    let mut depth = 0i32;
    for (index, token) in tokens.iter().enumerate().skip(start) {
        match token.kind {
            SqlTokenKind::OpenParen => depth += 1,
            SqlTokenKind::CloseParen => depth = depth.saturating_sub(1),
            SqlTokenKind::Word if depth == 0 && token_is_word(sql, *token, "AS") => return Some(index),
            _ => {}
        }
    }
    None
}

fn matching_token_parenthesis(tokens: &[SqlToken], open_index: usize) -> Option<usize> {
    let mut depth = 0i32;
    for (index, token) in tokens.iter().enumerate().skip(open_index) {
        match token.kind {
            SqlTokenKind::OpenParen => depth += 1,
            SqlTokenKind::CloseParen => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

struct CteBodyScanner<'a> {
    tokens: &'a [SqlToken],
    sql: &'a str,
    start: usize,
    end: usize,
}

impl CteBodyScanner<'_> {
    fn classify(&self) -> (bool, bool) {
        let mut depth = 0i32;
        let mut has_mutation = false;
        let mut has_destructive_delete = false;

        for index in self.start..self.end {
            let token = self.tokens[index];
            if token.kind == SqlTokenKind::Word {
                if token_is_word(self.sql, token, "DROP") || token_is_word(self.sql, token, "TRUNCATE") {
                    has_mutation = true;
                    has_destructive_delete = true;
                } else if token_is_word(self.sql, token, "INSERT") || token_is_word(self.sql, token, "UPDATE") {
                    has_mutation = true;
                } else if token_is_word(self.sql, token, "DELETE") {
                    has_mutation = true;
                    if !self.delete_has_same_level_where(index, depth) {
                        has_destructive_delete = true;
                    }
                }
            }

            match token.kind {
                SqlTokenKind::OpenParen => depth += 1,
                SqlTokenKind::CloseParen => depth = depth.saturating_sub(1),
                _ => {}
            }
        }

        (has_mutation, has_destructive_delete)
    }

    fn delete_has_same_level_where(&self, delete_index: usize, delete_depth: i32) -> bool {
        let mut depth = delete_depth;
        for token in self.tokens.iter().take(self.end).skip(delete_index + 1) {
            match token.kind {
                SqlTokenKind::OpenParen => depth += 1,
                SqlTokenKind::CloseParen => depth = depth.saturating_sub(1),
                SqlTokenKind::Word if depth == delete_depth && token_is_word(self.sql, *token, "WHERE") => {
                    return true;
                }
                _ => {}
            }
        }
        false
    }
}

fn combine_cte_safety(
    outer_safety: Option<StatementSafety>,
    has_mutation: bool,
    has_destructive_delete: bool,
) -> Option<StatementSafety> {
    if !has_mutation {
        return outer_safety;
    }
    if has_destructive_delete || outer_safety == Some(StatementSafety::Destructive) {
        Some(StatementSafety::Destructive)
    } else if outer_safety == Some(StatementSafety::Read) {
        Some(StatementSafety::Write)
    } else {
        outer_safety
    }
}
