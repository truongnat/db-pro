//! Schema Workbench — Phase A object mutation UI (#183–#190, #207, #216–#218, #249–#250).

use super::*;
use crate::components::{Button, ButtonSize, ButtonVariant};
use db_pro_core::domain::object_mutation::*;
use db_pro_core::ports::SqlDialect;
use egui::RichText;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum SchemaWorkbenchMode {
    #[default]
    Table,
    Column,
    View,
    Index,
    Constraint,
    Trigger,
    Sequence,
    Type,
    SchemaDb,
    Extension,
    Comment,
    Partition,
    Dependencies,
    Docs,
}

#[derive(Debug, Clone)]
pub(super) struct SchemaWorkbenchState {
    pub(super) mode: SchemaWorkbenchMode,
    pub(super) schema: String,
    pub(super) name: String,
    pub(super) parent_table: String,
    pub(super) data_type: String,
    pub(super) select_sql: String,
    pub(super) columns_csv: String,
    pub(super) unique: bool,
    pub(super) nullable: bool,
    pub(super) is_pk: bool,
    pub(super) default_expr: String,
    pub(super) new_name: String,
    pub(super) materialized: bool,
    pub(super) constraint_kind: ConstraintKindUi,
    pub(super) expression: String,
    pub(super) ref_schema: String,
    pub(super) ref_table: String,
    pub(super) ref_columns_csv: String,
    pub(super) on_delete: String,
    pub(super) timing: String,
    pub(super) event: String,
    pub(super) body: String,
    pub(super) enum_values_csv: String,
    pub(super) start: String,
    pub(super) increment: String,
    pub(super) cycle: bool,
    pub(super) cascade: bool,
    pub(super) comment_text: String,
    pub(super) partition_bound: String,
    pub(super) extension_schema: String,
    pub(super) preview_sql: String,
    pub(super) preview_safety: String,
    pub(super) preview_fingerprint: String,
    pub(super) preview_error: Option<String>,
    pub(super) apply_confirmation: bool,
    pub(super) docs_markdown: String,
    pub(super) docs_format_html: bool,
    pub(super) dependency_filter: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum ConstraintKindUi {
    #[default]
    PrimaryKey,
    Unique,
    Check,
    ForeignKey,
}

impl Default for SchemaWorkbenchState {
    fn default() -> Self {
        Self {
            mode: SchemaWorkbenchMode::Table,
            schema: "public".into(),
            name: String::new(),
            parent_table: String::new(),
            data_type: "INTEGER".into(),
            select_sql: "SELECT 1".into(),
            columns_csv: "id".into(),
            unique: false,
            nullable: true,
            is_pk: false,
            default_expr: String::new(),
            new_name: String::new(),
            materialized: false,
            constraint_kind: ConstraintKindUi::PrimaryKey,
            expression: "id > 0".into(),
            ref_schema: "public".into(),
            ref_table: String::new(),
            ref_columns_csv: "id".into(),
            on_delete: "NO ACTION".into(),
            timing: "BEFORE".into(),
            event: "INSERT".into(),
            body: "FOR EACH ROW EXECUTE FUNCTION noop()".into(),
            enum_values_csv: "a,b,c".into(),
            start: "1".into(),
            increment: "1".into(),
            cycle: false,
            cascade: false,
            comment_text: String::new(),
            partition_bound: "FROM (MINVALUE) TO (MAXVALUE)".into(),
            extension_schema: String::new(),
            preview_sql: String::new(),
            preview_safety: String::new(),
            preview_fingerprint: String::new(),
            preview_error: None,
            apply_confirmation: false,
            docs_markdown: String::new(),
            docs_format_html: false,
            dependency_filter: String::new(),
        }
    }
}

impl SchemaWorkbenchState {
    pub(super) fn apply_ddl_command(&self, request_id: RequestId, connection_id: String) -> Result<UiCommand, String> {
        let sql = self.preview_sql.trim();
        if sql.is_empty() {
            return Err("Plan a mutation before applying".to_owned());
        }
        Ok(UiCommand::ExecuteDdl {
            request_id,
            connection_id,
            sql: sql.to_owned(),
        })
    }
}

pub(super) struct QuoteDialect;

impl SqlDialect for QuoteDialect {
    fn placeholder(&self, index: usize) -> String {
        format!("${index}")
    }

    fn quote_identifier(&self, name: &str) -> String {
        format!("\"{}\"", name.replace('"', "\"\""))
    }
}

impl DbProApp {
    pub(crate) fn open_schema_workbench(&mut self) {
        if let Some(schema) = self.schema.explorer.selected_schema.clone() {
            self.schema.workbench.schema = schema;
        }
        if let Some(table) = self.schema.explorer.selected_table.clone() {
            self.schema.workbench.parent_table = table;
        }
        self.workspace.activity = Activity::Schema;
        self.workspace.active_tab = WorkspaceTab::SchemaWorkbench;
        self.workspace.sidebar_open = true;
    }

    pub(super) fn draw_schema_workbench_sidebar(&mut self, ui: &mut egui::Ui) {
        section_label(ui, "SCHEMA WORKBENCH", self.theme);
        ui.add_space(6.0);
        ui.label(
            RichText::new("Plan → preview → apply typed object mutations")
                .small()
                .color(self.theme.text_muted),
        );
        ui.add_space(10.0);
        for (mode, icon, label) in [
            (SchemaWorkbenchMode::Table, Icon::Table2, "Table / columns"),
            (SchemaWorkbenchMode::Column, Icon::Columns3, "Column alter"),
            (SchemaWorkbenchMode::View, Icon::Eye, "Views"),
            (SchemaWorkbenchMode::Index, Icon::ListTree, "Indexes"),
            (SchemaWorkbenchMode::Constraint, Icon::Link, "Constraints"),
            (SchemaWorkbenchMode::Trigger, Icon::Zap, "Triggers"),
            (SchemaWorkbenchMode::Sequence, Icon::Hash, "Sequences"),
            (SchemaWorkbenchMode::Type, Icon::Shapes, "Types / enums"),
            (SchemaWorkbenchMode::SchemaDb, Icon::Database, "Schema / database"),
            (SchemaWorkbenchMode::Extension, Icon::Puzzle, "Extensions"),
            (SchemaWorkbenchMode::Comment, Icon::MessageSquareText, "Comments"),
            (SchemaWorkbenchMode::Partition, Icon::LayoutGrid, "Partitions"),
            (SchemaWorkbenchMode::Dependencies, Icon::GitBranch, "Dependencies"),
            (SchemaWorkbenchMode::Docs, Icon::FileText, "Docs export"),
        ] {
            let selected = self.schema.workbench.mode == mode;
            if sidebar_item(ui, icon, label, selected, self.theme).clicked() {
                self.schema.workbench.mode = mode;
                self.workspace.active_tab = WorkspaceTab::SchemaWorkbench;
            }
            ui.add_space(2.0);
        }
    }

    pub(super) fn draw_schema_workbench(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(ui.available_width());
        egui::ScrollArea::vertical()
            .id_salt("schema_workbench_scroll")
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.add_space(SPACE_SM);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Schema Workbench")
                            .font(font_subheading())
                            .strong()
                            .color(self.theme.text_primary),
                    );
                    badge(
                        ui,
                        self.active_query_driver(),
                        self.theme.accent_soft,
                        self.theme.accent,
                    );
                });
                ui.add_space(SPACE_MD);

                match self.schema.workbench.mode {
                    SchemaWorkbenchMode::Dependencies => self.draw_dependency_navigator(ui),
                    SchemaWorkbenchMode::Docs => self.draw_docs_export(ui),
                    _ => {
                        // Cap form width so fields don't stretch across ultrawide canvases.
                        let form_width = ui.available_width().min(720.0);
                        ui.allocate_ui_with_layout(
                            egui::vec2(form_width, ui.available_height()),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                card_frame(self.theme).show(ui, |ui| {
                                    ui.set_min_width(form_width - 8.0);
                                    section_label(ui, "DEFINITION", self.theme);
                                    ui.add_space(SPACE_SM);
                                    let action = {
                                        let can_mutate = self.can_mutate_active_connection();
                                        let mut context = schema_workbench_form::SchemaWorkbenchFormContext {
                                            theme: self.theme,
                                            workbench: &mut self.schema.workbench,
                                            can_mutate,
                                        };
                                        schema_workbench_form::draw_workbench_form(&mut context, ui)
                                    };
                                    if let Some(action) = action {
                                        self.apply_workbench_form_action(action);
                                    }
                                });
                                ui.add_space(SPACE_MD);
                                let action = {
                                    let can_mutate = self.can_mutate_active_connection();
                                    let mut context = schema_workbench_form::SchemaWorkbenchFormContext {
                                        theme: self.theme,
                                        workbench: &mut self.schema.workbench,
                                        can_mutate,
                                    };
                                    schema_workbench_form::draw_workbench_preview(&mut context, ui)
                                };
                                if let Some(action) = action {
                                    self.apply_workbench_form_action(action);
                                }
                            },
                        );
                    }
                }
            });
    }

    // Form + preview: `schema_workbench_form.rs`.

    fn draw_dependency_navigator(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Object dependencies").strong());
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("Filter");
            ui.text_edit_singleline(&mut self.schema.workbench.dependency_filter);
        });
        ui.add_space(6.0);
        let filter = self.schema.workbench.dependency_filter.to_ascii_lowercase();
        let edges = self.collect_ui_dependency_edges();
        if edges.is_empty() {
            ui.label(RichText::new("No dependency edges in the loaded schema summary.").color(self.theme.text_muted));
            return;
        }
        egui::Grid::new("dep_grid").num_columns(3).striped(true).show(ui, |ui| {
            ui.label(RichText::new("From").strong());
            ui.label(RichText::new("Relation").strong());
            ui.label(RichText::new("To").strong());
            ui.end_row();
            for edge in edges {
                let line = format!(
                    "{:?}.{}.{} {} {:?}.{}.{}",
                    edge.from_kind,
                    edge.from_schema.as_deref().unwrap_or("-"),
                    edge.from_name,
                    edge.relation,
                    edge.to_kind,
                    edge.to_schema.as_deref().unwrap_or("-"),
                    edge.to_name
                );
                if !filter.is_empty() && !line.to_ascii_lowercase().contains(&filter) {
                    continue;
                }
                ui.label(format!(
                    "{}.{}",
                    edge.from_schema.as_deref().unwrap_or("-"),
                    edge.from_name
                ));
                ui.label(&edge.relation);
                ui.label(format!("{}.{}", edge.to_schema.as_deref().unwrap_or("-"), edge.to_name));
                ui.end_row();
            }
        });
    }

    fn draw_docs_export(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Schema documentation export").strong());
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .text("Generate Markdown")
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                self.schema.workbench.docs_format_html = false;
                self.schema.workbench.docs_markdown = self.export_schema_docs_markdown();
            }
            if Button::new(self.theme)
                .text("Generate HTML")
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                self.schema.workbench.docs_format_html = true;
                let md = self.export_schema_docs_markdown();
                self.schema.workbench.docs_markdown = format!(
                    "<!DOCTYPE html><html><body><pre>{}</pre></body></html>",
                    md.replace('&', "&amp;").replace('<', "&lt;")
                );
            }
            if Button::new(self.theme)
                .text("Open as query")
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Ghost)
                .show(ui)
                .clicked()
                && !self.schema.workbench.docs_markdown.is_empty()
            {
                let body = self.schema.workbench.docs_markdown.clone();
                self.new_query_document();
                if let Some(doc) = self.query.session.documents.last_mut() {
                    doc.set_text(format!("-- Schema docs export\n/*\n{body}\n*/"));
                }
                self.workspace.active_tab = WorkspaceTab::Query;
            }
        });
        ui.add_space(6.0);
        ui.add(
            egui::TextEdit::multiline(&mut self.schema.workbench.docs_markdown)
                .desired_rows(18)
                .desired_width(f32::INFINITY)
                .code_editor(),
        );
    }

    pub(crate) fn collect_ui_dependency_edges(&self) -> Vec<ObjectDependencyEdge> {
        let mut edges = Vec::new();
        for table in &self.schema.explorer.schema.table_details {
            for fk in &table.foreign_keys {
                edges.push(ObjectDependencyEdge {
                    from_kind: ObjectKind::Table,
                    from_schema: Some(table.schema.clone()),
                    from_name: table.name.clone(),
                    to_kind: ObjectKind::Table,
                    to_schema: Some(fk.to_schema.clone()),
                    to_name: fk.to_table.clone(),
                    relation: format!("fk:{}", fk.name),
                });
            }
        }
        for trigger in &self.schema.explorer.schema.triggers {
            edges.push(ObjectDependencyEdge {
                from_kind: ObjectKind::Trigger,
                from_schema: Some(trigger.schema.clone()),
                from_name: trigger.name.clone(),
                to_kind: ObjectKind::Table,
                to_schema: Some(trigger.schema.clone()),
                to_name: trigger.table_name.clone(),
                relation: "trigger_on".into(),
            });
        }
        for view in &self.schema.explorer.schema.views {
            edges.push(ObjectDependencyEdge {
                from_kind: ObjectKind::View,
                from_schema: Some(view.schema.clone()),
                from_name: view.name.clone(),
                to_kind: ObjectKind::Schema,
                to_schema: None,
                to_name: view.schema.clone(),
                relation: "contained_in".into(),
            });
        }
        edges
    }

    pub(crate) fn export_schema_docs_markdown(&self) -> String {
        let mut out = String::from("# Schema documentation\n\n");
        out.push_str(&format!("Connection driver: `{}`\n\n", self.active_query_driver()));
        out.push_str("## Tables\n\n");
        for table in &self.schema.explorer.schema.table_details {
            out.push_str(&format!("### `{}`.`{}`\n\n", table.schema, table.name));
            out.push_str("| Column | Type | Nullable | PK |\n| --- | --- | --- | --- |\n");
            for col in &table.columns {
                out.push_str(&format!(
                    "| `{}` | `{}` | {} | {} |\n",
                    col.name,
                    col.data_type,
                    if col.nullable { "yes" } else { "no" },
                    if col.is_primary_key { "yes" } else { "no" }
                ));
            }
            out.push('\n');
        }
        if !self.schema.explorer.schema.views.is_empty() {
            out.push_str("## Views\n\n");
            for view in &self.schema.explorer.schema.views {
                out.push_str(&format!("- `{}`.`{}`\n", view.schema, view.name));
            }
        }
        out
    }
}

pub(super) fn nonempty_opt(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

pub(super) fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

pub(super) fn parse_column_defs(schema: &str, table: &str, csv: &str) -> Result<Vec<ColumnDefinition>, String> {
    let mut cols = Vec::new();
    for part in csv.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let mut tokens = part.split(':');
        let name = tokens.next().unwrap_or("").trim().to_owned();
        if name.is_empty() {
            return Err("column name required".into());
        }
        let data_type = tokens.next().unwrap_or("TEXT").trim().to_owned();
        let mut nullable = true;
        let mut is_pk = false;
        for flag in tokens {
            match flag.trim().to_ascii_lowercase().as_str() {
                "pk" => is_pk = true,
                "nn" | "notnull" => nullable = false,
                _ => {}
            }
        }
        cols.push(ColumnDefinition {
            schema: schema.to_owned(),
            table: table.to_owned(),
            name,
            data_type,
            nullable: nullable && !is_pk,
            default: None,
            is_pk,
            new_name: None,
        });
    }
    if cols.is_empty() {
        return Err("at least one column is required".into());
    }
    Ok(cols)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_columns_with_pk_flag() {
        let cols = parse_column_defs("public", "t", "id:INTEGER:pk,name:TEXT").unwrap();
        assert_eq!(cols.len(), 2);
        assert!(cols[0].is_pk);
        assert!(!cols[0].nullable);
    }

    #[test]
    fn apply_ddl_command_requires_a_preview() {
        let state = SchemaWorkbenchState::default();

        assert_eq!(
            state.apply_ddl_command(RequestId(1), "source".to_owned()),
            Err("Plan a mutation before applying".to_owned())
        );
    }
}
