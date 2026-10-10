//! Query actions menu presentation and typed intents.
use super::super::*;
use super::AI_PREDICTION_EGRESS_NOTE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum QueryActionsSurfaceAction {
    Run,
    Format,
    Explain,
    AskAgent,
    Save,
    SaveAs,
    ToggleVisualBuilder,
    ToggleSearch,
    ToggleTransaction,
    DecreaseFont,
    IncreaseFont,
    GeneratePrediction,
    SetPredictionMode(PredictionMode),
    ToggleSnippets,
    CreateFolder,
    Close,
}

pub(super) struct QueryActionsSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) editor: &'a mut QueryEditorState,
    pub(super) execution: &'a mut QueryExecutionPolicyState,
    pub(super) library: &'a mut QueryLibraryState,
    pub(super) session: &'a QuerySessionState,
    pub(super) prediction_mode: &'a mut PredictionMode,
}

impl QueryActionsSurfaceContext<'_> {
    pub(super) fn draw(&mut self, ctx: &egui::Context, anchor: egui::Rect) -> Vec<QueryActionsSurfaceAction> {
        let menu_width = 264.0;
        let menu_position = egui::pos2((anchor.right() - menu_width).max(8.0), anchor.bottom() + 4.0);
        let mut actions = Vec::new();
        let menu = egui::Area::new(egui::Id::new("query_actions_menu"))
            .order(egui::Order::Foreground)
            .fixed_pos(menu_position)
            .constrain_to(ctx.content_rect().shrink(4.0))
            .show(ctx, |ui| {
                egui::Frame {
                    fill: self.theme.surface_elevated,
                    inner_margin: egui::Margin::same(8.0 as i8),
                    corner_radius: egui::CornerRadius::same(8.0 as u8),
                    stroke: egui::Stroke::new(1.0, self.theme.border_subtle),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    ui.set_min_width(menu_width);
                    ui.label(
                        RichText::new("Query actions")
                            .small()
                            .strong()
                            .color(self.theme.text_muted),
                    );
                    ui.add_space(4.0);
                    self.draw_run_actions(ui, &mut actions);
                    ui.separator();
                    ui.label(RichText::new("Editor").small().strong().color(self.theme.text_muted));
                    ui.add_space(4.0);
                    self.draw_editor_actions(ui, &mut actions);
                    ui.label(
                        RichText::new(format!("Editor font · {} px", self.editor.editor_font_size))
                            .small()
                            .color(self.theme.text_muted),
                    );
                });
            });
        let clicked_outside = ctx.input(|input| {
            input.pointer.any_click()
                && input
                    .pointer
                    .interact_pos()
                    .is_some_and(|position| !menu.response.rect.contains(position) && !anchor.contains(position))
        });
        if clicked_outside {
            actions.push(QueryActionsSurfaceAction::Close);
        }
        actions
    }

    #[cfg(test)]
    pub(super) fn draw_editor_only(&mut self, ui: &mut egui::Ui) -> Vec<QueryActionsSurfaceAction> {
        let mut actions = Vec::new();
        self.draw_editor_actions(ui, &mut actions);
        actions
    }

    fn draw_run_actions(&self, ui: &mut egui::Ui, actions: &mut Vec<QueryActionsSurfaceAction>) {
        let run_label = if self.session.selected_text.is_empty() {
            t!("query.run")
        } else {
            t!("query.run")
        };
        if action_menu_button(ui, Icon::Play, &run_label, self.theme).clicked() {
            actions.push(QueryActionsSurfaceAction::Run);
            actions.push(QueryActionsSurfaceAction::Close);
        }
        if action_menu_button(ui, Icon::WandSparkles, &t!("query.format_sql"), self.theme).clicked() {
            actions.push(QueryActionsSurfaceAction::Format);
            actions.push(QueryActionsSurfaceAction::Close);
        }
        if action_menu_button(ui, Icon::ChartNoAxesCombined, &t!("query.explain"), self.theme).clicked() {
            actions.push(QueryActionsSurfaceAction::Explain);
            actions.push(QueryActionsSurfaceAction::Close);
        }
        if action_menu_button(ui, Icon::Bot, &t!("query.ask_agent"), self.theme).clicked() {
            actions.push(QueryActionsSurfaceAction::AskAgent);
            actions.push(QueryActionsSurfaceAction::Close);
        }
        if action_menu_button(ui, Icon::Save, &t!("query.save_query"), self.theme).clicked() {
            actions.push(QueryActionsSurfaceAction::Save);
            actions.push(QueryActionsSurfaceAction::Close);
        }
        if action_menu_button(ui, Icon::Save, &t!("query.save_query_as"), self.theme).clicked() {
            actions.push(QueryActionsSurfaceAction::SaveAs);
            actions.push(QueryActionsSurfaceAction::Close);
        }
        let builder_label = if self.editor.visual_builder.open {
            t!("query.visual_builder")
        } else {
            t!("query.visual_builder")
        };
        if action_menu_button(ui, Icon::LayoutTemplate, &builder_label, self.theme).clicked() {
            actions.push(QueryActionsSurfaceAction::ToggleVisualBuilder);
            actions.push(QueryActionsSurfaceAction::Close);
        }
    }

    fn draw_editor_actions(&mut self, ui: &mut egui::Ui, actions: &mut Vec<QueryActionsSurfaceAction>) {
        self.draw_editor_controls(ui, actions);
        self.draw_prediction_controls(ui, actions);
        self.draw_folder_controls(ui, actions);
    }

    fn draw_editor_controls(&self, ui: &mut egui::Ui, actions: &mut Vec<QueryActionsSurfaceAction>) {
        if action_menu_button(ui, Icon::Search, &t!("query.find_in_sql"), self.theme).clicked() {
            actions.push(QueryActionsSurfaceAction::ToggleSearch);
            actions.push(QueryActionsSurfaceAction::Close);
        }
        let transaction_label = if self.execution.query_txn_bar_open {
            "Hide transaction controls"
        } else {
            "Show transaction controls"
        };
        if action_menu_button(ui, Icon::GitBranch, transaction_label, self.theme).clicked() {
            actions.push(QueryActionsSurfaceAction::ToggleTransaction);
            actions.push(QueryActionsSurfaceAction::Close);
        }
        if action_menu_button(ui, Icon::Minus, "Decrease font size", self.theme).clicked() {
            actions.push(QueryActionsSurfaceAction::DecreaseFont);
        }
        if action_menu_button(ui, Icon::Plus, "Increase font size", self.theme).clicked() {
            actions.push(QueryActionsSurfaceAction::IncreaseFont);
        }
    }

    fn draw_prediction_controls(&self, ui: &mut egui::Ui, actions: &mut Vec<QueryActionsSurfaceAction>) {
        if action_menu_button(ui, Icon::Bot, "Generate SQL Prediction", self.theme).clicked() {
            actions.push(QueryActionsSurfaceAction::GeneratePrediction);
            actions.push(QueryActionsSurfaceAction::Close);
        }
        ui.horizontal(|ui| {
            ui.label(RichText::new("AI prediction").small().color(self.theme.text_muted));
            for (mode, label) in [
                (PredictionMode::Off, "Off"),
                (PredictionMode::Subtle, "Subtle"),
                (PredictionMode::Eager, "Eager"),
            ] {
                if Button::new(self.theme)
                    .text(label)
                    .variant(if *self.prediction_mode == mode {
                        ButtonVariant::Secondary
                    } else {
                        ButtonVariant::Ghost
                    })
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    actions.push(QueryActionsSurfaceAction::SetPredictionMode(mode));
                }
            }
        });
        ui.label(
            RichText::new(AI_PREDICTION_EGRESS_NOTE)
                .font(font_caption())
                .color(self.theme.text_muted),
        );
        ui.add_space(4.0);
    }

    fn draw_folder_controls(&mut self, ui: &mut egui::Ui, actions: &mut Vec<QueryActionsSurfaceAction>) {
        if action_menu_button(ui, Icon::FileCode2, &t!("query.sql_snippets"), self.theme).clicked() {
            actions.push(QueryActionsSurfaceAction::ToggleSnippets);
            actions.push(QueryActionsSurfaceAction::Close);
        }
        ui.horizontal(|ui| {
            Input::new(
                &mut self.library.query_folder,
                "folder (optional)",
                self.theme,
            )
            .width(150.0)
            .show(ui);
            if Button::new(self.theme)
                .text(t!("query.new_folder"))
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(QueryActionsSurfaceAction::CreateFolder);
            }
        });
    }
}

fn action_menu_button(ui: &mut egui::Ui, icon: Icon, label: &str, theme: DbProTheme) -> egui::Response {
    Button::new(theme)
        .icon(icon)
        .text(label)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .full_width(true)
        .left_aligned()
        .show(ui)
}
