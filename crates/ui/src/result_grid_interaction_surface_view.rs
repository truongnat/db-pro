//! Result-grid keyboard and edit intent collection.

use super::result_grid_keyboard_view::{self, GridKeyboardInputContext};
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResultGridInteractionAction {
    CopySelectedRows,
    CopySelectedCell,
    ApplyStagedChanges,
    DiscardStagedChanges,
    DeleteSelectedRows,
    SubmitCellEdit { row_index: usize, column_index: usize },
    BeginCellEdit { row_index: usize, column_index: usize },
    CommitEditAndNavigate,
    Navigate,
}

pub(crate) struct ResultGridInteractionContext<'a> {
    pub(crate) result: &'a UiQueryResult,
    pub(crate) indexes: &'a [usize],
    pub(crate) order: &'a [usize],
    pub(crate) editable: bool,
    pub(crate) table_state: &'a TableState,
    pub(crate) table_data: &'a mut TableDataState,
    pub(crate) table_editing: &'a mut TableEditingState,
    pub(crate) table_mutation: &'a TableMutationState,
    pub(crate) feedback: &'a mut FeedbackState,
}

impl ResultGridInteractionContext<'_> {
    pub(crate) fn handle_keyboard(
        &mut self,
        ui: &mut egui::Ui,
        connection_dialog_open: bool,
    ) -> Vec<ResultGridInteractionAction> {
        let intent = result_grid_keyboard_view::read_keyboard_intent(
            ui,
            GridKeyboardInputContext {
                editable: self.editable,
                editing_cell: self.table_editing.data_editing_cell.is_some(),
                connection_dialog_open,
            },
        );
        match intent {
            result_grid_keyboard_view::GridKeyboardIntent::SelectAll => {
                self.table_data.select_all_visible_cells(self.indexes, self.order);
                self.feedback.copy_status.clear();
                Vec::new()
            }
            result_grid_keyboard_view::GridKeyboardIntent::ClearSelection => {
                self.table_data.clear_selection();
                self.feedback.copy_status.clear();
                Vec::new()
            }
            result_grid_keyboard_view::GridKeyboardIntent::Commands(commands) => {
                self.collect_command_actions(ui, commands)
            }
        }
    }

    fn collect_command_actions(
        &mut self,
        ui: &mut egui::Ui,
        commands: result_grid_keyboard_view::GridKeyboardCommands,
    ) -> Vec<ResultGridInteractionAction> {
        let mut actions = Vec::new();
        if commands.copy_selected_rows {
            actions.push(ResultGridInteractionAction::CopySelectedRows);
        } else if commands.copy_selected_cell {
            actions.push(ResultGridInteractionAction::CopySelectedCell);
        }
        if commands.apply_staged_changes {
            actions.push(ResultGridInteractionAction::ApplyStagedChanges);
        }
        if commands.discard_staged_changes {
            if self.table_mutation.staged_changes.counts().total() > 1 {
                self.table_editing.discard_changes_confirmation = true;
            } else {
                actions.push(ResultGridInteractionAction::DiscardStagedChanges);
            }
        }
        if commands.delete_selected_rows {
            actions.push(ResultGridInteractionAction::DeleteSelectedRows);
        }
        if self.editable {
            actions.extend(self.collect_edit_actions(ui, commands.pasted_text));
        }
        if commands.commit_edit_and_navigate {
            actions.push(ResultGridInteractionAction::CommitEditAndNavigate);
            return actions;
        }
        if commands.navigate {
            actions.push(ResultGridInteractionAction::Navigate);
        }
        actions
    }

    fn collect_edit_actions(
        &mut self,
        ui: &mut egui::Ui,
        pasted_text: Option<String>,
    ) -> Vec<ResultGridInteractionAction> {
        let mut actions = Vec::new();
        if let (Some((row_index, column_index)), Some(text)) = (self.table_data.selected_cell, pasted_text) {
            if let Some(block) = self
                .result
                .columns
                .get(column_index)
                .and_then(|column| self.table_state.column_write_block(&column.name))
            {
                self.feedback.copy_status = block.reason().to_owned();
                return actions;
            }
            self.table_editing.data_editing_cell = Some((row_index, column_index));
            self.table_editing.data_edit_value = text;
            actions.push(ResultGridInteractionAction::SubmitCellEdit {
                row_index,
                column_index,
            });
        }
        if self.table_editing.data_editing_cell.is_none()
            && self.table_data.selected_cell.is_some()
            && ui.input(|input| input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::F2))
        {
            if let Some((row_index, column_index)) = self.table_data.selected_cell {
                if self
                    .result
                    .rows
                    .get(row_index)
                    .and_then(|row| row.get(column_index))
                    .is_some()
                {
                    actions.push(ResultGridInteractionAction::BeginCellEdit {
                        row_index,
                        column_index,
                    });
                }
            }
        }
        actions
    }
}
