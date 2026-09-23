use super::*;
use lucide_icons::Icon;

pub(super) struct ShellOutputPanelContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) workspace: &'a mut WorkspaceFeatureState,
    pub(super) output: &'a mut QueryOutputState,
    pub(super) session: &'a QuerySessionState,
    pub(super) editor: &'a QueryEditorState,
    pub(super) active_result: Option<&'a UiQueryResult>,
}

impl ShellOutputPanelContext<'_> {
    pub(super) fn draw(&mut self, ctx: &egui::Context) {
        if !self.workspace.bottom_panel_open {
            return;
        }
        let height = self.workspace.bottom_panel_height;
        let response = TopBottomPanel::bottom("output_panel")
            .resizable(true)
            .default_height(height)
            .height_range(OUTPUT_MIN_HEIGHT..=OUTPUT_MAX_HEIGHT)
            .frame(panel_frame(self.theme))
            .show(ctx, |ui| self.draw_panel_contents(ui));
        self.workspace.set_bottom_panel_height(response.response.rect.height());
    }

    fn draw_panel_contents(&mut self, ui: &mut egui::Ui) {
        ui.set_min_size(ui.available_size());
        ui.horizontal(|ui| {
            section_label(ui, "OUTPUT", self.theme);
            self.draw_tabs(ui);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if compact_icon_button(ui, Icon::X, self.theme)
                    .on_hover_text("Close output")
                    .clicked()
                {
                    self.workspace.bottom_panel_open = false;
                }
            });
        });
        ui.separator();
        self.draw_active_tab(ui);
    }

    fn draw_tabs(&mut self, ui: &mut egui::Ui) {
        for (tab, label) in [
            (OutputTab::Results, "Results"),
            (OutputTab::Chart, "Chart"),
            (OutputTab::Messages, "Messages"),
            (OutputTab::Explain, "Explain"),
            (OutputTab::History, "History"),
        ] {
            if tab_frame(self.theme, self.output.active_tab == tab)
                .show(ui, |ui| ui.selectable_label(self.output.active_tab == tab, label))
                .inner
                .clicked()
            {
                self.output.active_tab = tab;
            }
        }
    }

    fn draw_active_tab(&self, ui: &mut egui::Ui) {
        match self.output.active_tab {
            OutputTab::Results => {
                let summary = self
                    .active_result
                    .map(|result| format!("{} rows · {} ms", result.row_count, result.duration_ms))
                    .unwrap_or_else(|| "No result".to_owned());
                ui.label(RichText::new(summary).small().color(self.theme.text_secondary));
            }
            OutputTab::Chart => {
                ui.label(
                    RichText::new("Chart view — open the Chart tab for full controls")
                        .small()
                        .color(self.theme.text_muted),
                );
            }
            OutputTab::Messages => {
                for message in self.session.active_messages().iter().rev().take(8) {
                    ui.label(RichText::new(message).small().color(self.theme.text_secondary));
                }
            }
            OutputTab::Explain => self.draw_explain_summary(ui),
            OutputTab::History => {
                for query in self.editor.query_history.iter().rev().take(8) {
                    ui.label(
                        RichText::new(query)
                            .monospace()
                            .small()
                            .color(self.theme.text_secondary),
                    );
                }
            }
        };
    }

    fn draw_explain_summary(&self, ui: &mut egui::Ui) {
        if let Some(plan) = self.session.active_explain_plan() {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(RichText::new(plan).monospace().small().color(self.theme.text_secondary));
            });
        } else {
            ui.label(
                RichText::new("Run Explain to inspect the query plan")
                    .small()
                    .color(self.theme.text_muted),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_panel_uses_explicit_output_tabs() {
        assert_ne!(OutputTab::Results, OutputTab::Messages);
    }
}
