//! DDL completion reducer over table and security feature state.

use super::database_feature_states::SecurityState;
use super::{FeedbackState, TableState};
use crate::RequestId;

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct DdlCompletedTransition {
    pub(super) refresh_schema: bool,
    pub(super) refresh_rls: bool,
}

pub(super) fn on_ddl_completed(
    table: &mut TableState,
    security: &mut SecurityState,
    feedback: &mut FeedbackState,
    request_id: RequestId,
    affected_rows: u64,
    has_selected_table: bool,
) -> Option<DdlCompletedTransition> {
    if table.ddl_execution_request != Some(request_id) {
        return None;
    }
    table.ddl_execution_request = None;
    table.ddl_execute_confirmation = false;
    table.table_ddl_error = None;
    table.refresh_table_info_after_schema = has_selected_table;
    feedback.set_runtime_message(format!("DDL applied · {affected_rows} affected rows"));
    let refresh_rls = !security.security_rls_table.trim().is_empty();
    security.security_rls_confirm_apply = false;
    Some(DdlCompletedTransition {
        refresh_schema: true,
        refresh_rls,
    })
}
