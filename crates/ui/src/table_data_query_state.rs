//! Request and read-model state for the table data surface.
//!
//! Table metadata/DDL and table data are different lifecycles. Keeping paging,
//! filters and data requests together prevents metadata reducers from owning
//! query concerns and gives the data surface one explicit state boundary.

use super::*;

#[derive(Debug)]
pub(crate) struct TableDataQueryState {
    pub(super) result: Option<UiQueryResult>,
    pub(super) total_rows: Option<u64>,
    pub(super) offset: u64,
    pub(super) limit: u64,
    pub(super) filter_column: String,
    pub(super) filter_operator: UiTableFilterOperator,
    pub(super) filter_value: String,
    pub(super) filter_editing: Option<usize>,
    pub(super) filters: Vec<UiTableDataFilter>,
    pub(super) sorts: Vec<UiTableDataSort>,
    pub(super) error: Option<String>,
    pub(super) request: Option<RequestId>,
    pub(super) row_reload_request: Option<RequestId>,
    pub(super) row_reload_identity: Option<RowIdentity>,
}

impl Default for TableDataQueryState {
    fn default() -> Self {
        Self {
            result: None,
            total_rows: None,
            offset: 0,
            limit: TABLE_PAGE_SIZE,
            filter_column: String::new(),
            filter_operator: UiTableFilterOperator::default(),
            filter_value: String::new(),
            filter_editing: None,
            filters: Vec::new(),
            sorts: Vec::new(),
            error: None,
            request: None,
            row_reload_request: None,
            row_reload_identity: None,
        }
    }
}

impl TableDataQueryState {
    pub(crate) fn reset_for_table(&mut self) {
        self.result = None;
        self.total_rows = None;
        self.offset = 0;
        self.filter_column.clear();
        self.filter_operator = UiTableFilterOperator::default();
        self.filter_value.clear();
        self.filter_editing = None;
        self.filters.clear();
        self.sorts.clear();
        self.error = None;
        self.request = None;
        self.row_reload_request = None;
        self.row_reload_identity = None;
    }

    pub(crate) fn invalidate_result(&mut self) {
        self.result = None;
        self.total_rows = None;
        self.error = None;
        self.request = None;
    }

    pub(crate) fn reset_page(&mut self) {
        self.offset = 0;
    }

    pub(crate) fn filter_operator_supported(data_type: &str, operator: &UiTableFilterOperator) -> bool {
        Self::filter_operator_options(data_type)
            .iter()
            .any(|(candidate, _)| candidate == operator)
    }

    pub(crate) fn filter_operator_options(data_type: &str) -> Vec<(UiTableFilterOperator, &'static str)> {
        let normalized = data_type.to_ascii_lowercase();
        let mut operators = if table_editor_values::is_text_type(&normalized) {
            vec![
                (UiTableFilterOperator::Equals, "equals"),
                (UiTableFilterOperator::NotEquals, "not equals"),
                (UiTableFilterOperator::Contains, "contains"),
                (UiTableFilterOperator::StartsWith, "starts with"),
                (UiTableFilterOperator::EndsWith, "ends with"),
            ]
        } else if normalized.contains("bool") {
            vec![
                (UiTableFilterOperator::Equals, "equals"),
                (UiTableFilterOperator::NotEquals, "not equals"),
            ]
        } else if normalized.contains("int")
            || normalized.contains("serial")
            || normalized.contains("real")
            || normalized.contains("float")
            || normalized.contains("double")
            || table_editor_values::is_decimal_type(&normalized)
            || normalized == "date"
            || normalized.starts_with("time")
            || normalized.contains("timestamp")
        {
            vec![
                (UiTableFilterOperator::Equals, "equals"),
                (UiTableFilterOperator::NotEquals, "not equals"),
                (UiTableFilterOperator::GreaterThan, ">"),
                (UiTableFilterOperator::GreaterThanOrEqual, ">="),
                (UiTableFilterOperator::LessThan, "<"),
                (UiTableFilterOperator::LessThanOrEqual, "<="),
            ]
        } else {
            vec![
                (UiTableFilterOperator::Equals, "equals"),
                (UiTableFilterOperator::NotEquals, "not equals"),
            ]
        };
        operators.push((UiTableFilterOperator::IsNull, "IS NULL"));
        operators.push((UiTableFilterOperator::IsNotNull, "IS NOT NULL"));
        operators
    }

    pub(crate) fn commit_filter_draft(&mut self, table_info: Option<&UiTableInfo>) -> Result<bool, String> {
        let column = self.filter_column.trim();
        if column.is_empty() {
            return Ok(false);
        }
        let is_null_operator = matches!(
            self.filter_operator,
            UiTableFilterOperator::IsNull | UiTableFilterOperator::IsNotNull
        );
        let data_type = table_info
            .and_then(|info| info.columns.iter().find(|item| item.name == column))
            .map(|item| item.data_type.clone())
            .unwrap_or_else(|| "text".to_owned());
        if !Self::filter_operator_supported(&data_type, &self.filter_operator) {
            return Err(format!("That filter operator is not supported for {data_type}"));
        }
        if !is_null_operator
            && self.filter_value.is_empty()
            && !table_editor_values::is_text_type(&data_type.to_ascii_lowercase())
        {
            return Err("Enter a filter value first".to_owned());
        }
        if !is_null_operator {
            table_editor_values::parse_update_value(&self.filter_value, &data_type)
                .map_err(|error| format!("Invalid filter for {column}: {error}"))?;
        }
        let filter = UiTableDataFilter {
            column: column.to_owned(),
            data_type,
            operator: self.filter_operator.clone(),
            value: if is_null_operator {
                String::new()
            } else {
                self.filter_value.clone()
            },
        };
        self.replace_or_append_filter(filter);
        Ok(true)
    }

    pub(crate) fn remove_filter(&mut self, index: usize) -> bool {
        if index >= self.filters.len() {
            return false;
        }
        self.filters.remove(index);
        self.filter_editing = match self.filter_editing {
            Some(editing) if editing == index => None,
            Some(editing) if editing > index => Some(editing - 1),
            other => other,
        };
        true
    }

    pub(crate) fn clear_filters(&mut self) {
        self.filters.clear();
        self.filter_editing = None;
    }

    pub(crate) fn set_sort(&mut self, column: Option<String>, descending: Option<bool>) {
        self.sorts = match (column, descending) {
            (Some(column), Some(descending)) => vec![UiTableDataSort { column, descending }],
            _ => Vec::new(),
        };
    }

    pub(crate) fn cycle_sort(&mut self, column: String, additive: bool) {
        if additive {
            self.cycle_additive_sort(column);
            return;
        }
        if self.sorts.len() == 1 && self.sorts.first().is_some_and(|sort| sort.column == column) {
            if self.sorts[0].descending {
                self.sorts.clear();
            } else {
                self.sorts[0].descending = true;
            }
            return;
        }
        self.set_sort(Some(column), Some(false));
    }

    fn cycle_additive_sort(&mut self, column: String) {
        if let Some(index) = self.sorts.iter().position(|sort| sort.column == column) {
            if self.sorts[index].descending {
                self.sorts.remove(index);
            } else {
                self.sorts[index].descending = true;
            }
        } else {
            self.sorts.push(UiTableDataSort {
                column,
                descending: false,
            });
        }
    }

    fn replace_or_append_filter(&mut self, filter: UiTableDataFilter) {
        if let Some(index) = self.filter_editing.take() {
            if let Some(existing) = self.filters.get_mut(index) {
                *existing = filter;
                return;
            }
        }
        self.filters.push(filter);
    }

    pub(super) fn load_data_command(
        &self,
        request_id: RequestId,
        connection_id: String,
        schema: String,
        table: String,
    ) -> UiCommand {
        UiCommand::LoadTableData {
            request_id,
            connection_id,
            schema,
            table,
            limit: self.limit,
            offset: self.offset,
            filters: self.filters.clone(),
            sorts: self.sorts.clone(),
        }
    }

    pub(super) fn load_row_command(
        &self,
        request_id: RequestId,
        connection_id: String,
        schema: String,
        table: String,
        filters: Vec<UiTableDataFilter>,
    ) -> UiCommand {
        UiCommand::LoadTableData {
            request_id,
            connection_id,
            schema,
            table,
            limit: 1,
            offset: 0,
            filters,
            sorts: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UiTableColumn;

    fn table_info(data_type: &str) -> UiTableInfo {
        UiTableInfo {
            schema: "public".to_owned(),
            name: "orders".to_owned(),
            row_count: None,
            columns: vec![UiTableColumn {
                name: "amount".to_owned(),
                data_type: data_type.to_owned(),
                ..Default::default()
            }],
            primary_key: None,
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            check_constraints: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    #[test]
    fn commit_filter_draft_validates_typed_values_before_mutating_filters() {
        let mut state = TableDataQueryState {
            filter_column: "amount".to_owned(),
            filter_operator: UiTableFilterOperator::GreaterThan,
            filter_value: "10.00".to_owned(),
            ..Default::default()
        };

        assert_eq!(state.commit_filter_draft(Some(&table_info("numeric(12,2)"))), Ok(true));
        assert_eq!(state.filters.len(), 1);
        assert_eq!(state.filters[0].value, "10.00");

        state.filter_value = "not-a-number".to_owned();
        assert!(state.commit_filter_draft(Some(&table_info("numeric(12,2)"))).is_err());
        assert_eq!(state.filters[0].value, "10.00");
    }

    #[test]
    fn commit_filter_draft_replaces_the_selected_filter_and_normalizes_null_values() {
        let mut state = TableDataQueryState {
            filter_column: "amount".to_owned(),
            filter_operator: UiTableFilterOperator::Equals,
            filter_value: "10".to_owned(),
            ..Default::default()
        };
        assert_eq!(state.commit_filter_draft(Some(&table_info("integer"))), Ok(true));

        state.filter_editing = Some(0);
        state.filter_operator = UiTableFilterOperator::IsNull;
        state.filter_value = "ignored".to_owned();
        assert_eq!(state.commit_filter_draft(Some(&table_info("integer"))), Ok(true));
        assert_eq!(state.filters.len(), 1);
        assert_eq!(state.filters[0].value, "");
        assert_eq!(state.filters[0].operator, UiTableFilterOperator::IsNull);
    }

    #[test]
    fn removing_filters_keeps_edit_index_consistent() {
        let mut state = TableDataQueryState {
            filters: vec![
                UiTableDataFilter {
                    column: "a".to_owned(),
                    data_type: "text".to_owned(),
                    operator: UiTableFilterOperator::Equals,
                    value: "one".to_owned(),
                },
                UiTableDataFilter {
                    column: "b".to_owned(),
                    data_type: "text".to_owned(),
                    operator: UiTableFilterOperator::Equals,
                    value: "two".to_owned(),
                },
            ],
            filter_editing: Some(1),
            ..Default::default()
        };

        assert!(state.remove_filter(0));
        assert_eq!(state.filter_editing, Some(0));
        assert!(!state.remove_filter(9));
    }

    #[test]
    fn sort_transitions_preserve_single_and_additive_cycle_semantics() {
        let mut state = TableDataQueryState::default();

        state.set_sort(Some("amount".to_owned()), Some(false));
        state.cycle_sort("amount".to_owned(), false);
        assert!(state.sorts[0].descending);
        state.cycle_sort("amount".to_owned(), false);
        assert!(state.sorts.is_empty());

        state.cycle_sort("amount".to_owned(), true);
        state.cycle_sort("created_at".to_owned(), true);
        assert_eq!(state.sorts.len(), 2);
        state.cycle_sort("amount".to_owned(), true);
        assert!(state.sorts[0].descending);
    }
}
