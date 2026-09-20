//! Feature-owned state for table metadata, paging and request lifecycle.

use super::*;

#[derive(Debug)]
pub(crate) struct TableState {
    pub(super) table_info: Option<UiTableInfo>,
    pub(super) table_ddl: Option<String>,
    pub(super) table_info_error: Option<String>,
    pub(super) table_ddl_error: Option<String>,
    pub(super) ddl_execute_confirmation: bool,
    pub(super) ddl_execution_request: Option<crate::RequestId>,
    pub(super) refresh_table_info_after_schema: bool,
    pub(super) table_structure_search: String,
    pub(super) table_metadata_search: String,
    pub(super) table_column_detail: Option<String>,
    pub(super) table_index_detail: Option<String>,
    pub(super) table_dependency_filter: String,
    pub(super) table_constraint_filter: String,
    pub(super) table_info_request: Option<crate::RequestId>,
    pub(super) table_ddl_request: Option<crate::RequestId>,
    pub(super) table_view: TableView,
}

impl Default for TableState {
    fn default() -> Self {
        Self {
            table_info: None,
            table_ddl: None,
            table_info_error: None,
            table_ddl_error: None,
            ddl_execute_confirmation: false,
            ddl_execution_request: None,
            refresh_table_info_after_schema: false,
            table_structure_search: String::new(),
            table_metadata_search: String::new(),
            table_column_detail: None,
            table_index_detail: None,
            table_dependency_filter: String::new(),
            table_constraint_filter: String::new(),
            table_info_request: None,
            table_ddl_request: None,
            table_view: TableView::Structure,
        }
    }
}

impl TableState {
    pub(super) fn load_info_command(
        &self,
        request_id: RequestId,
        connection_id: String,
        schema: String,
        table: String,
    ) -> UiCommand {
        UiCommand::LoadTableInfo {
            request_id,
            connection_id,
            schema,
            table,
        }
    }

    pub(super) fn load_ddl_command(
        &self,
        request_id: RequestId,
        connection_id: String,
        schema: String,
        table: String,
    ) -> UiCommand {
        UiCommand::LoadTableDdl {
            request_id,
            connection_id,
            schema,
            table,
        }
    }

    pub(super) fn execute_ddl_command(
        &self,
        request_id: RequestId,
        connection_id: String,
    ) -> Result<UiCommand, String> {
        let sql = self
            .table_ddl
            .as_deref()
            .ok_or_else(|| "Load the table DDL before executing it".to_owned())?
            .trim();
        if sql.is_empty() {
            return Err("DDL cannot be empty".to_owned());
        }
        Ok(UiCommand::ExecuteDdl {
            request_id,
            connection_id,
            sql: sql.to_owned(),
        })
    }

    pub(crate) fn has_primary_key(&self) -> bool {
        self.table_info
            .as_ref()
            .is_some_and(|info| info.primary_key.as_ref().is_some_and(|columns| !columns.is_empty()))
    }

    pub(crate) fn column_write_policy(&self, column_name: &str) -> Option<ColumnWritePolicy> {
        self.table_info
            .as_ref()?
            .columns
            .iter()
            .find(|column| column.name == column_name)
            .map(ColumnWritePolicy::read)
    }

    pub(crate) fn column_write_block(&self, column_name: &str) -> Option<ColumnWriteBlock> {
        self.column_write_policy(column_name)
            .and_then(|policy| policy.write_block())
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
        assert!(state.table_info_request.is_none());
    }

    #[test]
    fn execute_ddl_requires_non_empty_loaded_sql() {
        let state = TableState::default();

        assert_eq!(
            state.execute_ddl_command(RequestId(1), "source".to_owned()),
            Err("Load the table DDL before executing it".to_owned())
        );
    }
}
