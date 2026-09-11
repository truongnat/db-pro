use super::*;

impl DbProApp {
    pub(super) fn reset_agent_context(&mut self) {
        self.agent_request = None;
        self.agent_pending_prompt = None;
        self.agent_pending_context = None;
        self.agent_input.clear();
        self.agent_messages.clear();
    }

    pub(super) fn agent_context(&self) -> AgentContext {
        let connection_name = Some(self.active_connection_name().to_owned());
        let driver = self.active_driver().to_owned();
        let selected_columns = self
            .table_info
            .as_ref()
            .map(|info| info.columns.iter().map(|column| column.name.clone()).collect())
            .unwrap_or_default();
        let current_sql = if self.selected_query.trim().is_empty() {
            self.query_text.clone()
        } else {
            self.selected_query.clone()
        };
        let result_summary = self
            .query_result
            .as_ref()
            .or(self.table_data_result.as_ref())
            .map(|result| format!("{} rows returned in {} ms", result.row_count, result.duration_ms));
        let last_error = self.has_runtime_error().then(|| self.runtime_message.clone());

        AgentContext {
            connection_name,
            driver,
            tables: self.active_schema_table_names(),
            columns: self.active_schema_column_names(),
            schema: Some(self.active_schema().to_owned()),
            selected_table: self.selected_table.clone(),
            selected_columns,
            current_sql,
            result_summary,
            explain_plan: self.explain_plan.clone(),
            last_error,
        }
    }

    pub(super) fn submit_agent_prompt(&mut self) {
        let prompt = self.agent_input.trim().to_owned();
        if prompt.is_empty() {
            return;
        }

        self.agent_messages.push(AgentMessage {
            role: AgentRole::User,
            content: prompt.clone(),
            sql: None,
            requires_confirmation: false,
        });
        let context = self.agent_context();
        let request_id = self.task_bridge.next_request_id();
        self.agent_request = Some(request_id);
        self.agent_pending_prompt = Some(prompt.clone());
        self.agent_pending_context = Some(context.clone());
        self.agent_input.clear();
        self.runtime_message = "Sending request to Codex…".to_owned();
        if self
            .task_bridge
            .send(UiCommand::RunAgent {
                request_id,
                prompt,
                context,
            })
            .is_err()
        {
            self.agent_request = None;
            self.runtime_message = "Agent runtime unavailable · using offline draft".to_owned();
            self.fallback_agent_response(None);
        }
    }

    pub(super) fn fallback_agent_response(&mut self, reason: Option<&str>) {
        let Some(prompt) = self.agent_pending_prompt.take() else {
            return;
        };
        let context = self.agent_pending_context.take().unwrap_or_default();
        let mut response = self
            .agent_provider
            .respond(&prompt, &context)
            .unwrap_or_else(|error| AgentMessage {
                role: AgentRole::Assistant,
                content: format!("Agent provider unavailable: {error}"),
                sql: None,
                requires_confirmation: false,
            });
        if let Some(reason) = reason {
            response.content = format!("{reason}\n\n{}", response.content);
        }
        let info = self.agent_provider.info();
        self.agent_provider_label = info.label.to_owned();
        self.agent_provider_detail = info.detail.to_owned();
        self.agent_messages.push(response);
    }
}
