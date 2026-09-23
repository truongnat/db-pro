//! Agent panel shell and context presentation.
use super::agent_context_actions_view::{AgentContextAction, AgentContextActionsContext};
use super::agent_header_view::{AgentHeaderAction, AgentHeaderContext};
use super::agent_settings_view::{AgentSettingsAction, AgentSettingsContext};
use super::agent_thread_surface_view::{AgentThreadAction, AgentThreadSurfaceContext};
use super::agent_workflow_state::AgentUiSession;
use super::*;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum AgentPanelAction {
    Header(AgentHeaderAction),
    Settings(AgentSettingsAction),
    Context(AgentContextAction),
    Thread(AgentThreadAction),
    Composer(AgentComposerAction),
}

pub(super) struct AgentPanelSurfaceContext {
    pub(super) theme: DbProTheme,
    pub(super) default_width: f32,
}

impl AgentPanelSurfaceContext {
    pub(super) fn show<F>(self, ctx: &egui::Context, content: F) -> f32
    where
        F: FnOnce(&mut egui::Ui),
    {
        let response = egui::SidePanel::right("agent_panel")
            .resizable(true)
            .default_width(self.default_width)
            .width_range(AGENT_MIN_WIDTH..=AGENT_MAX_WIDTH)
            .frame(sidebar_frame(self.theme))
            .show(ctx, |ui| {
                ui.set_min_size(ui.available_size());
                content(ui);
            });
        response.response.rect.width()
    }
}

pub(super) struct AgentPanelContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) default_width: f32,
    pub(super) document_id: Option<&'a str>,
    pub(super) settings_open: bool,
    pub(super) provider_label: &'a str,
    pub(super) provider_detail: &'a str,
    pub(super) api_key_draft: &'a mut String,
    pub(super) api_key_show_password: &'a mut bool,
    pub(super) configure_request: Option<RequestId>,
    pub(super) auto_run_read_only: &'a mut bool,
    pub(super) context: &'a AgentContext,
    pub(super) session: Option<&'a mut AgentUiSession>,
    pub(super) input: &'a mut String,
    pub(super) composer_mode: AgentMode,
    pub(super) is_generating: bool,
}

struct AgentPanelContent<'a> {
    theme: DbProTheme,
    document_id: Option<&'a str>,
    settings_open: bool,
    provider_label: &'a str,
    provider_detail: &'a str,
    api_key_draft: &'a mut String,
    api_key_show_password: &'a mut bool,
    configure_request: Option<RequestId>,
    auto_run_read_only: &'a mut bool,
    context: &'a AgentContext,
    session: Option<&'a mut AgentUiSession>,
    input: &'a mut String,
    composer_mode: AgentMode,
    is_generating: bool,
}

impl AgentPanelContext<'_> {
    pub(super) fn draw(self, ctx: &egui::Context) -> (f32, Vec<AgentPanelAction>) {
        let AgentPanelContext {
            theme,
            default_width,
            document_id,
            settings_open,
            provider_label,
            provider_detail,
            api_key_draft,
            api_key_show_password,
            configure_request,
            auto_run_read_only,
            context,
            session,
            input,
            composer_mode,
            is_generating,
        } = self;
        let mut actions = Vec::new();
        let mut content = AgentPanelContent {
            theme,
            document_id,
            settings_open,
            provider_label,
            provider_detail,
            api_key_draft,
            api_key_show_password,
            configure_request,
            auto_run_read_only,
            context,
            session,
            input,
            composer_mode,
            is_generating,
        };
        let panel_width = AgentPanelSurfaceContext { theme, default_width }.show(ctx, |ui| {
            content.draw(ui, &mut actions);
        });
        (panel_width, actions)
    }
}

impl AgentPanelContent<'_> {
    fn draw(&mut self, ui: &mut egui::Ui, actions: &mut Vec<AgentPanelAction>) {
        self.draw_header(ui, actions);
        if self.draw_settings(ui, actions) {
            return;
        }
        self.draw_context(ui, actions);
        self.draw_thread(ui, actions);
        self.draw_composer(ui, actions);
    }

    fn draw_header(&mut self, ui: &mut egui::Ui, actions: &mut Vec<AgentPanelAction>) {
        let Some(session) = self.session.as_deref_mut() else {
            return;
        };
        let mode_disabled =
            session.active_run_id.is_some() || session.request_id.is_some() || session.pending_confirmation.is_some();
        let mut header = AgentHeaderContext {
            theme: self.theme,
            mode: Some(&mut session.mode),
            mode_disabled,
            can_clear_conversation: !mode_disabled,
            has_messages: !session.messages.is_empty(),
        };
        actions.extend(header.draw(ui).into_iter().map(AgentPanelAction::Header));
    }

    fn draw_settings(&mut self, ui: &mut egui::Ui, actions: &mut Vec<AgentPanelAction>) -> bool {
        if !self.settings_open {
            return false;
        }
        let mut settings = AgentSettingsContext {
            theme: self.theme,
            provider_label: self.provider_label,
            api_key_draft: self.api_key_draft,
            api_key_show_password: self.api_key_show_password,
            configure_request: self.configure_request,
            auto_run_read_only: self.auto_run_read_only,
        };
        actions.extend(settings.draw(ui).into_iter().map(AgentPanelAction::Settings));
        true
    }

    fn draw_context(&self, ui: &mut egui::Ui, actions: &mut Vec<AgentPanelAction>) {
        ui.add_space(6.0);
        AgentContextSurfaceContext {
            theme: self.theme,
            provider_label: self.provider_label,
            provider_detail: self.provider_detail,
            auto_run_read_only: *self.auto_run_read_only,
            context: self.context,
        }
        .draw(ui);
        ui.add_space(8.0);
        actions.extend(
            AgentContextActionsContext {
                theme: self.theme,
                context: self.context,
            }
            .draw(ui)
            .into_iter()
            .map(AgentPanelAction::Context),
        );
    }

    fn draw_thread(&self, ui: &mut egui::Ui, actions: &mut Vec<AgentPanelAction>) {
        let (Some(document_id), Some(session)) = (self.document_id, self.session.as_deref()) else {
            return;
        };
        actions.extend(
            AgentThreadSurfaceContext {
                theme: self.theme,
                document_id,
                session,
            }
            .draw(ui)
            .into_iter()
            .map(AgentPanelAction::Thread),
        );
    }

    fn draw_composer(&mut self, ui: &mut egui::Ui, actions: &mut Vec<AgentPanelAction>) {
        if let Some(action) = AgentComposer::new(self.input, self.provider_label, self.composer_mode, self.theme)
            .is_generating(self.is_generating)
            .show(ui)
        {
            actions.push(AgentPanelAction::Composer(action));
        }
    }
}

pub(super) struct AgentContextSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) provider_label: &'a str,
    pub(super) provider_detail: &'a str,
    pub(super) auto_run_read_only: bool,
    pub(super) context: &'a AgentContext,
}

impl AgentContextSurfaceContext<'_> {
    pub(super) fn draw(&self, ui: &mut egui::Ui) {
        toolbar_frame(self.theme).show(ui, |ui| {
            egui::ScrollArea::horizontal()
                .id_salt("agent-context-chips")
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if self.provider_label == "Offline draft" {
                            badge(ui, "Preview", self.theme.surface_active, self.theme.text_secondary);
                        }
                        badge(ui, self.provider_label, self.theme.accent_soft, self.theme.accent);
                        if self.auto_run_read_only {
                            badge(ui, "Auto-run Read-only", self.theme.accent_soft, self.theme.accent);
                        }
                        ContextChip::new(
                            ContextChipKind::Connection,
                            self.context.connection_name.as_deref().unwrap_or("No connection"),
                            self.theme,
                        )
                        .show(ui);
                        ContextChip::new(ContextChipKind::Database, &self.context.driver, self.theme).show(ui);
                        if let Some(schema) = self.context.schema.as_deref() {
                            ContextChip::new(ContextChipKind::Schema, schema, self.theme).show(ui);
                        }
                        if let Some(table) = self.context.selected_table.as_deref() {
                            ContextChip::new(ContextChipKind::Table, table, self.theme).show(ui);
                        }
                        if self.context.explain_plan.is_some() {
                            ContextChip::new(ContextChipKind::Editor, "EXPLAIN PLAN", self.theme).show(ui);
                        }
                    });
                });
            ui.label(
                RichText::new(self.provider_detail)
                    .font(font_caption())
                    .color(self.theme.text_muted),
            );
        });
    }
}
