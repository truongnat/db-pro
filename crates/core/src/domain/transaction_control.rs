//! Recognize a transaction-control statement the editor can pin to one connection.
//!
//! Only forms that open, close, or roll back the whole transaction are recognized.
//! `COMMIT AND CHAIN` and `ROLLBACK TO SAVEPOINT` stay unrecognized so the caller
//! keeps the connection and the open-transaction flag.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionControl {
    Begin,
    Commit,
    Rollback,
}

/// `Some` when `sql` is a single transaction-control statement.
pub fn transaction_control(sql: &str) -> Option<TransactionControl> {
    let words = control_words(sql);
    let words: Vec<&str> = words.iter().map(String::as_str).collect();
    match words.as_slice() {
        ["begin"] | ["begin", "work"] | ["begin", "transaction"] | ["start", "transaction"] => {
            Some(TransactionControl::Begin)
        }
        ["commit"]
        | ["commit", "work"]
        | ["commit", "transaction"]
        | ["end"]
        | ["end", "work"]
        | ["end", "transaction"] => Some(TransactionControl::Commit),
        ["rollback"] | ["rollback", "work"] | ["rollback", "transaction"] | ["abort"] | ["abort", "work"] => {
            Some(TransactionControl::Rollback)
        }
        ["begin", _, ..] => Some(TransactionControl::Begin),
        _ => None,
    }
}

fn control_words(sql: &str) -> Vec<String> {
    sql.trim()
        .trim_end_matches(';')
        .split_whitespace()
        .map(|word| word.trim_matches(';').to_ascii_lowercase())
        .filter(|word| !word.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{transaction_control, TransactionControl};

    #[test]
    fn simple_controls_and_begin_modes_are_recognized() {
        assert_eq!(transaction_control("BEGIN"), Some(TransactionControl::Begin));
        assert_eq!(transaction_control("  begin ; "), Some(TransactionControl::Begin));
        assert_eq!(
            transaction_control("START TRANSACTION"),
            Some(TransactionControl::Begin)
        );
        assert_eq!(
            transaction_control("BEGIN ISOLATION LEVEL SERIALIZABLE"),
            Some(TransactionControl::Begin)
        );
        assert_eq!(transaction_control("COMMIT WORK"), Some(TransactionControl::Commit));
        assert_eq!(transaction_control("ROLLBACK"), Some(TransactionControl::Rollback));
    }

    #[test]
    fn savepoints_and_chained_commit_are_not_session_edges() {
        assert_eq!(transaction_control("ROLLBACK TO SAVEPOINT s"), None);
        assert_eq!(transaction_control("COMMIT AND CHAIN"), None);
        assert_eq!(transaction_control("SELECT 1"), None);
    }
}
