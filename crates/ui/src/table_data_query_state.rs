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
