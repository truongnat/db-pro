//! Schema Workbench mutation planning and execution.

use super::schema_workbench::QuoteDialect;
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
        match self
            .schema
            .workbench
            .build_mutation_request(action, self.active_query_driver().to_owned())
        {
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
        let request_id = self.next_request_id();
        let command = match self.schema.workbench.apply_ddl_command(request_id, connection.id) {
            Ok(command) => command,
            Err(error) => {
                self.feedback.runtime_message = error;
                return;
            }
        };
        if self.dispatch_command(command) {
            self.table.state.ddl_execution_request = Some(request_id);
            self.feedback.runtime_message = "Applying schema mutation…".into();
        }
    }
}
