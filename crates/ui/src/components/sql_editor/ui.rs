use super::config::{BUTTON_SPACING, TOOLBAR_BUTTON_SIZE, TOOLBAR_MARGIN, TOOLBAR_ROUNDING, TOOLBAR_STROKE_WIDTH};
use super::handler::{
    action_access_label, action_icon, action_label, action_variant, should_show_cancel, should_show_idle_actions,
    should_show_run_selection, SqlEditorAction,
};
use crate::components::button::Button;
use crate::DbProTheme;
use egui::{Stroke, Ui};

pub struct SqlEditorToolbar<'a> {
    is_running: bool,
    has_selection: bool,
    theme: DbProTheme,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> SqlEditorToolbar<'a> {
    pub fn new(is_running: bool, has_selection: bool, theme: DbProTheme) -> Self {
        Self {
            is_running,
            has_selection,
            theme,
            _marker: std::marker::PhantomData,
        }
    }

    /// Renders an action button with the unified button component and metadata.
    fn render_action_button(&self, ui: &mut Ui, action: SqlEditorAction) -> bool {
        let mut btn = Button::new(self.theme)
            .text(action_label(action))
            .access_label(action_access_label(action))
            .variant(action_variant(action))
            .size(TOOLBAR_BUTTON_SIZE);

        if let Some(icon) = action_icon(action) {
            btn = btn.icon(icon);
        }

        btn.show(ui).clicked()
    }

    /// Renders the toolbar surface and returns any triggered action.
    pub fn show(self, ui: &mut Ui) -> Option<SqlEditorAction> {
        let mut triggered = None;

        let frame = egui::Frame::none()
            .fill(self.theme.surface_panel)
            .stroke(Stroke::new(TOOLBAR_STROKE_WIDTH, self.theme.border_subtle))
            .rounding(TOOLBAR_ROUNDING)
            .inner_margin(TOOLBAR_MARGIN);

        frame.show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if should_show_cancel(self.is_running) {
                    if self.render_action_button(ui, SqlEditorAction::CancelQuery) {
                        triggered = Some(SqlEditorAction::CancelQuery);
                    }
                } else if should_show_idle_actions(self.is_running) {
                    if self.render_action_button(ui, SqlEditorAction::RunQuery) {
                        triggered = Some(SqlEditorAction::RunQuery);
                    }

                    if should_show_run_selection(self.is_running, self.has_selection) {
                        ui.add_space(BUTTON_SPACING);
                        if self.render_action_button(ui, SqlEditorAction::RunSelection) {
                            triggered = Some(SqlEditorAction::RunSelection);
                        }
                    }

                    ui.add_space(BUTTON_SPACING);
                    if self.render_action_button(ui, SqlEditorAction::ExplainQuery) {
                        triggered = Some(SqlEditorAction::ExplainQuery);
                    }

                    ui.add_space(BUTTON_SPACING);
                    if self.render_action_button(ui, SqlEditorAction::FormatSql) {
                        triggered = Some(SqlEditorAction::FormatSql);
                    }
                }

                // AI Action trigger on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.render_action_button(ui, SqlEditorAction::AskAi) {
                        triggered = Some(SqlEditorAction::AskAi);
                    }
                });
            });
        });

        triggered
    }
}
