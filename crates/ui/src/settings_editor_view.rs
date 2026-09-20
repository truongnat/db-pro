//! Editor settings presentation over preferences and query-editor state.

use super::*;
use crate::editor::PredictionMode;

pub(crate) struct SettingsEditorContext<'a> {
    pub(crate) theme: DbProTheme,
    pub(crate) preferences: &'a mut PreferencesState,
    pub(crate) query: &'a mut QueryFeatureState,
}

impl SettingsEditorContext<'_> {
    pub(crate) fn draw(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "EDITOR", self.theme);
            ui.add_space(10.0);
            self.draw_editor_core(ui);
            self.draw_prediction(ui);
            self.draw_lint(ui);
        });
    }

    fn draw_editor_core(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Font size").color(self.theme.text_secondary));
            if compact_button(ui, "−", self.theme).clicked() {
                self.query.editor.editor_font_size = (self.query.editor.editor_font_size - 1.0).max(10.0);
                self.preferences.settings.editor.font_size = self.query.editor.editor_font_size;
            }
            ui.label(
                RichText::new(format!("{:.0} px", self.query.editor.editor_font_size)).color(self.theme.text_primary),
            );
            if compact_button(ui, "+", self.theme).clicked() {
                self.query.editor.editor_font_size = (self.query.editor.editor_font_size + 1.0).min(24.0);
                self.preferences.settings.editor.font_size = self.query.editor.editor_font_size;
            }
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Tab width").color(self.theme.text_secondary));
            ui.add(egui::DragValue::new(&mut self.preferences.settings.editor.tab_width).range(2..=8));
        });
        ui.checkbox(
            &mut self.preferences.settings.editor.completion_enabled,
            "Schema completion",
        );
        ui.checkbox(
            &mut self.preferences.settings.editor.format_on_save,
            "Format SQL on save",
        );
    }

    fn draw_prediction(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.label(RichText::new("AI prediction").color(self.theme.text_secondary));
        ui.horizontal_wrapped(|ui| {
            for (mode, label) in [
                (PredictionMode::Off, "Off"),
                (PredictionMode::Subtle, "Subtle"),
                (PredictionMode::Eager, "Eager"),
            ] {
                if ui
                    .selectable_label(self.preferences.prediction_mode == mode, label)
                    .clicked()
                {
                    self.preferences.prediction_mode = mode;
                    self.preferences.settings.editor.prediction_mode = label.to_ascii_lowercase();
                }
            }
        });
        ui.label(
            RichText::new("Sends the SQL around your cursor and its schema context to your configured AI provider.")
                .small()
                .color(self.theme.text_muted),
        );
    }

    fn draw_lint(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);
        section_label(ui, "SQL LINT", self.theme);
        ui.add_space(6.0);
        ui.checkbox(
            &mut self.preferences.settings.editor.lint.enabled,
            "Enable SQL lint warnings",
        );
        ui.add_enabled_ui(self.preferences.settings.editor.lint.enabled, |ui| {
            ui.checkbox(
                &mut self.preferences.settings.editor.lint.select_star,
                "Warn on SELECT *",
            );
            ui.checkbox(
                &mut self.preferences.settings.editor.lint.null_compare,
                "Warn on = NULL / != NULL",
            );
            ui.checkbox(
                &mut self.preferences.settings.editor.lint.delete_no_where,
                "Warn on DELETE without WHERE",
            );
            ui.checkbox(
                &mut self.preferences.settings.editor.lint.update_no_where,
                "Warn on UPDATE without WHERE",
            );
            ui.checkbox(
                &mut self.preferences.settings.editor.lint.order_by_ordinal,
                "Warn on ORDER BY ordinal",
            );
            ui.checkbox(
                &mut self.preferences.settings.editor.lint.comma_join,
                "Warn on comma / cartesian joins",
            );
            ui.checkbox(
                &mut self.preferences.settings.editor.lint.duplicate_alias,
                "Warn on duplicate projection aliases",
            );
        });
        ui.label(
            RichText::new("Lint warnings stay local and never call AI or the network.")
                .small()
                .color(self.theme.text_muted),
        );
    }
}
