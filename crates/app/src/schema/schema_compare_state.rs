//! UI state for schema comparison, migration planning and cross-connection diff.

use super::schema_compare::{self, UiSchemaDiffResult, UiSchemaSnapshot};
use super::{FeedbackState, UiSchemaSummary};

pub(super) struct DataDiffRequest {
    pub(super) target_id: String,
    pub(super) schema: String,
    pub(super) table: String,
    pub(super) key_columns: Vec<String>,
    pub(super) sample_limit: Option<u64>,
}

/// State owned by the schema-comparison workspace and its migration preview.
pub(super) struct SchemaCompareState {
    pub(super) schema_snapshot: Option<UiSchemaSnapshot>,
    /// Live schema captured with the diff. Planning reads its columns instead of a name-only diff.
    pub(super) schema_diff_current: Option<UiSchemaSnapshot>,
    pub(super) schema_diff: Option<UiSchemaDiffResult>,
    pub(super) migration_plan: Option<db_pro_core::domain::migration::MigrationPlan>,
    pub(super) migration_preview_sql: String,
    pub(super) migration_confirm_destructive: bool,
    pub(super) migration_fingerprint_at_preview: String,
    /// Connection·schema the migration plan was generated against (spec 09 F1
    /// target binding). Apply is blocked when the active session diverges.
    pub(super) migration_plan_target: Option<String>,
    pub(super) data_diff_target_id: String,
    pub(super) data_diff_schema: String,
    pub(super) data_diff_table: String,
    pub(super) data_diff_keys: String,
    pub(super) data_diff_result: Option<db_pro_core::domain::cross_connection::DataDiff>,
    pub(super) data_diff_request: Option<(u64, String, String)>,
    pub(super) data_diff_filter: String,
}

impl Default for SchemaCompareState {
    fn default() -> Self {
        Self {
            schema_snapshot: None,
            schema_diff_current: None,
            schema_diff: None,
            migration_plan: None,
            migration_preview_sql: String::new(),
            migration_confirm_destructive: false,
            migration_fingerprint_at_preview: String::new(),
            migration_plan_target: None,
            data_diff_target_id: String::new(),
            data_diff_schema: "public".to_owned(),
            data_diff_table: String::new(),
            data_diff_keys: "id".to_owned(),
            data_diff_result: None,
            data_diff_request: None,
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
        self.schema_diff_current = Some(current);
        self.migration_plan = None;
        self.migration_preview_sql.clear();
        self.migration_confirm_destructive = false;
        self.migration_fingerprint_at_preview.clear();
        self.migration_plan_target = None;
        feedback.set_runtime_message("Schema diff ready");
    }

    pub(super) fn plan_migration(
        &mut self,
        driver: &str,
        target: &str,
        feedback: &mut FeedbackState,
    ) {
        use db_pro_core::application::MigrationPlanner;

        let (Some(source_snapshot), Some(target_snapshot)) =
            (self.schema_snapshot.clone(), self.schema_diff_current.clone())
        else {
            feedback.set_runtime_message("Diff a schema snapshot before planning a migration");
            return;
        };
        let plan = MigrationPlanner::plan_from_introspection(
            &schema_compare::snapshot_to_introspect(&source_snapshot),
            &schema_compare::snapshot_to_introspect(&target_snapshot),
            driver,
        );
        self.migration_preview_sql = MigrationPlanner::preview_sql(&plan, true);
        self.migration_fingerprint_at_preview = plan.fingerprint.clone();
        self.migration_confirm_destructive = false;
        self.migration_plan_target = Some(target.to_owned());
        self.migration_plan = Some(plan);
        feedback.set_runtime_message("Migration plan ready — review SQL before apply");
    }

    pub(super) fn prepare_data_diff_request(&self) -> Result<DataDiffRequest, String> {
        let target_id = self.data_diff_target_id.trim();
        if target_id.is_empty() {
            return Err("Target connection id is required".to_owned());
        }
        let table = self.data_diff_table.trim();
        if table.is_empty() {
            return Err("Table is required".to_owned());
        }
        let key_columns = self
            .data_diff_keys
            .split(',')
            .map(str::trim)
            .filter(|column| !column.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if key_columns.is_empty() {
            return Err("At least one key column is required".to_owned());
        }
        Ok(DataDiffRequest {
            target_id: target_id.to_owned(),
            schema: self.data_diff_schema.trim().to_owned(),
            table: table.to_owned(),
            key_columns,
            sample_limit: Some(1_000),
        })
    }

    pub(super) fn prepare_migration_sql(&self, current_target: &str) -> Result<String, String> {
        use db_pro_core::application::MigrationPlanner;

        let plan = self
            .migration_plan
            .as_ref()
            .ok_or_else(|| "Plan a migration before applying".to_owned())?;
        if self
            .migration_plan_target
            .as_deref()
            .is_some_and(|target| target != current_target)
        {
            return Err("Plan target mismatch — the active session moved since the plan was generated".to_owned());
        }
        if !MigrationPlanner::verify_fingerprint(plan, &self.migration_fingerprint_at_preview) {
            return Err("Migration fingerprint changed — re-plan before apply".to_owned());
        }
        if plan.has_destructive && !self.migration_confirm_destructive {
            return Err("Destructive migration requires explicit confirmation checkbox".to_owned());
        }
        let sql = if plan.has_destructive && self.migration_confirm_destructive {
            MigrationPlanner::preview_sql(plan, false)
        } else {
            MigrationPlanner::non_destructive_sql(plan)
        };
        if sql.trim().is_empty() {
            return Err("No supported SQL operations to apply".to_owned());
        }
        if db_pro_core::application::placeholder_remains(&sql) {
            return Err(
                "Migration SQL still contains a placeholder; expand it from introspection before apply".to_owned(),
            );
        }
        Ok(sql)
    }
}

#[cfg(test)]
#[path = "tests/schema_compare_state_tests.rs"]
mod tests;

