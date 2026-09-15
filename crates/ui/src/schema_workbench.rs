//! Schema Workbench — Phase A object mutation UI (#183–#190, #207, #216–#218, #249–#250).

use super::*;
use db_pro_core::application::ObjectMutationService;
use db_pro_core::domain::object_mutation::*;
use db_pro_core::ports::SqlDialect;
use egui::RichText;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SchemaWorkbenchMode {
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
pub(crate) struct SchemaWorkbenchState {
    pub mode: SchemaWorkbenchMode,
    pub schema: String,
    pub name: String,
    pub parent_table: String,
    pub data_type: String,
    pub select_sql: String,
    pub columns_csv: String,
    pub unique: bool,
    pub nullable: bool,
    pub is_pk: bool,
    pub default_expr: String,
    pub new_name: String,
    pub materialized: bool,
    pub constraint_kind: ConstraintKindUi,
    pub expression: String,
    pub ref_schema: String,
    pub ref_table: String,
    pub ref_columns_csv: String,
    pub on_delete: String,
    pub timing: String,
    pub event: String,
    pub body: String,
    pub enum_values_csv: String,
    pub start: String,
    pub increment: String,
    pub cycle: bool,
    pub cascade: bool,
    pub comment_text: String,
    pub partition_bound: String,
    pub extension_schema: String,
    pub preview_sql: String,
    pub preview_safety: String,
    pub preview_fingerprint: String,
    pub preview_error: Option<String>,
    pub apply_confirmation: bool,
    pub docs_markdown: String,
    pub docs_format_html: bool,
    pub dependency_filter: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ConstraintKindUi {
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

struct QuoteDialect;

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
        if let Some(schema) = self.selected_schema.clone() {
            self.schema_workbench.schema = schema;
        }
        if let Some(table) = self.selected_table.clone() {
            self.schema_workbench.parent_table = table;
        }
        self.activity = Activity::Schema;
        self.active_tab = WorkspaceTab::SchemaWorkbench;
        self.sidebar_open = true;
    }

    pub(super) fn draw_schema_workbench_sidebar(&mut self, ui: &mut egui::Ui) {
        section_label(ui, "SCHEMA WORKBENCH", self.theme);
        ui.add_space(6.0);
        ui.label(
            RichText::new("Plan → preview → apply typed object mutations")
                .small()
                .color(self.theme.text_muted),
        );
        ui.add_space(8.0);
        for (mode, label) in [
            (SchemaWorkbenchMode::Table, "Table / columns"),
            (SchemaWorkbenchMode::Column, "Column alter"),
            (SchemaWorkbenchMode::View, "Views"),
            (SchemaWorkbenchMode::Index, "Indexes"),
            (SchemaWorkbenchMode::Constraint, "Constraints"),
            (SchemaWorkbenchMode::Trigger, "Triggers"),
            (SchemaWorkbenchMode::Sequence, "Sequences"),
            (SchemaWorkbenchMode::Type, "Types / enums"),
            (SchemaWorkbenchMode::SchemaDb, "Schema / database"),
            (SchemaWorkbenchMode::Extension, "Extensions"),
            (SchemaWorkbenchMode::Comment, "Comments"),
            (SchemaWorkbenchMode::Partition, "Partitions"),
            (SchemaWorkbenchMode::Dependencies, "Dependencies"),
            (SchemaWorkbenchMode::Docs, "Docs export"),
        ] {
            let selected = self.schema_workbench.mode == mode;
            if ui.selectable_label(selected, label).clicked() {
                self.schema_workbench.mode = mode;
                self.active_tab = WorkspaceTab::SchemaWorkbench;
            }
        }
    }

    pub(super) fn draw_schema_workbench(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_salt("schema_workbench_scroll")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Schema Workbench")
                            .strong()
                            .color(self.theme.text_primary),
                    );
                    ui.label(
                        RichText::new(format!("· {}", self.active_query_driver()))
                            .small()
                            .color(self.theme.text_muted),
                    );
                });
                ui.add_space(8.0);

                match self.schema_workbench.mode {
                    SchemaWorkbenchMode::Dependencies => self.draw_dependency_navigator(ui),
                    SchemaWorkbenchMode::Docs => self.draw_docs_export(ui),
                    _ => {
                        self.draw_workbench_form(ui);
                        ui.add_space(10.0);
                        self.draw_workbench_preview(ui);
                    }
                }
            });
    }

    fn draw_workbench_form(&mut self, ui: &mut egui::Ui) {
        let mode = self.schema_workbench.mode;
        ui.horizontal(|ui| {
            ui.label("Schema");
            ui.text_edit_singleline(&mut self.schema_workbench.schema);
            ui.label("Name");
            ui.text_edit_singleline(&mut self.schema_workbench.name);
        });

        match mode {
            SchemaWorkbenchMode::Table => {
                ui.label("Columns CSV (name:type[:pk|:nn]) e.g. id:INTEGER:pk,name:TEXT");
                ui.text_edit_singleline(&mut self.schema_workbench.columns_csv);
            }
            SchemaWorkbenchMode::Column => {
                ui.horizontal(|ui| {
                    ui.label("Table");
                    ui.text_edit_singleline(&mut self.schema_workbench.parent_table);
                    ui.label("Type");
                    ui.text_edit_singleline(&mut self.schema_workbench.data_type);
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.schema_workbench.nullable, "Nullable");
                    ui.checkbox(&mut self.schema_workbench.is_pk, "PK (create table only)");
                    ui.label("Default");
                    ui.text_edit_singleline(&mut self.schema_workbench.default_expr);
                });
                ui.horizontal(|ui| {
                    ui.label("Rename to");
                    ui.text_edit_singleline(&mut self.schema_workbench.new_name);
                });
            }
            SchemaWorkbenchMode::View => {
                ui.checkbox(&mut self.schema_workbench.materialized, "Materialized");
                ui.label("SELECT body");
                ui.add(
                    egui::TextEdit::multiline(&mut self.schema_workbench.select_sql)
                        .desired_rows(4)
                        .desired_width(f32::INFINITY),
                );
            }
            SchemaWorkbenchMode::Index => {
                ui.horizontal(|ui| {
                    ui.label("Table");
                    ui.text_edit_singleline(&mut self.schema_workbench.parent_table);
                    ui.checkbox(&mut self.schema_workbench.unique, "Unique");
                });
                ui.label("Columns CSV");
                ui.text_edit_singleline(&mut self.schema_workbench.columns_csv);
            }
            SchemaWorkbenchMode::Constraint => {
                ui.horizontal(|ui| {
                    ui.label("Table");
                    ui.text_edit_singleline(&mut self.schema_workbench.parent_table);
                    egui::ComboBox::from_id_salt("constraint_kind")
                        .selected_text(match self.schema_workbench.constraint_kind {
                            ConstraintKindUi::PrimaryKey => "Primary key",
                            ConstraintKindUi::Unique => "Unique",
                            ConstraintKindUi::Check => "Check",
                            ConstraintKindUi::ForeignKey => "Foreign key",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.schema_workbench.constraint_kind,
                                ConstraintKindUi::PrimaryKey,
                                "Primary key",
                            );
                            ui.selectable_value(
                                &mut self.schema_workbench.constraint_kind,
                                ConstraintKindUi::Unique,
                                "Unique",
                            );
                            ui.selectable_value(
                                &mut self.schema_workbench.constraint_kind,
                                ConstraintKindUi::Check,
                                "Check",
                            );
                            ui.selectable_value(
                                &mut self.schema_workbench.constraint_kind,
                                ConstraintKindUi::ForeignKey,
                                "Foreign key",
                            );
                        });
                });
                ui.label("Columns CSV");
                ui.text_edit_singleline(&mut self.schema_workbench.columns_csv);
                if self.schema_workbench.constraint_kind == ConstraintKindUi::Check {
                    ui.label("Expression");
                    ui.text_edit_singleline(&mut self.schema_workbench.expression);
                }
                if self.schema_workbench.constraint_kind == ConstraintKindUi::ForeignKey {
                    ui.horizontal(|ui| {
                        ui.label("Ref schema");
                        ui.text_edit_singleline(&mut self.schema_workbench.ref_schema);
                        ui.label("Ref table");
                        ui.text_edit_singleline(&mut self.schema_workbench.ref_table);
                    });
                    ui.label("Ref columns CSV");
                    ui.text_edit_singleline(&mut self.schema_workbench.ref_columns_csv);
                    ui.label("ON DELETE");
                    ui.text_edit_singleline(&mut self.schema_workbench.on_delete);
                }
            }
            SchemaWorkbenchMode::Trigger => {
                ui.horizontal(|ui| {
                    ui.label("Table");
                    ui.text_edit_singleline(&mut self.schema_workbench.parent_table);
                    ui.label("Timing");
                    ui.text_edit_singleline(&mut self.schema_workbench.timing);
                    ui.label("Event");
                    ui.text_edit_singleline(&mut self.schema_workbench.event);
                });
                ui.label("Body");
                ui.add(
                    egui::TextEdit::multiline(&mut self.schema_workbench.body)
                        .desired_rows(3)
                        .desired_width(f32::INFINITY),
                );
            }
            SchemaWorkbenchMode::Sequence => {
                ui.horizontal(|ui| {
                    ui.label("Start");
                    ui.text_edit_singleline(&mut self.schema_workbench.start);
                    ui.label("Increment");
                    ui.text_edit_singleline(&mut self.schema_workbench.increment);
                    ui.checkbox(&mut self.schema_workbench.cycle, "Cycle");
                });
            }
            SchemaWorkbenchMode::Type => {
                ui.label("Enum values CSV");
                ui.text_edit_singleline(&mut self.schema_workbench.enum_values_csv);
            }
            SchemaWorkbenchMode::SchemaDb => {
                ui.label("Schema name uses Name field; Database create/drop uses Name as DB name.");
                ui.checkbox(&mut self.schema_workbench.cascade, "CASCADE on drop schema");
            }
            SchemaWorkbenchMode::Extension => {
                ui.label("Extension schema (optional)");
                ui.text_edit_singleline(&mut self.schema_workbench.extension_schema);
                ui.checkbox(&mut self.schema_workbench.cascade, "CASCADE on drop");
            }
            SchemaWorkbenchMode::Comment => {
                ui.horizontal(|ui| {
                    ui.label("Parent (column comments)");
                    ui.text_edit_singleline(&mut self.schema_workbench.parent_table);
                });
                ui.label("Comment text (empty clears)");
                ui.text_edit_singleline(&mut self.schema_workbench.comment_text);
            }
            SchemaWorkbenchMode::Partition => {
                ui.horizontal(|ui| {
                    ui.label("Parent table");
                    ui.text_edit_singleline(&mut self.schema_workbench.parent_table);
                });
                ui.label("FOR VALUES …");
                ui.text_edit_singleline(&mut self.schema_workbench.partition_bound);
            }
            SchemaWorkbenchMode::Dependencies | SchemaWorkbenchMode::Docs => {}
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("Plan create").clicked() {
                self.plan_workbench_action(ObjectAction::Create);
            }
            if ui.button("Plan drop").clicked() {
                self.plan_workbench_action(ObjectAction::Drop);
            }
            if mode == SchemaWorkbenchMode::Column && ui.button("Plan rename").clicked() {
                self.plan_workbench_action(ObjectAction::Rename);
            }
            if mode == SchemaWorkbenchMode::View
                && self.schema_workbench.materialized
                && ui.button("Plan refresh").clicked()
            {
                self.plan_workbench_action(ObjectAction::Refresh);
            }
            if mode == SchemaWorkbenchMode::Comment && ui.button("Plan comment").clicked() {
                self.plan_workbench_action(ObjectAction::Comment);
            }
            if mode == SchemaWorkbenchMode::SchemaDb {
                if ui.button("Plan create database").clicked() {
                    self.plan_database_action(ObjectAction::Create);
                }
                if ui.button("Plan drop database").clicked() {
                    self.plan_database_action(ObjectAction::Drop);
                }
            }
        });
    }

    fn draw_workbench_preview(&mut self, ui: &mut egui::Ui) {
        if let Some(err) = &self.schema_workbench.preview_error {
            ui.colored_label(self.theme.danger, err);
        }
        if !self.schema_workbench.preview_safety.is_empty() {
            ui.label(
                RichText::new(format!(
                    "Safety: {} · fingerprint {}",
                    self.schema_workbench.preview_safety, self.schema_workbench.preview_fingerprint
                ))
                .small()
                .color(self.theme.text_muted),
            );
        }
        ui.add_space(4.0);
        ui.label(RichText::new("Preview SQL").strong());
        ui.add(
            egui::TextEdit::multiline(&mut self.schema_workbench.preview_sql)
                .desired_rows(8)
                .desired_width(f32::INFINITY)
                .code_editor(),
        );
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let can_apply = !self.schema_workbench.preview_sql.trim().is_empty()
                && self.schema_workbench.preview_error.is_none()
                && self.can_mutate_active_connection();
            if ui.add_enabled(can_apply, egui::Button::new("Apply DDL…")).clicked() {
                self.schema_workbench.apply_confirmation = true;
            }
            if ui.button("Open in SQL editor").clicked() && !self.schema_workbench.preview_sql.trim().is_empty() {
                let sql = self.schema_workbench.preview_sql.clone();
                self.new_query_document();
                if let Some(doc) = self.query_documents.last_mut() {
                    doc.set_text(sql);
                }
                self.active_tab = WorkspaceTab::Query;
                self.activity = Activity::Queries;
            }
        });

        if self.schema_workbench.apply_confirmation {
            egui::Window::new("Confirm DDL apply")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label("Apply the previewed DDL to the active connection?");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.schema_workbench.apply_confirmation = false;
                        }
                        if ui.button("Apply").clicked() {
                            self.schema_workbench.apply_confirmation = false;
                            self.apply_workbench_ddl();
                        }
                    });
                });
        }
    }

    fn draw_dependency_navigator(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Object dependencies").strong());
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("Filter");
            ui.text_edit_singleline(&mut self.schema_workbench.dependency_filter);
        });
        ui.add_space(6.0);
        let filter = self.schema_workbench.dependency_filter.to_ascii_lowercase();
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
            if ui.button("Generate Markdown").clicked() {
                self.schema_workbench.docs_format_html = false;
                self.schema_workbench.docs_markdown = self.export_schema_docs_markdown();
            }
            if ui.button("Generate HTML").clicked() {
                self.schema_workbench.docs_format_html = true;
                let md = self.export_schema_docs_markdown();
                self.schema_workbench.docs_markdown = format!(
                    "<!DOCTYPE html><html><body><pre>{}</pre></body></html>",
                    md.replace('&', "&amp;").replace('<', "&lt;")
                );
            }
            if ui.button("Open as query").clicked() && !self.schema_workbench.docs_markdown.is_empty() {
                let body = self.schema_workbench.docs_markdown.clone();
                self.new_query_document();
                if let Some(doc) = self.query_documents.last_mut() {
                    doc.set_text(format!("-- Schema docs export\n/*\n{body}\n*/"));
                }
                self.active_tab = WorkspaceTab::Query;
            }
        });
        ui.add_space(6.0);
        ui.add(
            egui::TextEdit::multiline(&mut self.schema_workbench.docs_markdown)
                .desired_rows(18)
                .desired_width(f32::INFINITY)
                .code_editor(),
        );
    }

    fn collect_ui_dependency_edges(&self) -> Vec<ObjectDependencyEdge> {
        let mut edges = Vec::new();
        for table in &self.schema.table_details {
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
        for trigger in &self.schema.triggers {
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
        for view in &self.schema.views {
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

    fn export_schema_docs_markdown(&self) -> String {
        let mut out = String::from("# Schema documentation\n\n");
        out.push_str(&format!("Connection driver: `{}`\n\n", self.active_query_driver()));
        out.push_str("## Tables\n\n");
        for table in &self.schema.table_details {
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
        if !self.schema.views.is_empty() {
            out.push_str("## Views\n\n");
            for view in &self.schema.views {
                out.push_str(&format!("- `{}`.`{}`\n", view.schema, view.name));
            }
        }
        out
    }

    fn plan_workbench_action(&mut self, action: ObjectAction) {
        match self.build_mutation_request(action) {
            Ok(request) => self.run_plan(request),
            Err(err) => {
                self.schema_workbench.preview_error = Some(err);
                self.schema_workbench.preview_sql.clear();
            }
        }
    }

    fn plan_database_action(&mut self, action: ObjectAction) {
        let request = ObjectMutationRequest {
            action,
            target: Some(ObjectRef {
                kind: ObjectKind::Database,
                schema: None,
                name: self.schema_workbench.name.clone(),
                parent: None,
            }),
            definition: ObjectDefinition::Database(DatabaseDefinition {
                name: self.schema_workbench.name.clone(),
                owner: None,
                template: None,
            }),
            options: MutationOptions {
                cascade: self.schema_workbench.cascade,
                ..MutationOptions::default()
            },
            driver: self.active_query_driver().to_owned(),
        };
        self.run_plan(request);
    }

    fn run_plan(&mut self, request: ObjectMutationRequest) {
        match ObjectMutationService::plan(&request, &QuoteDialect) {
            Ok(preview) => {
                if let Some(reason) = preview.unsupported_reason {
                    self.schema_workbench.preview_error = Some(reason);
                    self.schema_workbench.preview_sql.clear();
                } else {
                    self.schema_workbench.preview_error = None;
                    self.schema_workbench.preview_sql = preview.statements.join(";\n");
                    if !self.schema_workbench.preview_sql.is_empty() {
                        self.schema_workbench.preview_sql.push(';');
                    }
                }
                self.schema_workbench.preview_safety = preview.safety;
                self.schema_workbench.preview_fingerprint = preview.fingerprint;
            }
            Err(err) => {
                self.schema_workbench.preview_error = Some(err.to_string());
                self.schema_workbench.preview_sql.clear();
            }
        }
    }

    fn apply_workbench_ddl(&mut self) {
        if !self.can_mutate_active_connection() {
            self.runtime_message = "Connect with write access to apply DDL".into();
            return;
        }
        if self.ddl_execution_request.is_some() {
            return;
        }
        let sql = self.schema_workbench.preview_sql.trim().to_owned();
        if sql.is_empty() {
            self.runtime_message = "Plan a mutation before applying".into();
            return;
        }
        let Some(connection) = self.active_connection().cloned() else {
            self.runtime_message = "Connect to a database before applying DDL".into();
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::ExecuteDdl {
            request_id,
            connection_id: connection.id,
            sql,
        });
        self.ddl_execution_request = Some(request_id);
        self.runtime_message = "Applying schema mutation…".into();
    }

    fn build_mutation_request(&self, action: ObjectAction) -> Result<ObjectMutationRequest, String> {
        let driver = self.active_query_driver().to_owned();
        let schema = self.schema_workbench.schema.clone();
        let name = self.schema_workbench.name.clone();
        if name.trim().is_empty()
            && !matches!(
                self.schema_workbench.mode,
                SchemaWorkbenchMode::Dependencies | SchemaWorkbenchMode::Docs
            )
        {
            return Err("Name is required".into());
        }
        let options = MutationOptions {
            cascade: self.schema_workbench.cascade,
            ..MutationOptions::default()
        };

        let (definition, kind, parent) = match self.schema_workbench.mode {
            SchemaWorkbenchMode::Table => {
                let columns = parse_column_defs(&schema, &name, &self.schema_workbench.columns_csv)?;
                (
                    ObjectDefinition::Table(TableDefinition {
                        schema: schema.clone(),
                        name: name.clone(),
                        columns,
                    }),
                    ObjectKind::Table,
                    None,
                )
            }
            SchemaWorkbenchMode::Column => (
                ObjectDefinition::Column(ColumnDefinition {
                    schema: schema.clone(),
                    table: self.schema_workbench.parent_table.clone(),
                    name: name.clone(),
                    data_type: self.schema_workbench.data_type.clone(),
                    nullable: self.schema_workbench.nullable,
                    default: nonempty_opt(&self.schema_workbench.default_expr),
                    is_pk: self.schema_workbench.is_pk,
                    new_name: nonempty_opt(&self.schema_workbench.new_name),
                }),
                ObjectKind::Column,
                Some(self.schema_workbench.parent_table.clone()),
            ),
            SchemaWorkbenchMode::View => {
                let def = ViewDefinition {
                    schema: schema.clone(),
                    name: name.clone(),
                    select_sql: self.schema_workbench.select_sql.clone(),
                    materialized: self.schema_workbench.materialized,
                    replace: false,
                };
                let kind = if self.schema_workbench.materialized {
                    ObjectKind::MaterializedView
                } else {
                    ObjectKind::View
                };
                let definition = if self.schema_workbench.materialized {
                    ObjectDefinition::MaterializedView(def)
                } else {
                    ObjectDefinition::View(def)
                };
                (definition, kind, None)
            }
            SchemaWorkbenchMode::Index => (
                ObjectDefinition::Index(IndexDefinition {
                    schema: schema.clone(),
                    table: self.schema_workbench.parent_table.clone(),
                    name: name.clone(),
                    columns: split_csv(&self.schema_workbench.columns_csv),
                    unique: self.schema_workbench.unique,
                    method: None,
                    predicate: None,
                }),
                ObjectKind::Index,
                Some(self.schema_workbench.parent_table.clone()),
            ),
            SchemaWorkbenchMode::Constraint => {
                let columns = split_csv(&self.schema_workbench.columns_csv);
                match self.schema_workbench.constraint_kind {
                    ConstraintKindUi::PrimaryKey => (
                        ObjectDefinition::PrimaryKey(NamedColumnsDefinition {
                            schema: schema.clone(),
                            table: self.schema_workbench.parent_table.clone(),
                            name: name.clone(),
                            columns,
                        }),
                        ObjectKind::PrimaryKey,
                        Some(self.schema_workbench.parent_table.clone()),
                    ),
                    ConstraintKindUi::Unique => (
                        ObjectDefinition::UniqueConstraint(NamedColumnsDefinition {
                            schema: schema.clone(),
                            table: self.schema_workbench.parent_table.clone(),
                            name: name.clone(),
                            columns,
                        }),
                        ObjectKind::UniqueConstraint,
                        Some(self.schema_workbench.parent_table.clone()),
                    ),
                    ConstraintKindUi::Check => (
                        ObjectDefinition::CheckConstraint(CheckDefinition {
                            schema: schema.clone(),
                            table: self.schema_workbench.parent_table.clone(),
                            name: name.clone(),
                            expression: self.schema_workbench.expression.clone(),
                        }),
                        ObjectKind::CheckConstraint,
                        Some(self.schema_workbench.parent_table.clone()),
                    ),
                    ConstraintKindUi::ForeignKey => (
                        ObjectDefinition::ForeignKey(ForeignKeyDefinition {
                            schema: schema.clone(),
                            table: self.schema_workbench.parent_table.clone(),
                            name: name.clone(),
                            columns,
                            ref_schema: self.schema_workbench.ref_schema.clone(),
                            ref_table: self.schema_workbench.ref_table.clone(),
                            ref_columns: split_csv(&self.schema_workbench.ref_columns_csv),
                            on_delete: nonempty_opt(&self.schema_workbench.on_delete),
                            on_update: None,
                        }),
                        ObjectKind::ForeignKey,
                        Some(self.schema_workbench.parent_table.clone()),
                    ),
                }
            }
            SchemaWorkbenchMode::Trigger => (
                ObjectDefinition::Trigger(TriggerDefinition {
                    schema: schema.clone(),
                    table: self.schema_workbench.parent_table.clone(),
                    name: name.clone(),
                    timing: self.schema_workbench.timing.clone(),
                    event: self.schema_workbench.event.clone(),
                    body: self.schema_workbench.body.clone(),
                }),
                ObjectKind::Trigger,
                Some(self.schema_workbench.parent_table.clone()),
            ),
            SchemaWorkbenchMode::Sequence => (
                ObjectDefinition::Sequence(SequenceDefinition {
                    schema: schema.clone(),
                    name: name.clone(),
                    start: self.schema_workbench.start.parse().ok(),
                    increment: self.schema_workbench.increment.parse().ok(),
                    min_value: None,
                    max_value: None,
                    cache: None,
                    cycle: self.schema_workbench.cycle,
                }),
                ObjectKind::Sequence,
                None,
            ),
            SchemaWorkbenchMode::Type => (
                ObjectDefinition::EnumType(EnumTypeDefinition {
                    schema: schema.clone(),
                    name: name.clone(),
                    values: split_csv(&self.schema_workbench.enum_values_csv),
                }),
                ObjectKind::EnumType,
                None,
            ),
            SchemaWorkbenchMode::SchemaDb => (
                ObjectDefinition::Schema(SchemaDefinition {
                    name: name.clone(),
                    new_name: None,
                    owner: None,
                }),
                ObjectKind::Schema,
                None,
            ),
            SchemaWorkbenchMode::Extension => (
                ObjectDefinition::Extension(ExtensionDefinition {
                    name: name.clone(),
                    schema: nonempty_opt(&self.schema_workbench.extension_schema),
                    version: None,
                    cascade: self.schema_workbench.cascade,
                }),
                ObjectKind::Extension,
                None,
            ),
            SchemaWorkbenchMode::Comment => {
                let parent = nonempty_opt(&self.schema_workbench.parent_table);
                let kind = if parent.is_some() {
                    ObjectKind::Column
                } else {
                    ObjectKind::Table
                };
                (
                    ObjectDefinition::Comment(CommentDefinition {
                        object: ObjectRef {
                            kind,
                            schema: Some(schema.clone()),
                            name: name.clone(),
                            parent: parent.clone(),
                        },
                        comment: nonempty_opt(&self.schema_workbench.comment_text),
                    }),
                    ObjectKind::Comment,
                    parent,
                )
            }
            SchemaWorkbenchMode::Partition => (
                ObjectDefinition::Partition(PartitionDefinition {
                    schema: schema.clone(),
                    parent_table: self.schema_workbench.parent_table.clone(),
                    name: name.clone(),
                    strategy: "RANGE".into(),
                    bound_expression: self.schema_workbench.partition_bound.clone(),
                }),
                ObjectKind::Partition,
                Some(self.schema_workbench.parent_table.clone()),
            ),
            SchemaWorkbenchMode::Dependencies | SchemaWorkbenchMode::Docs => {
                return Err("This mode does not produce DDL".into());
            }
        };

        Ok(ObjectMutationRequest {
            action,
            target: Some(ObjectRef {
                kind,
                schema: Some(schema),
                name,
                parent,
            }),
            definition,
            options,
            driver,
        })
    }
}

fn nonempty_opt(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

fn parse_column_defs(schema: &str, table: &str, csv: &str) -> Result<Vec<ColumnDefinition>, String> {
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
}
