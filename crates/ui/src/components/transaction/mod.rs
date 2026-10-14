mod config;
mod handler;
mod ui;

pub use ui::{DestructiveOperationDialog, TransactionBar};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionAction {
    Commit,
    Rollback,
    Begin,
    ToggleAutoCommit(bool),
}

#[cfg(test)]
#[path = "tests/mod_tests.rs"]
mod tests;
