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
mod tests {
    // Rendering coverage for this component remains in the component gallery tests.
    // Pure interaction and state-transition coverage lives beside the handler code.
}
