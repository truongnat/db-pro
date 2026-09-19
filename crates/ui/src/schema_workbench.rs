//! Schema Workbench — Phase A object mutation UI (#183–#190, #207, #216–#218, #249–#250).

use super::*;
use crate::components::{Button, ButtonSize, ButtonVariant};
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
        if let Some(schema) = self.schema_explorer.selected_schema.clone() {
            self.database_operations.schema_workbench.schema = schema;
        }
        if let Some(table) = self.schema_explorer.selected_table.clone() {
            self.database_operations.schema_workbench.parent_table = table;
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
            let selected = self.database_operations.schema_workbench.mode == mode;
            if sidebar_item(ui, icon, label, selected, self.theme).clicked() {
                self.database_operations.schema_workbench.mode = mode;
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

                match self.database_operations.schema_workbench.mode {
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
                                    self.draw_workbench_form(ui);
                                });
                                ui.add_space(SPACE_MD);
                                self.draw_workbench_preview(ui);
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
            ui.text_edit_singleline(&mut self.database_operations.schema_workbench.dependency_filter);
        });
        ui.add_space(6.0);
        let filter = self
            .database_operations
            .schema_workbench
            .dependency_filter
            .to_ascii_lowercase();
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
                self.database_operations.schema_workbench.docs_format_html = false;
                self.database_operations.schema_workbench.docs_markdown = self.export_schema_docs_markdown();
            }
            if Button::new(self.theme)
                .text("Generate HTML")
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                self.database_operations.schema_workbench.docs_format_html = true;
                let md = self.export_schema_docs_markdown();
                self.database_operations.schema_workbench.docs_markdown = format!(
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
                && !self.database_operations.schema_workbench.docs_markdown.is_empty()
            {
                let body = self.database_operations.schema_workbench.docs_markdown.clone();
                self.new_query_document();
                if let Some(doc) = self.query_session_state.documents.last_mut() {
                    doc.set_text(format!("-- Schema docs export\n/*\n{body}\n*/"));
                }
                self.workspace.active_tab = WorkspaceTab::Query;
            }
        });
        ui.add_space(6.0);
        ui.add(
            egui::TextEdit::multiline(&mut self.database_operations.schema_workbench.docs_markdown)
                .desired_rows(18)
                .desired_width(f32::INFINITY)
                .code_editor(),
        );
    }

    pub(crate) fn collect_ui_dependency_edges(&self) -> Vec<ObjectDependencyEdge> {
        let mut edges = Vec::new();
        for table in &self.schema_explorer.schema.table_details {
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
        for trigger in &self.schema_explorer.schema.triggers {
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
        for view in &self.schema_explorer.schema.views {
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
        for table in &self.schema_explorer.schema.table_details {
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
        if !self.schema_explorer.schema.views.is_empty() {
            out.push_str("## Views\n\n");
            for view in &self.schema_explorer.schema.views {
                out.push_str(&format!("- `{}`.`{}`\n", view.schema, view.name));
            }
        }
        out
    }

    pub(crate) fn plan_workbench_action(&mut self, action: ObjectAction) {
        match self.build_mutation_request(action) {
            Ok(request) => self.run_plan(request),
            Err(err) => {
                self.database_operations.schema_workbench.preview_error = Some(err);
                self.database_operations.schema_workbench.preview_sql.clear();
            }
        }
    }

    pub(crate) fn plan_database_action(&mut self, action: ObjectAction) {
        let request = ObjectMutationRequest {
            action,
            target: Some(ObjectRef {
                kind: ObjectKind::Database,
                schema: None,
                name: self.database_operations.schema_workbench.name.clone(),
                parent: None,
            }),
            definition: ObjectDefinition::Database(DatabaseDefinition {
                name: self.database_operations.schema_workbench.name.clone(),
                owner: None,
                template: None,
            }),
            options: MutationOptions {
                cascade: self.database_operations.schema_workbench.cascade,
                ..MutationOptions::default()
            },
            driver: self.active_query_driver().to_owned(),
        };
        self.run_plan(request);
    }

    pub(crate) fn run_plan(&mut self, request: ObjectMutationRequest) {
        match ObjectMutationService::plan(&request, &QuoteDialect) {
            Ok(preview) => {
                if let Some(reason) = preview.unsupported_reason {
                    self.database_operations.schema_workbench.preview_error = Some(reason);
                    self.database_operations.schema_workbench.preview_sql.clear();
                } else {
                    self.database_operations.schema_workbench.preview_error = None;
                    self.database_operations.schema_workbench.preview_sql = preview.statements.join(";\n");
                    if !self.database_operations.schema_workbench.preview_sql.is_empty() {
                        self.database_operations.schema_workbench.preview_sql.push(';');
                    }
                }
                self.database_operations.schema_workbench.preview_safety = preview.safety;
                self.database_operations.schema_workbench.preview_fingerprint = preview.fingerprint;
            }
            Err(err) => {
                self.database_operations.schema_workbench.preview_error = Some(err.to_string());
                self.database_operations.schema_workbench.preview_sql.clear();
            }
        }
    }

    pub(crate) fn apply_workbench_ddl(&mut self) {
        if !self.can_mutate_active_connection() {
            self.feedback.runtime_message = "Connect with write access to apply DDL".into();
            return;
        }
        if self.table_state.ddl_execution_request.is_some() {
            return;
        }
        let sql = self.database_operations.schema_workbench.preview_sql.trim().to_owned();
        if sql.is_empty() {
            self.feedback.runtime_message = "Plan a mutation before applying".into();
            return;
        }
        let Some(connection) = self.active_connection().cloned() else {
            self.feedback.runtime_message = "Connect to a database before applying DDL".into();
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::ExecuteDdl {
            request_id,
            connection_id: connection.id,
            sql,
        });
        self.table_state.ddl_execution_request = Some(request_id);
        self.feedback.runtime_message = "Applying schema mutation…".into();
    }

    pub(crate) fn build_mutation_request(&self, action: ObjectAction) -> Result<ObjectMutationRequest, String> {
        let driver = self.active_query_driver().to_owned();
        let schema = self.database_operations.schema_workbench.schema.clone();
        let name = self.database_operations.schema_workbench.name.clone();
        if name.trim().is_empty()
            && !matches!(
                self.database_operations.schema_workbench.mode,
                SchemaWorkbenchMode::Dependencies | SchemaWorkbenchMode::Docs
            )
        {
            return Err("Name is required".into());
        }
        let options = MutationOptions {
            cascade: self.database_operations.schema_workbench.cascade,
            ..MutationOptions::default()
        };

        let (definition, kind, parent) = match self.database_operations.schema_workbench.mode {
            SchemaWorkbenchMode::Table => {
                let columns =
                    parse_column_defs(&schema, &name, &self.database_operations.schema_workbench.columns_csv)?;
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
                    table: self.database_operations.schema_workbench.parent_table.clone(),
                    name: name.clone(),
                    data_type: self.database_operations.schema_workbench.data_type.clone(),
                    nullable: self.database_operations.schema_workbench.nullable,
                    default: nonempty_opt(&self.database_operations.schema_workbench.default_expr),
                    is_pk: self.database_operations.schema_workbench.is_pk,
                    new_name: nonempty_opt(&self.database_operations.schema_workbench.new_name),
                }),
                ObjectKind::Column,
                Some(self.database_operations.schema_workbench.parent_table.clone()),
            ),
            SchemaWorkbenchMode::View => {
                let def = ViewDefinition {
                    schema: schema.clone(),
                    name: name.clone(),
                    select_sql: self.database_operations.schema_workbench.select_sql.clone(),
                    materialized: self.database_operations.schema_workbench.materialized,
                    replace: false,
                };
                let kind = if self.database_operations.schema_workbench.materialized {
                    ObjectKind::MaterializedView
                } else {
                    ObjectKind::View
                };
                let definition = if self.database_operations.schema_workbench.materialized {
                    ObjectDefinition::MaterializedView(def)
                } else {
                    ObjectDefinition::View(def)
                };
                (definition, kind, None)
            }
            SchemaWorkbenchMode::Index => (
                ObjectDefinition::Index(IndexDefinition {
                    schema: schema.clone(),
                    table: self.database_operations.schema_workbench.parent_table.clone(),
                    name: name.clone(),
                    columns: split_csv(&self.database_operations.schema_workbench.columns_csv),
                    unique: self.database_operations.schema_workbench.unique,
                    method: None,
                    predicate: None,
                }),
                ObjectKind::Index,
                Some(self.database_operations.schema_workbench.parent_table.clone()),
            ),
            SchemaWorkbenchMode::Constraint => {
                let columns = split_csv(&self.database_operations.schema_workbench.columns_csv);
                match self.database_operations.schema_workbench.constraint_kind {
                    ConstraintKindUi::PrimaryKey => (
                        ObjectDefinition::PrimaryKey(NamedColumnsDefinition {
                            schema: schema.clone(),
                            table: self.database_operations.schema_workbench.parent_table.clone(),
                            name: name.clone(),
                            columns,
                        }),
                        ObjectKind::PrimaryKey,
                        Some(self.database_operations.schema_workbench.parent_table.clone()),
                    ),
                    ConstraintKindUi::Unique => (
                        ObjectDefinition::UniqueConstraint(NamedColumnsDefinition {
                            schema: schema.clone(),
                            table: self.database_operations.schema_workbench.parent_table.clone(),
                            name: name.clone(),
                            columns,
                        }),
                        ObjectKind::UniqueConstraint,
                        Some(self.database_operations.schema_workbench.parent_table.clone()),
                    ),
                    ConstraintKindUi::Check => (
                        ObjectDefinition::CheckConstraint(CheckDefinition {
                            schema: schema.clone(),
                            table: self.database_operations.schema_workbench.parent_table.clone(),
                            name: name.clone(),
                            expression: self.database_operations.schema_workbench.expression.clone(),
                        }),
                        ObjectKind::CheckConstraint,
                        Some(self.database_operations.schema_workbench.parent_table.clone()),
                    ),
                    ConstraintKindUi::ForeignKey => (
                        ObjectDefinition::ForeignKey(ForeignKeyDefinition {
                            schema: schema.clone(),
                            table: self.database_operations.schema_workbench.parent_table.clone(),
                            name: name.clone(),
                            columns,
                            ref_schema: self.database_operations.schema_workbench.ref_schema.clone(),
                            ref_table: self.database_operations.schema_workbench.ref_table.clone(),
                            ref_columns: split_csv(&self.database_operations.schema_workbench.ref_columns_csv),
                            on_delete: nonempty_opt(&self.database_operations.schema_workbench.on_delete),
                            on_update: None,
                        }),
                        ObjectKind::ForeignKey,
                        Some(self.database_operations.schema_workbench.parent_table.clone()),
                    ),
                }
            }
            SchemaWorkbenchMode::Trigger => (
                ObjectDefinition::Trigger(TriggerDefinition {
                    schema: schema.clone(),
                    table: self.database_operations.schema_workbench.parent_table.clone(),
                    name: name.clone(),
                    timing: self.database_operations.schema_workbench.timing.clone(),
                    event: self.database_operations.schema_workbench.event.clone(),
                    body: self.database_operations.schema_workbench.body.clone(),
                }),
                ObjectKind::Trigger,
                Some(self.database_operations.schema_workbench.parent_table.clone()),
            ),
            SchemaWorkbenchMode::Sequence => (
                ObjectDefinition::Sequence(SequenceDefinition {
                    schema: schema.clone(),
                    name: name.clone(),
                    start: self.database_operations.schema_workbench.start.parse().ok(),
                    increment: self.database_operations.schema_workbench.increment.parse().ok(),
                    min_value: None,
                    max_value: None,
                    cache: None,
                    cycle: self.database_operations.schema_workbench.cycle,
                }),
                ObjectKind::Sequence,
                None,
            ),
            SchemaWorkbenchMode::Type => (
                ObjectDefinition::EnumType(EnumTypeDefinition {
                    schema: schema.clone(),
                    name: name.clone(),
                    values: split_csv(&self.database_operations.schema_workbench.enum_values_csv),
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
                    schema: nonempty_opt(&self.database_operations.schema_workbench.extension_schema),
                    version: None,
                    cascade: self.database_operations.schema_workbench.cascade,
                }),
                ObjectKind::Extension,
                None,
            ),
            SchemaWorkbenchMode::Comment => {
                let parent = nonempty_opt(&self.database_operations.schema_workbench.parent_table);
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
                        comment: nonempty_opt(&self.database_operations.schema_workbench.comment_text),
                    }),
                    ObjectKind::Comment,
                    parent,
                )
            }
            SchemaWorkbenchMode::Partition => (
                ObjectDefinition::Partition(PartitionDefinition {
                    schema: schema.clone(),
                    parent_table: self.database_operations.schema_workbench.parent_table.clone(),
                    name: name.clone(),
                    strategy: "RANGE".into(),
                    bound_expression: self.database_operations.schema_workbench.partition_bound.clone(),
                }),
                ObjectKind::Partition,
                Some(self.database_operations.schema_workbench.parent_table.clone()),
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
