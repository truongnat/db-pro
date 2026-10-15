//! Schema Workbench — action handling and mutation dispatch.

use super::schema_workbench::{SchemaWorkbenchMode, WorkbenchPendingPlan};
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
                // Bind the document to the connection that produced the plan —
                // the query-context default may point at another database (#350).
                self.new_query_document();
                let index = self.query.session.documents.len().saturating_sub(1);
                if let Some(doc) = self.query.session.documents.last_mut() {
                    doc.set_text(sql);
                }
                if let Some(connection_id) = self.schema.workbench.preview_connection_id.clone() {
                    self.set_document_connection(index, Some(connection_id));
                }
                let schema = self.schema.workbench.schema.clone();
                if !schema.is_empty() {
                    self.set_document_schema(index, Some(schema));
                }
                self.workspace.active_tab = WorkspaceTab::Query;
                self.workspace.activity = Activity::Queries;
            }
        }
    }

    pub(crate) fn plan_drop_schema(&mut self, schema: &str) {
        let schema = schema.trim();
        let driver = self.active_driver().to_owned();
        if schema.is_empty() || !is_user_visible_schema(&driver, schema) {
            self.feedback.runtime_message = format!("Cannot drop schema `{schema}`");
            return;
        }
        if driver.eq_ignore_ascii_case("sqlite") {
            self.feedback.runtime_message = "SQLite has no DROP SCHEMA".to_owned();
            return;
        }
        self.schema.workbench.mode = SchemaWorkbenchMode::SchemaDb;
        self.schema.workbench.schema = schema.to_owned();
        self.schema.workbench.name = schema.to_owned();
        self.plan_workbench_action(ObjectAction::Drop);
        // Only open the confirm dialog when the plan produced executable SQL —
        // a failed or empty plan must not confirm against nothing (#346).
        if self.schema.workbench.preview_error.is_none()
            && !self.schema.workbench.preview_sql.trim().is_empty()
        {
            self.schema.workbench.apply_confirmation = true;
        }
        // The Apply-DDL confirm renders at app level — stay on the current
        // surface instead of dragging the user to Schema Workbench (#334).
        self.feedback
            .runtime_message = format!("Planned DROP SCHEMA \"{schema}\" CASCADE");
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
                self.schema.workbench.pending_plan = None;
            }
        }
    }

    pub(crate) fn plan_database_action(&mut self, action: ObjectAction) {
        // Flags follow the action kind; CASCADE is never valid on DROP
        // DATABASE (#354).
        let request = self
            .schema
            .workbench
            .build_database_request(action, self.active_query_driver());
        self.run_plan(request);
    }

    pub(crate) fn run_plan(&mut self, request: ObjectMutationRequest) {
        // Remember how this plan was built so apply can re-derive it and reject
        // a form mutated after the preview (#364/#353).
        let pending = match request.target.as_ref().map(|target| target.kind) {
            Some(ObjectKind::Database) => Some(WorkbenchPendingPlan::Database(request.action)),
            _ => Some(WorkbenchPendingPlan::Object(request.action)),
        };
        self.schema.workbench.preview_connection_id =
            self.active_query_connection_id().map(str::to_owned);
        match ObjectMutationService::plan_for_driver(&request) {
            Ok(preview) => {
                if let Some(reason) = preview.unsupported_reason {
                    self.schema.workbench.preview_error = Some(reason);
                    self.schema.workbench.preview_sql.clear();
                    self.schema.workbench.pending_plan = None;
                } else {
                    self.schema.workbench.preview_error = None;
                    self.schema.workbench.preview_sql = preview.statements.join(";\n");
                    if !self.schema.workbench.preview_sql.is_empty() {
                        self.schema.workbench.preview_sql.push(';');
                    }
                    self.schema.workbench.pending_plan = pending;
                }
                self.schema.workbench.preview_safety = preview.safety;
                self.schema.workbench.preview_fingerprint = preview.fingerprint;
            }
            Err(err) => {
                self.schema.workbench.preview_error = Some(err.to_string());
                self.schema.workbench.preview_sql.clear();
                self.schema.workbench.pending_plan = None;
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
        // Re-plan from the current form inputs and refuse to apply a preview
        // whose fingerprint no longer matches (#364/#353).
        let driver = self.active_query_driver().to_owned();
        match self.schema.workbench.replan_fingerprint(&driver) {
            Ok(live) if live != self.schema.workbench.preview_fingerprint => {
                self.schema.workbench.apply_confirmation = false;
                self.feedback.runtime_message =
                    "Plan is stale — regenerate preview".to_owned();
                return;
            }
            Ok(_) => {}
            Err(error) => {
                self.schema.workbench.apply_confirmation = false;
                self.feedback.runtime_message = error;
                return;
            }
        }
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
            // The entry starts as pending (success=false) and is patched by the DdlCompleted /
            // failure event — dispatching only proves the command was queued.
            let index = self.schema.workbench.record_execution(&kind, &name, &sql, &safety, false);
            self.schema.workbench.pending_execution = Some((request_id, index));
            self.table.state.ddl_execution_request = Some(request_id);
            self.feedback.runtime_message = "Applying schema mutation…".into();
        }
    }
}
