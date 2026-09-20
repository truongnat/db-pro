//! State owned by the routine-management surface.

#[derive(Default)]
pub(super) struct RoutineState {
    pub(super) routine_source_draft: String,
    pub(super) routine_param_values: Vec<String>,
    pub(super) routine_param_nulls: Vec<bool>,
    pub(super) routine_ddl_preview: Option<String>,
    pub(super) routine_drop_confirm: bool,
}
