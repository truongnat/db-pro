//! Schema Workbench — action handling and mutation dispatch.

use super::schema_workbench::QuoteDialect;
use super::schema_workbench_form::SchemaWorkbenchFormAction;
use super::*;
use db_pro_core::application::ObjectMutationService;
use db_pro_core::domain::object_mutation::*;

impl DbProApp {
    pub(crate) fn apply_workbench_form_action(&mut self, action: SchemaWorkbenchFormAction) {
        match action {
            SchemaWorkbenchFormAction::PlanObject(action) => {
                self.plan_workbench_action(action);
            }
            SchemaWorkbenchFormAction::PlanDatabase(action) => {
                self.plan_database_action(action);
            }
            SchemaWorkbenchFormAction::ApplyDdl => {
                self.apply_workbench_ddl();
            }
            SchemaWorkbenchFormAction::OpenSql(sql) => {
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
            .build_mutation_request(action, self.active_query_driver())
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
                if_exists: true,
                if_not_exists: true,
                dry_run: false,
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
        let request = match self.schema.workbench.prepare_ddl_request(connection.id) {
            Ok(request) => request,
            Err(error) => {
                self.feedback.runtime_message = error;
                return;
            }
        };
        let kind = format!("{:?}", self.schema.workbench.mode);
        let name = self.schema.workbench.name.clone();
        let safety = self.schema.workbench.preview_safety.clone();
        let sql = request.sql.clone();
        let command = UiCommand::ExecuteDdl {
            request_id,
            connection_id: request.connection_id,
            sql: request.sql,
        };
        if self.dispatch_command(command) {
            self.schema.workbench.record_execution(&kind, &name, &sql, &safety, true);
            self.table.state.ddl_execution_request = Some(request_id);
            self.feedback.runtime_message = "Applying schema mutation…".into();
        }
    }
}
