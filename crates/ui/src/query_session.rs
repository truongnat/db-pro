//! Active query document / connection / result session helpers.
use super::*;

/// Read-only query connection resolution across the query and connection
/// aggregates. The query feature supplies document binding; the connection
/// feature supplies the active lifecycle and saved-connection catalog.
pub(crate) struct QueryConnectionContext<'a> {
    query_session: &'a QuerySessionState,
    catalog: &'a ConnectionCatalogState,
    lifecycle: &'a ConnectionLifecycleState,
    schema_explorer: &'a SchemaExplorerState,
}

impl<'a> QueryConnectionContext<'a> {
    pub(crate) fn active_connection_id(&self) -> Option<&'a str> {
        self.query_session
            .active_connection_id()
            .or(self.lifecycle.active_connection_id())
    }

    pub(crate) fn active_connection(&self) -> Option<&'a UiConnectionSummary> {
        self.active_connection_id().and_then(|id| self.catalog.find(id))
    }

    pub(crate) fn capabilities(&self) -> CapabilityLookup {
        self.active_connection()
            .map(|connection| CapabilityLookup::for_driver_label(&connection.driver))
            .or_else(|| {
                self.lifecycle
                    .active_connection_id()
                    .and_then(|id| self.catalog.find(id))
                    .map(|connection| CapabilityLookup::for_driver_label(&connection.driver))
            })
            .unwrap_or(CapabilityLookup::NoActiveConnection)
    }

    pub(crate) fn connection_name(&self) -> &'a str {
        self.active_connection()
            .map(|connection| connection.name.as_str())
            .unwrap_or(self.lifecycle.fallback_name())
    }

    pub(crate) fn driver(&self) -> &'a str {
        self.active_connection()
            .map(|connection| connection.driver.as_str())
            .unwrap_or("PostgreSQL")
    }

    pub(crate) fn schema(&self) -> &'a str {
        self.query_session
            .active_schema()
            .unwrap_or_else(|| connection_status::active_schema(self.schema_explorer, self.catalog, self.lifecycle))
    }
}

impl DbProApp {
    // Problems / diagnostics: `problems_view.rs`.

    pub(crate) fn apply_migration_preview(&mut self) {
        if self.table.state.ddl_execution_request.is_some() {
            return;
        }
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        let sql = match self.schema.compare.prepare_migration_sql() {
            Ok(sql) => sql,
            Err(error) => {
                self.feedback.runtime_message = error;
                return;
            }
        };
        let command = UiCommand::ExecuteDdl {
            request_id,
            connection_id,
            sql,
        };
        if self.dispatch_command(command) {
            self.table.state.ddl_execution_request = Some(request_id);
            self.feedback.runtime_message = "Applying migration plan…".into();
        }
    }

    pub(crate) fn handle_transaction_action(&mut self, action: crate::components::TransactionAction) {
        if let Some(sql) = self
            .query
            .execution
            .apply_transaction_action(action, &mut self.feedback)
        {
            self.dispatch_transaction_sql(sql);
        }
    }

    pub(super) fn dispatch_transaction_sql(&mut self, sql: &str) {
        let Some(connection_id) = self.active_query_connection_id().map(str::to_owned) else {
            self.feedback.runtime_message = "Connect before using transaction controls".into();
            return;
        };
        // Reuse the normal run path so execution state / cancel / history stay consistent.
        let version = self.query.session.active_buffer_version();
        self.send_query_run(connection_id, sql.to_owned(), (0, sql.len()), version, false);
    }

    pub(crate) fn set_active_query_result(&mut self, index: usize) {
        if self.query.session.set_active_result(index) {
            self.table.data.invalidate_grid_projection();
        }
    }

    pub(crate) fn active_query_connection_id(&self) -> Option<&str> {
        self.query_connection_context().active_connection_id()
    }

    pub(crate) fn active_query_connection(&self) -> Option<&UiConnectionSummary> {
        self.query_connection_context().active_connection()
    }

    /// Capabilities for the connection the active query document is bound to.
    ///
    /// Resolved from the bound connection, then from the active connection. It
    /// deliberately does **not** go through `active_query_driver`, whose display fallback
    /// is the literal `"PostgreSQL"`: answering with PostgreSQL's set while no connection
    /// exists is the same silent-wrong-answer this lookup replaces with a named state.
    pub(crate) fn query_capabilities(&self) -> CapabilityLookup {
        self.query_connection_context().capabilities()
    }

    pub(crate) fn active_query_connection_name(&self) -> &str {
        self.query_connection_context().connection_name()
    }

    pub(crate) fn active_query_driver(&self) -> &str {
        self.query_connection_context().driver()
    }

    pub(crate) fn active_query_schema(&self) -> &str {
        self.query_connection_context().schema()
    }

    fn query_connection_context(&self) -> QueryConnectionContext<'_> {
        QueryConnectionContext {
            query_session: &self.query.session,
            catalog: &self.connection.catalog,
            lifecycle: &self.connection.lifecycle,
            schema_explorer: &self.schema.explorer,
        }
    }

    // Query documents: `query_documents.rs`.
    // Workspace actions: `workspace_actions.rs`.
}
