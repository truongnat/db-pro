//! Schema Workbench — Phase A object mutation UI (#183–#190, #207, #216–#218, #249–#250).

use crate::UiTableSummary;
use super::*;
use db_pro_core::domain::object_mutation::*;
use db_pro_core::ports::SqlDialect;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TableDesignerColumn {
    pub(super) name: String,
    pub(super) data_type: String,
    pub(super) nullable: bool,
    pub(super) is_pk: bool,
    pub(super) auto_increment: bool,
    pub(super) default_expr: String,
    pub(super) comment: String,
}

impl Default for TableDesignerColumn {
    fn default() -> Self {
        Self {
            name: "id".into(),
            data_type: "BIGINT".into(),
            nullable: false,
            is_pk: true,
            auto_increment: true,
            default_expr: String::new(),
            comment: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct SchemaWorkbenchState {
    pub(super) mode: SchemaWorkbenchMode,
    pub(super) schema: String,
    pub(super) name: String,
    pub(super) parent_table: String,
    pub(super) table_columns: Vec<TableDesignerColumn>,
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
            table_columns: vec![
                TableDesignerColumn {
                    name: "id".into(),
                    data_type: "BIGINT".into(),
                    nullable: false,
                    is_pk: true,
                    auto_increment: true,
                    default_expr: String::new(),
                    comment: "Primary Key".into(),
                },
                TableDesignerColumn {
                    name: "created_at".into(),
                    data_type: "TIMESTAMPTZ".into(),
                    nullable: false,
                    is_pk: false,
                    auto_increment: false,
                    default_expr: "CURRENT_TIMESTAMP".into(),
                    comment: "Created timestamp".into(),
                },
            ],
            data_type: "INTEGER".into(),
            select_sql: "SELECT 1".into(),
            columns_csv: "id:INTEGER:pk,name:TEXT".into(),
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
    pub(super) fn prepare_ddl_request(&self, connection_id: String) -> Result<SchemaWorkbenchDdlRequest, String> {
        let sql = self.preview_sql.trim();
        if sql.is_empty() {
            return Err("Plan a mutation before applying".to_owned());
        }
        Ok(SchemaWorkbenchDdlRequest {
            connection_id,
            sql: sql.to_owned(),
        })
    }

    pub(super) fn add_table_column(&mut self) {
        let next_idx = self.table_columns.len() + 1;
        self.table_columns.push(TableDesignerColumn {
            name: format!("col_{next_idx}"),
            data_type: "VARCHAR(255)".into(),
            nullable: true,
            is_pk: false,
            auto_increment: false,
            default_expr: String::new(),
            comment: String::new(),
        });
        self.sync_table_columns_to_csv();
    }

    pub(super) fn remove_table_column(&mut self, index: usize) {
        if index < self.table_columns.len() {
            self.table_columns.remove(index);
            self.sync_table_columns_to_csv();
        }
    }

    pub(super) fn move_table_column_up(&mut self, index: usize) {
        if index > 0 && index < self.table_columns.len() {
            self.table_columns.swap(index, index - 1);
            self.sync_table_columns_to_csv();
        }
    }

    pub(super) fn move_table_column_down(&mut self, index: usize) {
        if index + 1 < self.table_columns.len() {
            self.table_columns.swap(index, index + 1);
            self.sync_table_columns_to_csv();
        }
    }

    pub(super) fn sync_table_columns_to_csv(&mut self) {
        let parts: Vec<String> = self
            .table_columns
            .iter()
            .map(|col| {
                let mut s = format!("{}:{}", col.name, col.data_type);
                if col.is_pk {
                    s.push_str(":pk");
                } else if !col.nullable {
                    s.push_str(":nn");
                }
                s
            })
            .collect();
        self.columns_csv = parts.join(",");
    }

    pub(super) fn load_from_table_details(&mut self, details: &UiTableSummary) {
        self.schema = details.schema.clone();
        self.name = details.name.clone();
        self.parent_table = details.name.clone();
        self.table_columns = details
            .columns
            .iter()
            .map(|col| TableDesignerColumn {
                name: col.name.clone(),
                data_type: col.data_type.clone(),
                nullable: col.nullable,
                is_pk: col.is_primary_key,
                auto_increment: false,
                default_expr: String::new(),
                comment: String::new(),
            })
            .collect();
        self.sync_table_columns_to_csv();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SchemaWorkbenchDdlRequest {
    pub(super) connection_id: String,
    pub(super) sql: String,
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
        let driver = self.active_query_driver().to_owned();
        let action = {
            let mut context = schema_workbench_surface_view::SchemaWorkbenchSurfaceContext {
                theme: self.theme,
                workbench: &mut self.schema.workbench,
                can_mutate: false,
                driver: &driver,
                edges: &[],
            };
            context.draw_sidebar(ui)
        };
        if let Some(schema_workbench_surface_view::SchemaWorkbenchSurfaceAction::SelectMode(mode)) = action {
            self.schema.workbench.mode = mode;
            self.workspace.active_tab = WorkspaceTab::SchemaWorkbench;
        }
    }

    pub(super) fn draw_schema_workbench(&mut self, ui: &mut egui::Ui) {
        let edges = self.collect_ui_dependency_edges();
        let driver = self.active_query_driver().to_owned();
        let actions = {
            let can_mutate = self.can_mutate_active_connection();
            let mut context = schema_workbench_surface_view::SchemaWorkbenchSurfaceContext {
                theme: self.theme,
                workbench: &mut self.schema.workbench,
                can_mutate,
                driver: &driver,
                edges: &edges,
            };
            context.draw_main(ui)
        };
        for action in actions {
            self.apply_schema_workbench_surface_action(action);
        }
    }

    fn apply_schema_workbench_surface_action(
        &mut self,
        action: schema_workbench_surface_view::SchemaWorkbenchSurfaceAction,
    ) {
        match action {
            schema_workbench_surface_view::SchemaWorkbenchSurfaceAction::SelectMode(mode) => {
                self.schema.workbench.mode = mode;
                self.workspace.active_tab = WorkspaceTab::SchemaWorkbench;
            }
            schema_workbench_surface_view::SchemaWorkbenchSurfaceAction::Form(action) => {
                self.apply_workbench_form_action(action);
            }
            schema_workbench_surface_view::SchemaWorkbenchSurfaceAction::Secondary(action) => {
                self.apply_schema_workbench_secondary_action(action);
            }
        }
    }

    fn apply_schema_workbench_secondary_action(
        &mut self,
        action: schema_workbench_secondary_view::SchemaWorkbenchSecondaryAction,
    ) {
        match action {
            schema_workbench_secondary_view::SchemaWorkbenchSecondaryAction::GenerateMarkdown => {
                self.schema.workbench.docs_format_html = false;
                self.schema.workbench.docs_markdown = self.export_schema_docs_markdown();
            }
            schema_workbench_secondary_view::SchemaWorkbenchSecondaryAction::GenerateHtml => {
                self.schema.workbench.docs_format_html = true;
                let md = self.export_schema_docs_markdown();
                self.schema.workbench.docs_markdown = format!(
                    "<!DOCTYPE html><html><body><pre>{}</pre></body></html>",
                    md.replace('&', "&amp;").replace('<', "&lt;")
                );
            }
            schema_workbench_secondary_view::SchemaWorkbenchSecondaryAction::OpenDocsAsQuery(body) => {
                self.new_query_document();
                if let Some(doc) = self.query.session.documents.last_mut() {
                    doc.set_text(format!("-- Schema docs export\n/*\n{body}\n*/"));
                }
                self.workspace.active_tab = WorkspaceTab::Query;
            }
        }
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
    }
}
