//! Tests for `../query_transaction_surface_view.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn transaction_actions_keep_runtime_effects_explicit() {
        assert_ne!(
            QueryTransactionAction::DismissDisconnectGuard,
            QueryTransactionAction::Transaction(crate::components::TransactionAction::Commit)
        );
    }