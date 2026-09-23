use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum QueryTransactionAction {
    Transaction(crate::components::TransactionAction),
    DismissDisconnectGuard,
}

pub(super) struct QueryTransactionSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) execution: &'a mut QueryExecutionPolicyState,
}

impl QueryTransactionSurfaceContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<QueryTransactionAction> {
        let mut actions = Vec::new();
        if self.execution.query_txn_bar_open || self.execution.query_in_transaction {
            ui.add_space(4.0);
            if let Some(action) = TransactionBar::new(
                self.execution.query_in_transaction,
                self.execution.query_txn_pending,
                self.theme,
            )
            .auto_commit(self.execution.query_auto_commit)
            .show(ui)
            {
                actions.push(QueryTransactionAction::Transaction(action));
            }
        }
        if self.execution.disconnect_txn_guard {
            self.draw_disconnect_guard(ui, &mut actions);
        }
        actions
    }

    fn draw_disconnect_guard(&self, ui: &mut egui::Ui, actions: &mut Vec<QueryTransactionAction>) {
        ui.colored_label(
            self.theme.warning,
            "Open transaction blocks disconnect — Commit or Rollback first.",
        );
        if Button::new(self.theme)
            .text("Dismiss")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            actions.push(QueryTransactionAction::DismissDisconnectGuard);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_actions_keep_runtime_effects_explicit() {
        assert_ne!(
            QueryTransactionAction::DismissDisconnectGuard,
            QueryTransactionAction::Transaction(crate::components::TransactionAction::Commit)
        );
    }
}
