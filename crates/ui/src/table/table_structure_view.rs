//! Table structure root adapter.
use super::schema_workbench::SchemaWorkbenchMode;
use self::table_structure_surface_view::TableStructureAction;
use super::*;
use db_pro_core::domain::capabilities::CapabilityFeature;
use db_pro_core::domain::object_mutation::ObjectAction;

#[path = "table_structure_surface_view.rs"]
pub(super) mod table_structure_surface_view;

impl DbProApp {
    /// Draws the structure surface from an immutable table-info snapshot.
    pub(super) fn draw_table_structure_view(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table.state.table_info.clone() else {
            table_structure_surface_view::draw_placeholder(
                self.theme,
                self.table.state.table_info_error.as_deref(),
                ui,
            );
            return;
        };
        let alter_column_limitation = self
            .query_capabilities()
            .feature_limitation(CapabilityFeature::AlterColumnType);
        let action = {
            let mut context = table_structure_surface_view::TableStructureContext {
                theme: self.theme,
                info: &info,
                search: &mut self.table.state.table_structure_search,
                alter_column_limitation,
            };
            context.draw(ui)
        };
        if let Some(TableStructureAction::AlterColumn(column)) = action {
            self.plan_column_alter(&info, &column);
        }
    }

    /// Seed the schema workbench with a column's introspected definition and
    /// plan the provider-specific ALTER COLUMN statement. The preview + apply
    /// path is the same confirmation-gated `ExecuteDdl` flow every other
    /// workbench mutation uses; capability mismatches surface as a plan error.
    pub(crate) fn plan_column_alter(&mut self, info: &UiTableInfo, column: &crate::UiTableColumn) {
        self.schema.workbench.mode = SchemaWorkbenchMode::Column;
        self.schema.workbench.schema = info.schema.clone();
        self.schema.workbench.name = column.name.clone();
        self.schema.workbench.parent_table = info.name.clone();
        self.schema.workbench.data_type = column.data_type.clone();
        self.schema.workbench.nullable = column.nullable;
        self.schema.workbench.is_pk = column.is_primary_key;
        self.schema.workbench.default_expr = column.default.clone().unwrap_or_default();
        self.schema.workbench.new_name.clear();
        self.plan_workbench_action(ObjectAction::Alter);
        self.workspace.activity = Activity::Schema;
        self.workspace.active_tab = WorkspaceTab::SchemaWorkbench;
        self.workspace.sidebar_open = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_column_alter_seeds_workbench_and_plans_alter_sql() {
        let mut app = DbProApp::default();
        let column = crate::UiTableColumn {
            name: "email".into(),
            data_type: "VARCHAR(255)".into(),
            ordinal: 2,
            nullable: true,
            default: Some("'x'".into()),
            is_primary_key: false,
            ..Default::default()
        };
        let info = UiTableInfo {
            schema: "public".into(),
            name: "users".into(),
            row_count: None,
            columns: vec![column.clone()],
            primary_key: None,
            indexes: vec![],
            foreign_keys: vec![],
            check_constraints: vec![],
            dependencies: vec![],
        };

        app.plan_column_alter(&info, &column);

        let workbench = &app.schema.workbench;
        assert_eq!(workbench.mode, SchemaWorkbenchMode::Column);
        assert_eq!(workbench.parent_table, "users");
        assert_eq!(workbench.name, "email");
        assert_eq!(workbench.data_type, "VARCHAR(255)");
        assert_eq!(workbench.default_expr, "'x'");
        assert!(workbench.preview_error.is_none());
        assert!(workbench
            .preview_sql
            .contains("ALTER TABLE \"public\".\"users\" ALTER COLUMN \"email\" TYPE VARCHAR(255)"));
        assert_eq!(app.workspace.active_tab, WorkspaceTab::SchemaWorkbench);
    }
}
