//! Query status bar surface and typed intents.

use super::*;

pub(super) const QUERY_STATUS_HEIGHT: f32 = 24.0;

#[derive(Debug)]
pub(super) enum QueryStatusBarAction {
    ShowOutput,
    ToggleTransaction,
    ToggleParameters,
    ShowDiagnostics,
}

pub(super) struct QueryStatusBarContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) bottom_panel_open: bool,
    pub(super) in_transaction: bool,
    pub(super) transaction_pending: usize,
    pub(super) auto_commit: bool,
    pub(super) cursor_line: usize,
    pub(super) cursor_column: usize,
    pub(super) driver: &'a str,
    pub(super) schema: &'a str,
    pub(super) parameter_count: usize,
    pub(super) diagnostic_count: usize,
}

pub(super) fn draw_status_bar(context: &QueryStatusBarContext<'_>, ui: &mut egui::Ui) -> Option<QueryStatusBarAction> {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), QUERY_STATUS_HEIGHT),
        egui::Sense::hover(),
    );
    ui.painter().hline(
        rect.x_range(),
        rect.top(),
        egui::Stroke::new(1.0, context.theme.border_subtle),
    );
    let mut action = None;
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.set_min_height(QUERY_STATUS_HEIGHT);
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);
            ui.add_space(SPACE_XS);
            action = draw_right_controls(context, ui);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                if let Some(metadata_action) = draw_metadata(context, ui) {
                    action = Some(metadata_action);
                }
            });
        });
    });
    action
}

fn draw_right_controls(context: &QueryStatusBarContext<'_>, ui: &mut egui::Ui) -> Option<QueryStatusBarAction> {
    let mut action = None;
    if !context.bottom_panel_open
        && Button::new(context.theme)
            .icon(Icon::PanelBottom)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .tooltip("Show output")
            .show(ui)
            .clicked()
    {
        action = Some(QueryStatusBarAction::ShowOutput);
    }
    action
}

fn draw_metadata(context: &QueryStatusBarContext<'_>, ui: &mut egui::Ui) -> Option<QueryStatusBarAction> {
    ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);
    ui.add_space(SPACE_XS);
    ui.label(
        RichText::new(format!("Ln {}, Col {}", context.cursor_line, context.cursor_column))
            .font(font_mono_sm())
            .color(context.theme.text_muted),
    );
    ui.label(
        RichText::new(context.driver)
            .font(font_caption())
            .color(context.theme.text_muted),
    );
    ui.label(
        RichText::new(context.schema)
            .font(font_caption())
            .color(context.theme.text_muted),
    );
    let mut action = draw_transaction_label(context, ui);
    if context.parameter_count > 0 {
        action = draw_parameter_label(context, ui).or(action);
    }
    if context.diagnostic_count > 0 {
        action = draw_diagnostic_label(context, ui).or(action);
    }
    action
}

fn draw_transaction_label(context: &QueryStatusBarContext<'_>, ui: &mut egui::Ui) -> Option<QueryStatusBarAction> {
    let label = if context.in_transaction {
        format!("Transaction · {} pending", context.transaction_pending)
    } else if context.auto_commit {
        "Auto-commit".to_owned()
    } else {
        "Manual".to_owned()
    };
    let response = ui.add(
        egui::Label::new(
            RichText::new(label)
                .font(font_caption())
                .color(if context.in_transaction {
                    context.theme.warning
                } else {
                    context.theme.text_muted
                }),
        )
        .sense(egui::Sense::click()),
    );
    let clicked = response.clicked();
    response.on_hover_text("Toggle transaction controls");
    clicked.then_some(QueryStatusBarAction::ToggleTransaction)
}

fn draw_parameter_label(context: &QueryStatusBarContext<'_>, ui: &mut egui::Ui) -> Option<QueryStatusBarAction> {
    let label = if context.parameter_count == 1 {
        "1 parameter".to_owned()
    } else {
        format!("{} parameters", context.parameter_count)
    };
    let response = ui.add(
        egui::Label::new(RichText::new(label).font(font_caption()).color(context.theme.accent))
            .sense(egui::Sense::click()),
    );
    let clicked = response.clicked();
    response.on_hover_text("Edit bind parameters");
    clicked.then_some(QueryStatusBarAction::ToggleParameters)
}

fn draw_diagnostic_label(context: &QueryStatusBarContext<'_>, ui: &mut egui::Ui) -> Option<QueryStatusBarAction> {
    let response = ui.add(
        egui::Label::new(
            RichText::new(format!("{} diagnostics", context.diagnostic_count))
                .font(font_caption())
                .color(context.theme.warning),
        )
        .sense(egui::Sense::click()),
    );
    response.clicked().then_some(QueryStatusBarAction::ShowDiagnostics)
}
