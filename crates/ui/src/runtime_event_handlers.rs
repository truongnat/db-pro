use super::*;

impl DbProApp {
    pub(super) fn on_saved_queries_loaded(&mut self, queries: Vec<UiSavedQuerySummary>) {
        query_library_events::on_saved_queries_loaded(&mut self.query.library, queries);
    }

    pub(super) fn on_query_folders_loaded(&mut self, folders: Vec<UiQueryFolderSummary>) {
        query_library_events::on_query_folders_loaded(&mut self.query.library, folders);
    }

    pub(super) fn on_sql_prediction_ready(
        &mut self,
        request_id: RequestId,
        document_id: String,
        document_version: u64,
        anchor: usize,
        replacement_range: (usize, usize),
        prediction_text: String,
    ) {
        query_prediction_events::on_sql_prediction_ready(
            &mut self.query.session,
            request_id,
            document_id,
            document_version,
            anchor,
            replacement_range,
            prediction_text,
        );
    }

    pub(super) fn on_sql_prediction_failed(&mut self, request_id: RequestId, document_id: String) {
        query_prediction_events::on_sql_prediction_failed(&mut self.query.session, request_id, document_id);
    }

    pub(super) fn on_query_saved(&mut self, request_id: RequestId, query: UiSavedQuerySummary) {
        if let Some(index) = query_save_events::on_query_saved(
            &mut self.query.session,
            &mut self.query.library,
            &mut self.feedback,
            request_id,
            query,
        ) {
            self.close_query_document(index);
        }
    }

    pub(super) fn on_explain_completed(&mut self, request_id: RequestId, plan: String) {
        query_execution_events::on_explain_completed(
            &mut self.query.session,
            &mut self.query.output,
            &mut self.workspace.shell,
            &mut self.query.editor,
            &mut self.feedback,
            request_id,
            plan,
        );
    }

    pub(super) fn on_query_cancelled(&mut self, request_id: RequestId) {
        query_execution_events::on_query_cancelled(
            &mut self.query.session,
            &mut self.query.editor,
            &mut self.feedback,
            request_id,
        );
    }

    pub(super) fn on_query_completed(&mut self, request_id: RequestId, result: UiQueryResult) {
        let mut context = query_result_events::QueryResultContext {
            query_session: &mut self.query.session,
            query_editor: &mut self.query.editor,
            query_output: &mut self.query.output,
            table_data: &mut self.table.data,
            workspace: &mut self.workspace.shell,
            feedback: &mut self.feedback,
        };
        query_result_events::on_query_completed(&mut context, request_id, result);
    }

    pub(super) fn on_query_multi_completed(&mut self, request_id: RequestId, output: UiQueryExecutionOutput) {
        let mut context = query_multi_result_events::QueryMultiResultContext {
            query_session: &mut self.query.session,
            query_editor: &mut self.query.editor,
            query_output: &mut self.query.output,
            table_data: &mut self.table.data,
            workspace: &mut self.workspace.shell,
            feedback: &mut self.feedback,
        };
        query_multi_result_events::on_query_multi_completed(&mut context, request_id, output);
    }

    pub(super) fn on_query_failed(
        &mut self,
        request_id: RequestId,
        message: String,
        position: Option<usize>,
        code: Option<String>,
    ) {
        if self.handle_agent_request_failure(request_id, &message)
            || self.handle_connection_request_failure(request_id, &message)
            || self.handle_schema_request_failure(request_id, &message)
            || self.handle_table_request_failure(request_id, &message)
        {
            return;
        }
        let mut context = query_failure_events::QueryFailureContext {
            query_session: &mut self.query.session,
            query_editor: &mut self.query.editor,
            query_output: &mut self.query.output,
            workspace: &mut self.workspace.shell,
            feedback: &mut self.feedback,
        };
        query_failure_events::on_query_failed(&mut context, request_id, message, position, code);
    }

    pub(super) fn on_query_queued(&mut self, request_id: RequestId) {
        query_queue_events::on_query_queued(&mut self.feedback, request_id);
    }

    pub(super) fn handle_schema_request_failure(&mut self, request_id: RequestId, message: &str) -> bool {
        schema_events::handle_schema_request_failure(&mut self.schema_explorer, &mut self.feedback, request_id, message)
    }

    pub(super) fn on_schema_loaded(&mut self, request_id: RequestId, schema: UiSchemaSummary) {
        let transition = schema_events::on_schema_loaded(
            &mut self.schema_explorer,
            &mut self.table.state,
            &mut self.workspace.shell,
            &mut self.palette,
            &mut self.feedback,
            request_id,
            schema,
        );
        if transition.refresh_selected_table {
            self.request_table_info();
        }
    }

    pub(super) fn handle_agent_request_failure(&mut self, request_id: RequestId, message: &str) -> bool {
        agent_events::handle_agent_request_failure(&mut self.agent, &mut self.feedback, request_id, message)
    }

    pub(super) fn on_agent_provider_ready(&mut self, provider: String, detail: String) {
        agent_events::on_agent_provider_ready(&mut self.agent, provider, detail);
    }

    pub(super) fn on_agent_failed(&mut self, request_id: RequestId, message: String) {
        agent_events::on_agent_failed(&mut self.agent, &mut self.feedback, request_id, message);
    }

    pub(super) fn on_agent_configured(&mut self, request_id: RequestId, provider: String, detail: String) {
        agent_events::on_agent_configured(&mut self.agent, &mut self.feedback, request_id, provider, detail);
    }

    pub(super) fn on_agent_forgotten(&mut self, request_id: RequestId) {
        agent_events::on_agent_forgotten(&mut self.agent, &mut self.feedback, request_id);
    }

    pub(super) fn handle_table_request_failure(&mut self, request_id: RequestId, message: &str) -> bool {
        match table_events::handle_table_request_failure(
            &mut self.table.state,
            &mut self.table.mutation,
            &mut self.table.data,
            &mut self.feedback,
            request_id,
            message,
        ) {
            Some(table_events::TableFailureTransition::StagedApplyFailed) => {
                self.staged_apply_failed(usize::MAX, "UNKNOWN", message, false);
                true
            }
            Some(table_events::TableFailureTransition::Handled) => true,
            None => false,
        }
    }

    pub(super) fn on_table_info_loaded(&mut self, request_id: RequestId, table_info: UiTableInfo) {
        if let Some(transition) =
            table_events::on_table_info_loaded(&mut self.table.state, &mut self.feedback, request_id, table_info)
        {
            if transition.invalidate_grid_caches {
                self.table.data.invalidate_grid_row_caches();
            }
        }
    }

    pub(super) fn on_table_ddl_loaded(&mut self, request_id: RequestId, sql: String) {
        table_events::on_table_ddl_loaded(&mut self.table.state, &mut self.feedback, request_id, sql);
    }

    pub(super) fn on_table_data_loaded(&mut self, request_id: RequestId, result: UiQueryResult, total_rows: u64) {
        if self.table.state.table_row_reload_request == Some(request_id) {
            self.on_table_row_reloaded(result);
            return;
        }
        if let Some(transition) = table_events::on_table_data_loaded(
            &mut self.table.state,
            &mut self.table.mutation,
            &mut self.table.data,
            &mut self.feedback,
            request_id,
            result,
            total_rows,
        ) {
            if transition.invalidate_grid_caches {
                self.table.data.invalidate_grid_row_caches();
            }
            if transition.apply_staged_changes {
                self.table.mutation.table_mutation_retry_after_reload = false;
                self.apply_staged_changes();
            }
        }
    }

    pub(crate) fn on_table_row_reloaded(&mut self, result: UiQueryResult) {
        let transition = table_events::on_table_row_reloaded(
            &mut self.table.state,
            &mut self.table.mutation,
            &mut self.feedback,
            result,
        );
        if transition.invalidate_grid_caches {
            self.table.data.invalidate_grid_row_caches();
        }
        if transition.apply_staged_changes {
            self.table.mutation.table_mutation_retry_after_reload = false;
            self.apply_staged_changes();
        }
    }
}
