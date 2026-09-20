//! Schema Workbench mutation planning and execution.

use super::schema_workbench::{
    nonempty_opt, parse_column_defs, split_csv, ConstraintKindUi, QuoteDialect, SchemaWorkbenchMode,
};
use super::*;
use db_pro_core::application::ObjectMutationService;
use db_pro_core::domain::object_mutation::*;

impl DbProApp {
    pub(crate) fn apply_workbench_form_action(&mut self, action: schema_workbench_form::SchemaWorkbenchFormAction) {
        match action {
            schema_workbench_form::SchemaWorkbenchFormAction::PlanObject(action) => self.plan_workbench_action(action),
            schema_workbench_form::SchemaWorkbenchFormAction::PlanDatabase(action) => self.plan_database_action(action),
            schema_workbench_form::SchemaWorkbenchFormAction::ApplyDdl => self.apply_workbench_ddl(),
            schema_workbench_form::SchemaWorkbenchFormAction::OpenSql(sql) => {
                self.new_query_document();
                if let Some(doc) = self.query.session.documents.last_mut() {
                    doc.set_text(sql);
                }
                self.workspace.active_tab = WorkspaceTab::Query;
                self.workspace.activity = Activity::Queries;
            }
        }
    }

    pub(crate) fn plan_workbench_action(&mut self, action: ObjectAction) {
        match self.build_mutation_request(action) {
            Ok(request) => self.run_plan(request),
            Err(err) => {
                self.schema.workbench.preview_error = Some(err);
                self.schema.workbench.preview_sql.clear();
            }
        }
    }

    pub(crate) fn plan_database_action(&mut self, action: ObjectAction) {
        let request = ObjectMutationRequest {
            action,
            target: Some(ObjectRef {
                kind: ObjectKind::Database,
                schema: None,
                name: self.schema.workbench.name.clone(),
                parent: None,
            }),
            definition: ObjectDefinition::Database(DatabaseDefinition {
                name: self.schema.workbench.name.clone(),
                owner: None,
                template: None,
            }),
            options: MutationOptions {
                cascade: self.schema.workbench.cascade,
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
                    self.schema.workbench.preview_error = Some(reason);
                    self.schema.workbench.preview_sql.clear();
                } else {
                    self.schema.workbench.preview_error = None;
                    self.schema.workbench.preview_sql = preview.statements.join(";\n");
                    if !self.schema.workbench.preview_sql.is_empty() {
                        self.schema.workbench.preview_sql.push(';');
                    }
                }
                self.schema.workbench.preview_safety = preview.safety;
                self.schema.workbench.preview_fingerprint = preview.fingerprint;
            }
            Err(err) => {
                self.schema.workbench.preview_error = Some(err.to_string());
                self.schema.workbench.preview_sql.clear();
            }
        }
    }

    pub(crate) fn apply_workbench_ddl(&mut self) {
        if !self.can_mutate_active_connection() {
            self.feedback.runtime_message = "Connect with write access to apply DDL".into();
            return;
        }
        if self.table.state.ddl_execution_request.is_some() {
            return;
        }
        let Some(connection) = self.active_connection().cloned() else {
            self.feedback.runtime_message = "Connect to a database before applying DDL".into();
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        let command = match self.schema.workbench.apply_ddl_command(request_id, connection.id) {
            Ok(command) => command,
            Err(error) => {
                self.feedback.runtime_message = error;
                return;
            }
        };
        self.dispatch_command(command);
        self.table.state.ddl_execution_request = Some(request_id);
        self.feedback.runtime_message = "Applying schema mutation…".into();
    }

    pub(crate) fn build_mutation_request(&self, action: ObjectAction) -> Result<ObjectMutationRequest, String> {
        let driver = self.active_query_driver().to_owned();
        let schema = self.schema.workbench.schema.clone();
        let name = self.schema.workbench.name.clone();
        if name.trim().is_empty()
            && !matches!(
                self.schema.workbench.mode,
                SchemaWorkbenchMode::Dependencies | SchemaWorkbenchMode::Docs
            )
        {
            return Err("Name is required".into());
        }
        let options = MutationOptions {
            cascade: self.schema.workbench.cascade,
            ..MutationOptions::default()
        };

        let (definition, kind, parent) = match self.schema.workbench.mode {
            SchemaWorkbenchMode::Table => {
                let columns = parse_column_defs(&schema, &name, &self.schema.workbench.columns_csv)?;
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
                    table: self.schema.workbench.parent_table.clone(),
                    name: name.clone(),
                    data_type: self.schema.workbench.data_type.clone(),
                    nullable: self.schema.workbench.nullable,
                    default: nonempty_opt(&self.schema.workbench.default_expr),
                    is_pk: self.schema.workbench.is_pk,
                    new_name: nonempty_opt(&self.schema.workbench.new_name),
                }),
                ObjectKind::Column,
                Some(self.schema.workbench.parent_table.clone()),
            ),
            SchemaWorkbenchMode::View => {
                let def = ViewDefinition {
                    schema: schema.clone(),
                    name: name.clone(),
                    select_sql: self.schema.workbench.select_sql.clone(),
                    materialized: self.schema.workbench.materialized,
                    replace: false,
                };
                let kind = if self.schema.workbench.materialized {
                    ObjectKind::MaterializedView
                } else {
                    ObjectKind::View
                };
                let definition = if self.schema.workbench.materialized {
                    ObjectDefinition::MaterializedView(def)
                } else {
                    ObjectDefinition::View(def)
                };
                (definition, kind, None)
            }
            SchemaWorkbenchMode::Index => (
                ObjectDefinition::Index(IndexDefinition {
                    schema: schema.clone(),
                    table: self.schema.workbench.parent_table.clone(),
                    name: name.clone(),
                    columns: split_csv(&self.schema.workbench.columns_csv),
                    unique: self.schema.workbench.unique,
                    method: None,
                    predicate: None,
                }),
                ObjectKind::Index,
                Some(self.schema.workbench.parent_table.clone()),
            ),
            SchemaWorkbenchMode::Constraint => {
                let columns = split_csv(&self.schema.workbench.columns_csv);
                match self.schema.workbench.constraint_kind {
                    ConstraintKindUi::PrimaryKey => (
                        ObjectDefinition::PrimaryKey(NamedColumnsDefinition {
                            schema: schema.clone(),
                            table: self.schema.workbench.parent_table.clone(),
                            name: name.clone(),
                            columns,
                        }),
                        ObjectKind::PrimaryKey,
                        Some(self.schema.workbench.parent_table.clone()),
                    ),
                    ConstraintKindUi::Unique => (
                        ObjectDefinition::UniqueConstraint(NamedColumnsDefinition {
                            schema: schema.clone(),
                            table: self.schema.workbench.parent_table.clone(),
                            name: name.clone(),
                            columns,
                        }),
                        ObjectKind::UniqueConstraint,
                        Some(self.schema.workbench.parent_table.clone()),
                    ),
                    ConstraintKindUi::Check => (
                        ObjectDefinition::CheckConstraint(CheckDefinition {
                            schema: schema.clone(),
                            table: self.schema.workbench.parent_table.clone(),
                            name: name.clone(),
                            expression: self.schema.workbench.expression.clone(),
                        }),
                        ObjectKind::CheckConstraint,
                        Some(self.schema.workbench.parent_table.clone()),
                    ),
                    ConstraintKindUi::ForeignKey => (
                        ObjectDefinition::ForeignKey(ForeignKeyDefinition {
                            schema: schema.clone(),
                            table: self.schema.workbench.parent_table.clone(),
                            name: name.clone(),
                            columns,
                            ref_schema: self.schema.workbench.ref_schema.clone(),
                            ref_table: self.schema.workbench.ref_table.clone(),
                            ref_columns: split_csv(&self.schema.workbench.ref_columns_csv),
                            on_delete: nonempty_opt(&self.schema.workbench.on_delete),
                            on_update: None,
                        }),
                        ObjectKind::ForeignKey,
                        Some(self.schema.workbench.parent_table.clone()),
                    ),
                }
            }
            SchemaWorkbenchMode::Trigger => (
                ObjectDefinition::Trigger(TriggerDefinition {
                    schema: schema.clone(),
                    table: self.schema.workbench.parent_table.clone(),
                    name: name.clone(),
                    timing: self.schema.workbench.timing.clone(),
                    event: self.schema.workbench.event.clone(),
                    body: self.schema.workbench.body.clone(),
                }),
                ObjectKind::Trigger,
                Some(self.schema.workbench.parent_table.clone()),
            ),
            SchemaWorkbenchMode::Sequence => (
                ObjectDefinition::Sequence(SequenceDefinition {
                    schema: schema.clone(),
                    name: name.clone(),
                    start: self.schema.workbench.start.parse().ok(),
                    increment: self.schema.workbench.increment.parse().ok(),
                    min_value: None,
                    max_value: None,
                    cache: None,
                    cycle: self.schema.workbench.cycle,
                }),
                ObjectKind::Sequence,
                None,
            ),
            SchemaWorkbenchMode::Type => (
                ObjectDefinition::EnumType(EnumTypeDefinition {
                    schema: schema.clone(),
                    name: name.clone(),
                    values: split_csv(&self.schema.workbench.enum_values_csv),
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
                    schema: nonempty_opt(&self.schema.workbench.extension_schema),
                    version: None,
                    cascade: self.schema.workbench.cascade,
                }),
                ObjectKind::Extension,
                None,
            ),
            SchemaWorkbenchMode::Comment => {
                let parent = nonempty_opt(&self.schema.workbench.parent_table);
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
                        comment: nonempty_opt(&self.schema.workbench.comment_text),
                    }),
                    ObjectKind::Comment,
                    parent,
                )
            }
            SchemaWorkbenchMode::Partition => (
                ObjectDefinition::Partition(PartitionDefinition {
                    schema: schema.clone(),
                    parent_table: self.schema.workbench.parent_table.clone(),
                    name: name.clone(),
                    strategy: "RANGE".into(),
                    bound_expression: self.schema.workbench.partition_bound.clone(),
                }),
                ObjectKind::Partition,
                Some(self.schema.workbench.parent_table.clone()),
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
