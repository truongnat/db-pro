//! UI state for schema comparison, migration planning and cross-connection diff.

use super::schema_compare::{self, UiSchemaDiffResult, UiSchemaSnapshot};
use super::{FeedbackState, UiSchemaSummary};

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

impl SchemaCompareState {
    pub(super) fn take_snapshot(
        &mut self,
        schema: &UiSchemaSummary,
        connection_name: &str,
        feedback: &mut FeedbackState,
    ) {
        let label = format!("{} @ {}", connection_name, chrono::Utc::now().format("%H:%M:%S"));
        self.schema_snapshot = Some(UiSchemaSnapshot::from_summary(label, schema));
        feedback.set_runtime_message("Schema snapshot captured");
    }

    pub(super) fn diff_against_snapshot(&mut self, schema: &UiSchemaSummary, feedback: &mut FeedbackState) {
        let Some(snapshot) = self.schema_snapshot.clone() else {
            feedback.set_runtime_message("Take a schema snapshot before comparing");
            return;
        };
        let current = UiSchemaSnapshot::from_summary("current", schema);
        self.schema_diff = Some(schema_compare::diff_snapshots(&snapshot, &current));
        self.migration_plan = None;
        self.migration_preview_sql.clear();
        self.migration_confirm_destructive = false;
        self.migration_fingerprint_at_preview.clear();
        feedback.set_runtime_message("Schema diff ready");
    }

    pub(super) fn plan_migration(&mut self, driver: &str, feedback: &mut FeedbackState) {
        use db_pro_core::application::MigrationPlanner;

        let Some(diff) = self.schema_diff.clone() else {
            feedback.set_runtime_message("Diff a schema snapshot before planning a migration");
            return;
        };
        let core_diff = schema_compare::to_core_schema_diff(&diff);
        let plan = MigrationPlanner::plan_from_schema_diff(&core_diff, driver);
        self.migration_preview_sql = MigrationPlanner::preview_sql(&plan, true);
        self.migration_fingerprint_at_preview = plan.fingerprint.clone();
        self.migration_confirm_destructive = false;
        self.migration_plan = Some(plan);
        feedback.set_runtime_message("Migration plan ready — review SQL before apply");
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
