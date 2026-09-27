use super::TransactionAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TransactionStatus {
    AutoCommit,
    Uncommitted,
    Active,
    Inactive,
}

pub(super) fn transaction_status(
    auto_commit: bool,
    in_transaction: bool,
    pending_mutations: usize,
) -> TransactionStatus {
    if auto_commit {
        return TransactionStatus::AutoCommit;
    }
    if !in_transaction {
        return TransactionStatus::Inactive;
    }
    if pending_mutations > 0 {
        return TransactionStatus::Uncommitted;
    }
    TransactionStatus::Active
}

pub(super) const fn status_text(status: TransactionStatus) -> &'static str {
    match status {
        TransactionStatus::AutoCommit => "Auto-commit Active",
        TransactionStatus::Uncommitted => "Uncommitted Transaction",
        TransactionStatus::Active => "Transaction Active",
        TransactionStatus::Inactive => "No Active Transaction",
    }
}

pub(super) const fn pending_mutation_suffix(count: usize) -> &'static str {
    if count > 1 {
        "s"
    } else {
        ""
    }
}

pub(super) fn transaction_action(
    in_transaction: bool,
    auto_commit: bool,
    rollback_clicked: bool,
    commit_clicked: bool,
    begin_clicked: bool,
) -> Option<TransactionAction> {
    if in_transaction {
        // Preserve the existing visual button order's precedence if two signals
        // are ever reported in one frame: Commit is the final assignment.
        if rollback_clicked {
            if commit_clicked {
                return Some(TransactionAction::Commit);
            }
            return Some(TransactionAction::Rollback);
        }
        return commit_clicked.then_some(TransactionAction::Commit);
    }

    if !auto_commit && begin_clicked {
        return Some(TransactionAction::Begin);
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DestructiveDialogAction {
    None,
    Cancel,
    Confirm,
}

pub(super) fn confirmation_is_valid(input: &str, required_keyword: &str) -> bool {
    input.trim() == required_keyword
}

pub(super) fn destructive_dialog_action(
    cancel_clicked: bool,
    confirm_clicked: bool,
    keyword_valid: bool,
) -> DestructiveDialogAction {
    if confirm_clicked && keyword_valid {
        return DestructiveDialogAction::Confirm;
    }
    if cancel_clicked {
        return DestructiveDialogAction::Cancel;
    }
    DestructiveDialogAction::None
}

pub(super) fn apply_destructive_dialog_action(open: &mut bool, action: DestructiveDialogAction) -> bool {
    match action {
        DestructiveDialogAction::None => false,
        DestructiveDialogAction::Cancel => {
            *open = false;
            false
        }
        DestructiveDialogAction::Confirm => {
            *open = false;
            true
        }
    }
}

#[cfg(test)]
mod tests {
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
}
