//! A modern ChatGPT-inspired introduction with interactive quick-action cards.
use super::*;
use egui::RichText;
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WelcomeAction {
    NewConnection,
    NewQuery,
    ToggleAgent,
    OpenDiagram,
}

pub(super) fn draw(theme: DbProTheme, ui: &mut egui::Ui) -> Option<WelcomeAction> {
    let top_space = ((ui.available_height() - 400.0) * 0.4).max(SPACE_MD);
    ui.add_space(top_space);

    let mut action = None;
    ui.vertical_centered(|ui| {
        draw_header(theme, ui);
        ui.add_space(SPACE_2XL);
        action = draw_quick_actions(theme, ui);
    });

    action
}

fn draw_header(theme: DbProTheme, ui: &mut egui::Ui) {
    ui.label(
        RichText::new(char::from(Icon::Database).to_string())
            .font(font_icon(ICON_XL))
            .color(theme.accent),
    );
    ui.add_space(SPACE_MD);
    ui.label(RichText::new("DB Pro").font(font_display()).color(theme.text_primary));
    ui.add_space(SPACE_XS);
    ui.label(
        RichText::new("Explore databases, write SQL with AI Copilot, and inspect results.")
            .font(font_body())
            .color(theme.text_secondary),
    );
    ui.add_space(SPACE_XS);
    ui.label(
        RichText::new("PostgreSQL  ·  SQLite  ·  MySQL  ·  SQL Server")
            .font(font_caption())
            .color(theme.text_muted),
    );
}

fn draw_quick_actions(theme: DbProTheme, ui: &mut egui::Ui) -> Option<WelcomeAction> {
    let cards_width = 520.0_f32.min(ui.available_width() - 32.0);
    ui.set_max_width(cards_width);

    let mut triggered = None;
    for (icon, title, desc, action) in [
        (
            Icon::Plus,
            "New Connection",
            "Connect to PostgreSQL, SQLite or remote database",
            WelcomeAction::NewConnection,
        ),
        (
            Icon::FileCode2,
            "New SQL Query",
            "Open a blank scratchpad with auto-completion",
            WelcomeAction::NewQuery,
        ),
        (
            Icon::Sparkles,
            "AI Copilot Assistant",
            "Generate queries, explain schema or review SQL",
            WelcomeAction::ToggleAgent,
        ),
        (
            Icon::Workflow,
            "ER Diagram Canvas",
            "Visualize schema tables and relationships",
            WelcomeAction::OpenDiagram,
        ),
    ] {
        if draw_card_item(theme, ui, cards_width, icon, title, desc) {
            triggered = Some(action);
        }
        ui.add_space(SPACE_XS);
    }
    triggered
}

fn draw_card_item(
    theme: DbProTheme,
    ui: &mut egui::Ui,
    cards_width: f32,
    icon: Icon,
    title: &str,
    desc: &str,
) -> bool {
    let card_frame = egui::Frame {
        fill: theme.surface_elevated,
        inner_margin: egui::Margin::symmetric(SPACE_MD as i8, SPACE_SM as i8),
        stroke: egui::Stroke::new(STROKE_THIN, theme.border_subtle),
        corner_radius: egui::CornerRadius::same(RADIUS_MD as u8),
        ..Default::default()
    };

    let response = card_frame
        .show(ui, |ui| {
            ui.set_min_width(cards_width - SPACE_MD * 2.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(char::from(icon).to_string())
                        .font(font_icon(ICON_DEFAULT))
                        .color(theme.accent),
                );
                ui.add_space(SPACE_XS);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.label(
                        RichText::new(title)
                            .font(font_ui_label())
                            .strong()
                            .color(theme.text_primary),
                    );
                    ui.label(RichText::new(desc).font(font_caption()).color(theme.text_muted));
                });
            });
        })
        .response;

    let interact_resp = ui.interact(response.rect, response.id, egui::Sense::click());
    if interact_resp.hovered() {
        ui.painter().rect_stroke(
            response.rect,
            egui::CornerRadius::same(RADIUS_MD as u8),
            egui::Stroke::new(STROKE_THIN, theme.border_strong),
            egui::StrokeKind::Inside,
        );
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    interact_resp.clicked()
}
