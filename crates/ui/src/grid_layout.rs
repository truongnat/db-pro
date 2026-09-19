//! Persist and restore result-grid column layouts per table.
use super::*;

impl DbProApp {
    pub(super) fn grid_layout_scope(&self) -> Option<String> {
        Some(format!(
            "{}|{}|{}",
            self.connection_lifecycle.active_connection_id.as_deref()?,
            self.active_schema(),
            self.schema_explorer.selected_table.as_deref()?
        ))
    }

    pub(super) fn persist_current_grid_layout(&mut self) {
        let Some(scope) = self.grid_layout_scope() else {
            return;
        };
        let column_names = self.table_data.grid_layout_column_names.clone();
        if column_names.len() != self.table_data.grid_column_order.len() {
            return;
        }
        let columns = self
            .table_data
            .grid_column_order
            .iter()
            .enumerate()
            .filter_map(|(order, &column_index)| {
                let column_name = column_names.get(column_index)?.clone();
                Some(PersistedGridColumnLayout {
                    column_name,
                    width: self
                        .table_data
                        .grid_column_widths
                        .get(column_index)
                        .copied()
                        .unwrap_or(180.0),
                    order,
                    hidden: self.table_data.grid_hidden_columns.contains(&column_index),
                })
            })
            .collect();
        self.table_data.grid_layout_preferences.insert(
            scope,
            PersistedGridLayout {
                columns,
                ..PersistedGridLayout::default()
            },
        );
    }

    pub(crate) fn restore_grid_layout_for_active_table(&mut self) {
        let Some(scope) = self.grid_layout_scope() else {
            return;
        };
        let Some(layout) = self.table_data.grid_layout_preferences.get(&scope).cloned() else {
            self.table_data.grid_column_widths.clear();
            self.table_data.grid_column_order.clear();
            self.table_data.grid_hidden_columns.clear();
            self.table_data.grid_pending_named_layout = None;
            self.table_data.grid_legacy_layout_pending = false;
            self.table_data.grid_columns_user_resized = false;
            return;
        };
        self.table_data.grid_layout_column_names.clear();
        self.table_data.grid_pending_named_layout = (!layout.columns.is_empty()).then_some(layout.columns);
        self.table_data.grid_legacy_layout_pending = self.table_data.grid_pending_named_layout.is_none()
            && (!layout.widths.is_empty() || !layout.order.is_empty() || !layout.hidden_columns.is_empty());
        self.table_data.grid_column_widths = layout.widths;
        self.table_data.grid_column_order = layout.order;
        self.table_data.grid_hidden_columns = layout.hidden_columns.into_iter().collect();
        self.table_data.grid_columns_user_resized = !self.table_data.grid_column_widths.is_empty();
    }
}
