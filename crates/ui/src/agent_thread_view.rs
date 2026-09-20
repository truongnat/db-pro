use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};

use super::agent_workflow_state::{
    AgentUiActivity, AgentUiActivityStatus, AgentUiConfirmation, AgentUiSession, AgentUiToolResult,
};
use std::collections::HashMap;

impl DbProApp {
    pub(super) fn draw_agent_thread(&mut self, ui: &mut egui::Ui, _copy_sql: &mut Option<String>) -> bool {
        self.draw_typed_agent_thread(ui)
    }

    fn draw_typed_agent_thread(&mut self, ui: &mut egui::Ui) -> bool {
        let Some(document_id) = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
            .map(|document| document.id.clone())
        else {
            return false;
        };
        let connection_id = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
            .and_then(|document| document.connection_id.clone())
            .or_else(|| self.connection.lifecycle.active_connection_id().map(str::to_owned));
        let schema = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
            .and_then(|document| document.schema.clone())
            .or_else(|| Some(self.active_schema().to_owned()));
        let session = self
            .agent
            .sessions
            .entry(document_id.clone())
            .or_insert_with(|| AgentUiSession::for_document(&document_id, connection_id, schema));
        let messages = session.messages.clone();
        let activities = session.activities.clone();
        let tool_results = session.tool_results.clone();
        let streaming_text = session.streaming_text.clone();
        let pending = session.pending_confirmation.clone();
        let session_state = session.state;
        let running = session.request_id.is_some() || session.active_run_id.is_some();

        let mut open_result_call_id = None;
        let mut retry = false;
        let mut submit = false;
        let messages_height = (ui.available_height() - 86.0).max(160.0);
        egui::ScrollArea::vertical()
            .max_height(messages_height)
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if messages.is_empty() && activities.is_empty() && streaming_text.is_empty() && pending.is_none() {
                    submit |= self.draw_agent_empty_state(ui);
                }
                self.draw_agent_messages(ui, &messages, &streaming_text);
                open_result_call_id = self.draw_agent_activities(ui, &activities, &tool_results);
                if let Some(pending) = pending.as_ref() {
                    self.draw_agent_confirmation(ui, pending, &document_id);
                }
                retry |= self.draw_agent_retry(ui, session_state);
                if running {
                    self.draw_agent_thinking(ui);
                }
            });
        if let Some(call_id) = open_result_call_id {
            self.open_agent_result_in_workspace(&call_id);
        }
        if retry {
            self.retry_agent_run();
        }
        ui.add_space(SPACE_XS);
        ui.separator();
        ui.add_space(SPACE_XS);
        submit
    }

    fn draw_agent_empty_state(&mut self, ui: &mut egui::Ui) -> bool {
        let mut submit = false;
        ui.add_space(18.0);
        ui.vertical_centered(|ui| {
            ui.label(icon_text(Icon::Bot, "", self.theme.accent));
            ui.add_space(6.0);
            ui.label(RichText::new("Database Agent").strong().color(self.theme.text_primary));
            ui.label(
                RichText::new("Ask for schema info, propose SQL edits, or run safe queries.")
                    .small()
                    .color(self.theme.text_secondary),
            );
        });
        ui.add_space(18.0);
        for suggestion in [
            "Show me the schema overview",
            "Count rows in the active table",
            "Explain query performance",
        ] {
            if Button::new(self.theme)
                .icon(Icon::WandSparkles)
                .text(suggestion)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.agent.input = suggestion.to_owned();
                submit = true;
            }
        }
        submit
    }

    fn draw_agent_messages(&self, ui: &mut egui::Ui, messages: &[AgentMessage], streaming_text: &str) {
        for message in messages {
            let is_user = message.role == AgentRole::User;
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                agent_message_frame(self.theme, is_user).show(ui, |ui| {
                    ui.label(RichText::new(message.content.clone()).color(self.theme.text_primary));
                });
            });
            ui.add_space(SPACE_XS);
        }
        if !streaming_text.is_empty() {
            agent_message_frame(self.theme, false).show(ui, |ui| {
                ui.label(icon_text(Icon::Sparkles, "Agent", self.theme.accent));
                ui.label(RichText::new(streaming_text).color(self.theme.text_primary));
            });
        }
    }

    fn draw_agent_activities(
        &mut self,
        ui: &mut egui::Ui,
        activities: &[AgentUiActivity],
        tool_results: &HashMap<String, AgentUiToolResult>,
    ) -> Option<String> {
        let mut open_result_call_id = None;
        for activity in activities {
            let status_str = match activity.status {
                AgentUiActivityStatus::Running => "Running",
                AgentUiActivityStatus::AwaitingConfirmation => "Needs approval",
                AgentUiActivityStatus::Success => "Done",
                AgentUiActivityStatus::Failed => "Failed",
                AgentUiActivityStatus::Cancelled => "Cancelled",
            };
            let duration_str = activity
                .duration_ms
                .map(|duration| format!(" · {duration} ms"))
                .unwrap_or_default();
            ui.label(
                RichText::new(format!("{status_str} · {}{duration_str}", activity.label))
                    .small()
                    .color(self.theme.text_secondary),
            );
            if let Some(call_id) = &activity.call_id {
                if let Some(tool_result) = tool_results.get(call_id) {
                    if let db_pro_core::domain::agent::AgentToolOutput::QueryResult {
                        summary,
                        result_count,
                        statement_index,
                    } = &tool_result.output
                    {
                        let sample_len = summary.sample_rows.len();
                        let total_rows = summary.row_count.unwrap_or(sample_len as u64);
                        let is_sampled = total_rows > sample_len as u64;
                        let rows_str = if is_sampled {
                            format!("Showing {sample_len} sampled rows of {total_rows}")
                        } else {
                            format!("{total_rows} rows")
                        };
                        let cols_str = format!("{} cols", summary.columns.len());
                        let count_str = if *result_count > 1 {
                            if let Some(index) = statement_index {
                                format!(" (statement #{})", index + 1)
                            } else {
                                format!(" ({result_count} results)")
                            }
                        } else {
                            String::new()
                        };
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("↳ {rows_str}, {cols_str}{count_str}"))
                                    .small()
                                    .color(self.theme.text_muted),
                            );
                            let button_label = if is_sampled {
                                "Open sample in Results"
                            } else {
                                "Open in Results"
                            };
                            if Button::new(self.theme)
                                .icon(Icon::Table2)
                                .text(button_label)
                                .variant(ButtonVariant::Secondary)
                                .size(ButtonSize::Sm)
                                .show(ui)
                                .clicked()
                            {
                                open_result_call_id = Some(call_id.clone());
                            }
                        });
                    }
                }
            }
        }
        open_result_call_id
    }

    fn draw_agent_confirmation(&mut self, ui: &mut egui::Ui, pending: &AgentUiConfirmation, document_id: &str) {
        ui.add_space(SPACE_SM);
        editor_frame(self.theme).show(ui, |ui| {
            ui.label(
                RichText::new(super::agent_view::agent_confirmation_title(pending.kind))
                    .strong()
                    .color(self.theme.text_primary),
            );
            if pending.document_id != document_id {
                ui.label(
                    RichText::new(format!("Target query: {}", pending.document_id))
                        .small()
                        .color(self.theme.accent),
                );
            }
            self.draw_agent_patch_preview(ui, pending.preview.as_ref());
            if pending.preview.is_none() {
                self.draw_agent_confirmation_notice(ui, pending.kind);
            }
            self.draw_agent_confirmation_actions(ui, pending.kind);
        });
    }

    fn draw_agent_patch_preview(
        &self,
        ui: &mut egui::Ui,
        preview: Option<&db_pro_core::domain::agent::AgentToolOutput>,
    ) {
        let Some(db_pro_core::domain::agent::AgentToolOutput::PatchPreview { original, proposed, .. }) = preview else {
            return;
        };
        ui.label(
            RichText::new("Proposed SQL change")
                .small()
                .color(self.theme.text_secondary),
        );
        egui::ScrollArea::vertical()
            .max_height(120.0)
            .id_salt("patch-diff-scroll")
            .show(ui, |ui| {
                if !original.is_empty() {
                    ui.label(
                        RichText::new(format!("- {original}"))
                            .monospace()
                            .color(self.theme.danger),
                    );
                }
                ui.label(
                    RichText::new(format!("+ {proposed}"))
                        .monospace()
                        .color(self.theme.success),
                );
            });
    }

    fn draw_agent_confirmation_notice(
        &self,
        ui: &mut egui::Ui,
        kind: db_pro_core::domain::agent_workflow::AgentConfirmationKind,
    ) {
        let message = match kind {
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunMutation => {
                "Warning: This mutation will modify database data or schema."
            }
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunDestructive => {
                "Caution: Destructive query may irreversibly drop or truncate data."
            }
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunUnknown => {
                "Classification unknown: Review the query carefully before running."
            }
            _ => "Review the requested action before continuing.",
        };
        let color = match kind {
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunMutation => self.theme.warning,
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunDestructive => self.theme.danger,
            _ => self.theme.text_secondary,
        };
        ui.label(RichText::new(message).small().color(color));
    }

    fn draw_agent_confirmation_actions(
        &mut self,
        ui: &mut egui::Ui,
        kind: db_pro_core::domain::agent_workflow::AgentConfirmationKind,
    ) {
        let approve_label = match kind {
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::ApplyPatch => "Apply Change",
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunReadOnly => "Run Query",
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunMutation => "Run Mutation",
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunDestructive => "Execute Destructive Query",
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunUnknown => "Run Unclassified Query",
        };
        let is_destructive = matches!(
            kind,
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunDestructive
        );
        let approve_variant = if is_destructive {
            ButtonVariant::Destructive
        } else {
            ButtonVariant::Default
        };
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .icon(Icon::Check)
                .text(approve_label)
                .variant(approve_variant)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.agent_confirmation_action(true);
            }
            if Button::new(self.theme)
                .icon(Icon::X)
                .text("Reject")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.agent_confirmation_action(false);
            }
        });
    }

    fn draw_agent_retry(
        &mut self,
        ui: &mut egui::Ui,
        session_state: db_pro_core::domain::agent::AgentSessionState,
    ) -> bool {
        if session_state != db_pro_core::domain::agent::AgentSessionState::Failed {
            return false;
        }
        ui.add_space(SPACE_SM);
        let mut retry = false;
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .icon(Icon::RotateCcw)
                .text("Retry")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                retry = true;
            }
        });
        retry
    }

    fn draw_agent_thinking(&self, ui: &mut egui::Ui) {
        ui.add_space(SPACE_SM);
        AgentThinking::new("Working in this query…", &mut true, self.theme)
            .is_active(true)
            .show(ui);
    }
}
