//! Presentation and intent mapping for the connected database/schema tree.

use super::explorer_database_node_view::DatabaseNodeContext;
use super::explorer_schema_feedback_view::{ExplorerSchemaFeedbackAction, ExplorerSchemaFeedbackContext};
use super::explorer_schema_node_view::SchemaNodeContext;
use super::explorer_schema_objects_view::{
    ExplorerSchemaObjectsAction, ExplorerSchemaObjectsModel, ExplorerSchemaObjectsView,
};
use super::{is_user_visible_schema, DbProTheme, SchemaExplorerState, UiTableInfo};
use eframe::egui;

pub(super) enum ExplorerSchemaTreeAction {
    RefreshSchema,
    ActivateSchema(String),
    SchemaObjects(ExplorerSchemaObjectsAction),
}

pub(super) struct ExplorerSchemaTreeModel {
    pub(super) connection_id: String,
    pub(super) database: String,
    pub(super) active_schema: String,
    pub(super) schema_error: Option<String>,
    pub(super) schema_loading: bool,
    pub(super) reduce_motion: bool,
    pub(super) selected_table: Option<String>,
    pub(super) table_info: Option<UiTableInfo>,
    pub(super) functions_enabled: bool,
}

pub(super) struct ExplorerSchemaTreeView<'a> {
    theme: DbProTheme,
    explorer: &'a mut SchemaExplorerState,
    model: ExplorerSchemaTreeModel,
}

impl<'a> ExplorerSchemaTreeView<'a> {
    pub(super) fn new(
        theme: DbProTheme,
        explorer: &'a mut SchemaExplorerState,
        model: ExplorerSchemaTreeModel,
    ) -> Self {
        Self { theme, explorer, model }
    }

    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<ExplorerSchemaTreeAction> {
        let mut actions = self.draw_feedback(ui);
        let database_open = DatabaseNodeContext {
            theme: self.theme,
            connection_id: &self.model.connection_id,
            database: &self.model.database,
        }
        .draw(ui);

        if database_open {
            actions.extend(self.draw_schemas(ui));
        }
        actions
    }

    fn draw_feedback(&self, ui: &mut egui::Ui) -> Vec<ExplorerSchemaTreeAction> {
        ExplorerSchemaFeedbackContext {
            theme: self.theme,
            error: self.model.schema_error.as_deref(),
            loading: self.model.schema_loading,
            reduce_motion: self.model.reduce_motion,
            has_active_connection: true,
        }
        .draw(ui)
        .into_iter()
        .map(|action| match action {
            ExplorerSchemaFeedbackAction::RefreshSchema => ExplorerSchemaTreeAction::RefreshSchema,
        })
        .collect()
    }

    fn draw_schemas(&mut self, ui: &mut egui::Ui) -> Vec<ExplorerSchemaTreeAction> {
        let schemas = self.explorer.schema.schemas.clone();
        if schemas.is_empty() {
            return self.draw_schema_objects(ui, "");
        }

        let mut actions = Vec::new();
        for schema in schemas {
            if !is_user_visible_schema(&schema) {
                continue;
            }
            actions.extend(self.draw_schema_node(ui, &schema));
        }
        actions
    }

    fn draw_schema_node(&mut self, ui: &mut egui::Ui, schema: &str) -> Vec<ExplorerSchemaTreeAction> {
        let is_active = self.model.active_schema == schema;
        let table_count = super::connection_status::schema_table_count(self.explorer, schema);
        let render = SchemaNodeContext {
            theme: self.theme,
            connection_id: &self.model.connection_id,
            schema,
            is_active,
            table_count,
        }
        .draw(ui);

        let mut actions = Vec::new();
        if render.is_open {
            if is_active {
                actions.extend(self.draw_schema_objects(ui, schema));
            } else if super::explorer_tree::draw_hint_row(
                ui,
                &self.theme,
                3,
                lucide_icons::Icon::Circle,
                "Inactive schema — click to activate",
            )
            .clicked()
            {
                actions.push(ExplorerSchemaTreeAction::ActivateSchema(schema.to_owned()));
            }
        }
        if render.should_activate {
            actions.push(ExplorerSchemaTreeAction::ActivateSchema(schema.to_owned()));
        }
        actions
    }

    fn draw_schema_objects(&mut self, ui: &mut egui::Ui, schema: &str) -> Vec<ExplorerSchemaTreeAction> {
        let model = ExplorerSchemaObjectsModel {
            selected_table: self.model.selected_table.clone(),
            table_info: self.model.table_info.clone(),
            connection_id: self.model.connection_id.clone(),
            functions_enabled: self.model.functions_enabled,
        };
        ExplorerSchemaObjectsView::new(self.theme, self.explorer, model)
            .draw(ui, schema)
            .into_iter()
            .map(ExplorerSchemaTreeAction::SchemaObjects)
            .collect()
    }
}
