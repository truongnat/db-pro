//! Feature-owned state for table metadata, paging and request lifecycle.

use super::*;

#[derive(Debug)]
pub(crate) struct TableState {
    pub(crate) table_info: Option<UiTableInfo>,
    pub(crate) table_info_error: Option<String>,
    pub(crate) table_ddl_error: Option<String>,
    pub(crate) ddl_execute_confirmation: bool,
    pub(crate) ddl_execution_request: Option<crate::RequestId>,
    pub(crate) refresh_table_info_after_schema: bool,
    pub(crate) table_data_result: Option<UiQueryResult>,
    pub(crate) table_data_total_rows: Option<u64>,
    pub(crate) table_data_offset: u64,
    pub(crate) table_data_limit: u64,
    pub(crate) table_data_filter_column: String,
    pub(crate) table_data_filter_operator: UiTableFilterOperator,
    pub(crate) table_data_filter_value: String,
    pub(crate) table_data_filter_editing: Option<usize>,
    pub(crate) table_data_filters: Vec<UiTableDataFilter>,
    pub(crate) table_data_sorts: Vec<UiTableDataSort>,
    pub(crate) table_data_error: Option<String>,
    pub(crate) table_structure_search: String,
    pub(crate) table_metadata_search: String,
    pub(crate) table_column_detail: Option<String>,
    pub(crate) table_index_detail: Option<String>,
    pub(crate) table_dependency_filter: String,
    pub(crate) table_constraint_filter: String,
    pub(crate) table_info_request: Option<crate::RequestId>,
    pub(crate) table_ddl_request: Option<crate::RequestId>,
    pub(crate) table_data_request: Option<crate::RequestId>,
    pub(crate) table_row_reload_request: Option<crate::RequestId>,
    pub(crate) table_row_reload_identity: Option<RowIdentity>,
    pub(crate) table_view: TableView,
}

impl Default for TableState {
    fn default() -> Self {
        Self {
            table_info: None,
            table_info_error: None,
            table_ddl_error: None,
            ddl_execute_confirmation: false,
            ddl_execution_request: None,
            refresh_table_info_after_schema: false,
            table_data_result: None,
            table_data_total_rows: None,
            table_data_offset: 0,
            table_data_limit: TABLE_PAGE_SIZE,
            table_data_filter_column: String::new(),
            table_data_filter_operator: UiTableFilterOperator::default(),
            table_data_filter_value: String::new(),
            table_data_filter_editing: None,
            table_data_filters: Vec::new(),
            table_data_sorts: Vec::new(),
            table_data_error: None,
            table_structure_search: String::new(),
            table_metadata_search: String::new(),
            table_column_detail: None,
            table_index_detail: None,
            table_dependency_filter: String::new(),
            table_constraint_filter: String::new(),
            table_info_request: None,
            table_ddl_request: None,
            table_data_request: None,
            table_row_reload_request: None,
            table_row_reload_identity: None,
            table_view: TableView::Structure,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_table_state_starts_at_structure_without_requests() {
        let state = TableState::default();

        assert_eq!(state.table_view, TableView::Structure);
        assert!(state.table_info.is_none());
        assert!(state.table_data_result.is_none());
        assert!(state.table_info_request.is_none());
        assert_eq!(state.table_data_limit, TABLE_PAGE_SIZE);
    }
}
