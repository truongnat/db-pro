//! Agent thread presentation and intent collection.
use super::agent_workflow_state::{
    AgentUiActivity, AgentUiActivityStatus, AgentUiConfirmation, AgentUiSession, AgentUiToolResult,
};
use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum AgentThreadAction {
    Submit(String),
    OpenResult(String),
    Retry,
    Confirm(bool),
}

pub(super) struct AgentThreadSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) document_id: &'a str,
    pub(super) session: &'a AgentUiSession,
}

impl AgentThreadSurfaceContext<'_> {
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> Vec<AgentThreadAction> {
        let mut actions = Vec::new();
        let messages_height = (ui.available_height() - 86.0).max(160.0);
        egui::ScrollArea::vertical()
            .max_height(messages_height)
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if self.session.messages.is_empty()
                    && self.session.activities.is_empty()
                    && self.session.streaming_text.is_empty()
                    && self.session.pending_confirmation.is_none()
                {
                    if let Some(prompt) = self.draw_empty_state(ui) {
                        actions.push(AgentThreadAction::Submit(prompt));
                    }
                }
                self.draw_messages(ui);
                if let Some(call_id) = self.draw_activities(ui, &self.session.activities, &self.session.tool_results) {
                    actions.push(AgentThreadAction::OpenResult(call_id));
                }
                if let Some(pending) = self.session.pending_confirmation.as_ref() {
                    if let Some(action) = self.draw_confirmation(ui, pending) {
                        actions.push(action);
                    }
                }
                if self.session.state == db_pro_core::domain::agent::AgentSessionState::Failed {
                    if self.draw_retry(ui) {
                        actions.push(AgentThreadAction::Retry);
                    }
                }
                if self.session.request_id.is_some() || self.session.active_run_id.is_some() {
                    self.draw_thinking(ui);
                }
            });
        ui.add_space(SPACE_XS);
        ui.separator();
        ui.add_space(SPACE_XS);
        actions
    }

    fn draw_empty_state(&self, ui: &mut egui::Ui) -> Option<String> {
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
                return Some(suggestion.to_owned());
            }
        }
        None
    }

    fn draw_messages(&self, ui: &mut egui::Ui) {
        for message in &self.session.messages {
            let is_user = message.role == AgentRole::User;
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                agent_message_frame(self.theme, is_user).show(ui, |ui| {
                    ui.label(RichText::new(message.content.clone()).color(self.theme.text_primary));
                });
            });
            ui.add_space(SPACE_XS);
        }
        if !self.session.streaming_text.is_empty() {
            agent_message_frame(self.theme, false).show(ui, |ui| {
                ui.label(icon_text(Icon::Sparkles, "Agent", self.theme.accent));
                ui.label(RichText::new(&self.session.streaming_text).color(self.theme.text_primary));
            });
        }
    }

    fn draw_activities(
        &self,
        ui: &mut egui::Ui,
        activities: &[AgentUiActivity],
        tool_results: &HashMap<String, AgentUiToolResult>,
    ) -> Option<String> {
        let mut open_result_call_id = None;
        for activity in activities {
            let status = match activity.status {
                AgentUiActivityStatus::Running => "Running",
                AgentUiActivityStatus::AwaitingConfirmation => "Needs approval",
                AgentUiActivityStatus::Success => "Done",
                AgentUiActivityStatus::Failed => "Failed",
                AgentUiActivityStatus::Cancelled => "Cancelled",
            };
            let duration = activity
                .duration_ms
                .map(|duration| format!(" · {duration} ms"))
                .unwrap_or_default();
            ui.label(
                RichText::new(format!("{status} · {}{duration}", activity.label))
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
                        let rows = if is_sampled {
                            format!("Showing {sample_len} sampled rows of {total_rows}")
                        } else {
                            format!("{total_rows} rows")
                        };
                        let columns = format!("{} cols", summary.columns.len());
                        let count = if *result_count > 1 {
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
                                RichText::new(format!("↳ {rows}, {columns}{count}"))
                                    .small()
                                    .color(self.theme.text_muted),
                            );
                            let label = if is_sampled {
                                "Open sample in Results"
                            } else {
                                "Open in Results"
                            };
                            if Button::new(self.theme)
                                .icon(Icon::Table2)
                                .text(label)
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

    fn draw_confirmation(&self, ui: &mut egui::Ui, pending: &AgentUiConfirmation) -> Option<AgentThreadAction> {
        ui.add_space(SPACE_SM);
        let mut action = None;
        editor_frame(self.theme).show(ui, |ui| {
            ui.label(
                RichText::new(confirmation_title(pending.kind))
                    .strong()
                    .color(self.theme.text_primary),
            );
            if pending.document_id != self.document_id {
                ui.label(
                    RichText::new(format!("Target query: {}", pending.document_id))
                        .small()
                        .color(self.theme.accent),
                );
            }
            self.draw_patch_preview(ui, pending.preview.as_ref());
            if pending.preview.is_none() {
                self.draw_confirmation_notice(ui, pending.kind);
            }
            action = self.draw_confirmation_actions(ui, pending.kind);
        });
        action
    }

    fn draw_patch_preview(&self, ui: &mut egui::Ui, preview: Option<&db_pro_core::domain::agent::AgentToolOutput>) {
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

    fn draw_confirmation_notice(
        &self,
        ui: &mut egui::Ui,
        kind: db_pro_core::domain::agent_workflow::AgentConfirmationKind,
    ) {
        let (message, color) = match kind {
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunMutation => (
                "Warning: This mutation will modify database data or schema.",
                self.theme.warning,
            ),
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunDestructive => (
                "Caution: Destructive query may irreversibly drop or truncate data.",
                self.theme.danger,
            ),
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunUnknown => (
                "Classification unknown: Review the query carefully before running.",
                self.theme.text_secondary,
            ),
            _ => (
                "Review the requested action before continuing.",
                self.theme.text_secondary,
            ),
        };
        ui.label(RichText::new(message).small().color(color));
    }

    fn draw_confirmation_actions(
        &self,
        ui: &mut egui::Ui,
        kind: db_pro_core::domain::agent_workflow::AgentConfirmationKind,
    ) -> Option<AgentThreadAction> {
        let approve_label = match kind {
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::ApplyPatch => "Apply Change",
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunReadOnly => "Run Query",
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunMutation => "Run Mutation",
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunDestructive => "Execute Destructive Query",
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunUnknown => "Run Unclassified Query",
        };
        let approve_variant = if matches!(
            kind,
            db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunDestructive
        ) {
            ButtonVariant::Destructive
        } else {
            ButtonVariant::Default
        };
        let mut action = None;
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .icon(Icon::Check)
                .text(approve_label)
                .variant(approve_variant)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                action = Some(AgentThreadAction::Confirm(true));
            }
            if Button::new(self.theme)
                .icon(Icon::X)
                .text("Reject")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                action = Some(AgentThreadAction::Confirm(false));
            }
        });
        action
    }

    fn draw_retry(&self, ui: &mut egui::Ui) -> bool {
        ui.add_space(SPACE_SM);
        ui.horizontal(|ui| {
            Button::new(self.theme)
                .icon(Icon::RotateCcw)
                .text("Retry")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
        })
        .inner
    }

    fn draw_thinking(&self, ui: &mut egui::Ui) {
        ui.add_space(SPACE_SM);
        AgentThinking::new("Working in this query…", &mut true, self.theme)
            .is_active(true)
            .show(ui);
    }
}

pub(super) fn confirmation_title(kind: db_pro_core::domain::agent_workflow::AgentConfirmationKind) -> &'static str {
    match kind {
        db_pro_core::domain::agent_workflow::AgentConfirmationKind::ApplyPatch => "Apply Agent change?",
        db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunReadOnly => "Run read-only query?",
        db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunMutation => "Run mutation?",
        db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunDestructive => "Execute destructive query?",
        db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunUnknown => "Run unclassified query?",
    }
}
