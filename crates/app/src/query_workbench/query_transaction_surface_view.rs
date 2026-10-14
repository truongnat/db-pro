use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum QueryTransactionAction {
    Transaction(crate::components::TransactionAction),
    DismissDisconnectGuard,
}

pub(super) struct QueryTransactionSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) execution: &'a mut QueryExecutionPolicyState,
    pub(super) document_id: Option<&'a str>,
}

impl QueryTransactionSurfaceContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<QueryTransactionAction> {
        let mut actions = Vec::new();
        let in_transaction = self.execution.in_transaction(self.document_id);
        if self.execution.query_txn_bar_open || in_transaction {
            ui.add_space(4.0);
            if let Some(action) = TransactionBar::new(in_transaction, self.execution.txn_pending(self.document_id), self.theme)
                .auto_commit(self.execution.auto_commit(self.document_id))
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
#[path = "tests/query_transaction_surface_view_tests.rs"]
mod tests;

