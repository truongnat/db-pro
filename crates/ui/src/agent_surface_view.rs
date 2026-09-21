//! Agent panel shell and context presentation.
use super::*;

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
