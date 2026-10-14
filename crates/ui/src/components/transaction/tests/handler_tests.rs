//! Tests for `../handler.rs` (moved out of the source file).
use super::*;

#[test]
fn status_prioritizes_auto_commit_and_pending_work() {
    assert_eq!(transaction_status(true, true, 3), TransactionStatus::AutoCommit);
    assert_eq!(transaction_status(false, true, 1), TransactionStatus::Uncommitted);
    assert_eq!(transaction_status(false, true, 0), TransactionStatus::Active);
    assert_eq!(transaction_status(false, false, 4), TransactionStatus::Inactive);
    assert_eq!(status_text(TransactionStatus::Uncommitted), "Uncommitted Transaction");
}

#[test]
fn transaction_action_respects_available_controls() {
    assert_eq!(
        transaction_action(true, false, true, false, false),
        Some(TransactionAction::Rollback)
    );
    assert_eq!(
        transaction_action(true, false, true, true, false),
        Some(TransactionAction::Commit)
    );
    assert_eq!(
        transaction_action(false, false, false, false, true),
        Some(TransactionAction::Begin)
    );
    assert_eq!(transaction_action(false, true, false, false, true), None);
}

#[test]
fn confirmation_requires_exact_keyword_after_trimming() {
    assert!(confirmation_is_valid("  DROP\n", "DROP"));
    assert!(!confirmation_is_valid("drop", "DROP"));
    assert!(!confirmation_is_valid("DROP TABLE", "DROP"));
    assert_eq!(pending_mutation_suffix(1), "");
    assert_eq!(pending_mutation_suffix(2), "s");
}

#[test]
fn dialog_actions_close_without_confusing_cancel_with_confirmation() {
    let mut open = true;
    let action = destructive_dialog_action(false, true, true);
    assert!(apply_destructive_dialog_action(&mut open, action));
    assert!(!open);

    open = true;
    let action = destructive_dialog_action(true, false, false);
    assert!(!apply_destructive_dialog_action(&mut open, action));
    assert!(!open);

    open = true;
    let action = destructive_dialog_action(false, true, false);
    assert_eq!(action, DestructiveDialogAction::None);
    assert!(!apply_destructive_dialog_action(&mut open, action));
    assert!(open);
}
