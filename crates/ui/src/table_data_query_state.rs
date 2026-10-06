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
    pub(super) inline_query_request: Option<RequestId>,
    pub(super) pending_inline_query_confirmation: bool,
    pub(super) inline_query_result: bool,
    pub(super) row_reload_request: Option<RequestId>,
    pub(super) row_reload_identity: Option<RowIdentity>,
    pub(super) sql_condition_draft: String,
    pub(super) sql_condition_suggestions_open: bool,
    pub(super) sql_condition_suggestion_index: usize,
    pub(super) sql_condition_suggestions_dismissed_for: Option<String>,
    pub(super) active_inline_query_sql: Option<String>,
    pub(super) pending_inline_query_sql: Option<String>,
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
            inline_query_request: None,
            pending_inline_query_confirmation: false,
            inline_query_result: false,
            row_reload_request: None,
            row_reload_identity: None,
            sql_condition_draft: String::new(),
            sql_condition_suggestions_open: false,
            sql_condition_suggestion_index: 0,
            sql_condition_suggestions_dismissed_for: None,
            active_inline_query_sql: None,
            pending_inline_query_sql: None,
        }
    }
}

impl TableDataQueryState {
    pub(crate) fn begin_inline_query(&mut self, request_id: RequestId, sql: String) {
        self.inline_query_request = Some(request_id);
        self.pending_inline_query_sql = Some(sql);
    }

    pub(crate) fn complete_inline_query(&mut self, request_id: RequestId) -> bool {
        if self.inline_query_request != Some(request_id) {
            return false;
        }
        self.inline_query_request = None;
        if let Some(sql) = self.pending_inline_query_sql.take() {
            self.active_inline_query_sql = Some(sql);
        }
        self.inline_query_result = true;
        true
    }

    pub(crate) fn abandon_inline_query(&mut self, request_id: RequestId) -> bool {
        if self.inline_query_request != Some(request_id) {
            return false;
        }
        self.inline_query_request = None;
        self.pending_inline_query_sql = None;
        true
    }

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
        self.inline_query_request = None;
        self.pending_inline_query_confirmation = false;
        self.inline_query_result = false;
        self.row_reload_request = None;
        self.row_reload_identity = None;
        self.sql_condition_draft.clear();
        self.sql_condition_suggestions_open = false;
        self.sql_condition_suggestion_index = 0;
        self.sql_condition_suggestions_dismissed_for = None;
        self.active_inline_query_sql = None;
        self.pending_inline_query_sql = None;
    }

    pub(crate) fn invalidate_result(&mut self) {
        self.result = None;
        self.total_rows = None;
        self.error = None;
        self.request = None;
        self.inline_query_request = None;
        self.pending_inline_query_sql = None;
        self.inline_query_result = false;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RequestId, UiTableColumn};

    #[test]
    fn keeps_last_successful_inline_sql_until_another_query_succeeds_or_table_resets() {
        let mut state = TableDataQueryState::default();
        let first_sql = "SELECT * FROM customers WHERE id = 1".to_owned();
        state.begin_inline_query(RequestId(1), first_sql.clone());
        assert!(state.complete_inline_query(RequestId(1)));
        assert_eq!(state.active_inline_query_sql.as_deref(), Some(first_sql.as_str()));

        state.begin_inline_query(RequestId(2), "SELECT * FROM customers WHERE id = 2".to_owned());
        assert!(!state.complete_inline_query(RequestId(3)));
        assert!(state.abandon_inline_query(RequestId(2)));
        assert_eq!(state.active_inline_query_sql.as_deref(), Some(first_sql.as_str()));

        state.reset_for_table();
        assert!(state.active_inline_query_sql.is_none());
        assert!(state.pending_inline_query_sql.is_none());
    }

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
