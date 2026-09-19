//! UI state for schema comparison, migration planning and cross-connection diff.

use super::schema_compare::{UiSchemaDiffResult, UiSchemaSnapshot};

/// State owned by the schema-comparison workspace and its migration preview.
pub(super) struct SchemaCompareState {
    pub(super) schema_snapshot: Option<UiSchemaSnapshot>,
    pub(super) schema_diff: Option<UiSchemaDiffResult>,
    pub(super) migration_plan: Option<db_pro_core::domain::migration::MigrationPlan>,
    pub(super) migration_preview_sql: String,
    pub(super) migration_confirm_destructive: bool,
    pub(super) migration_fingerprint_at_preview: String,
    pub(super) data_diff_target_id: String,
    pub(super) data_diff_schema: String,
    pub(super) data_diff_table: String,
    pub(super) data_diff_keys: String,
    pub(super) data_diff_result: Option<db_pro_core::domain::cross_connection::DataDiff>,
    pub(super) data_diff_filter: String,
}

impl Default for SchemaCompareState {
    fn default() -> Self {
        Self {
            schema_snapshot: None,
            schema_diff: None,
            migration_plan: None,
            migration_preview_sql: String::new(),
            migration_confirm_destructive: false,
            migration_fingerprint_at_preview: String::new(),
            data_diff_target_id: String::new(),
            data_diff_schema: "public".to_owned(),
            data_diff_table: String::new(),
            data_diff_keys: "id".to_owned(),
            data_diff_result: None,
            data_diff_filter: "all".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SchemaCompareState;

    #[test]
    fn defaults_are_safe_for_schema_change_operations() {
        let state = SchemaCompareState::default();

        assert!(state.schema_snapshot.is_none());
        assert!(state.schema_diff.is_none());
        assert!(state.migration_plan.is_none());
        assert!(!state.migration_confirm_destructive);
        assert_eq!(state.data_diff_schema, "public");
        assert_eq!(state.data_diff_filter, "all");
    }
}
