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

struct WelcomeItem {
    icon: Icon,
    title: &'static str,
    desc: &'static str,
    action: WelcomeAction,
}

fn welcome_items() -> [WelcomeItem; 4] {
    [
        WelcomeItem {
            icon: Icon::Plus,
            title: "New Connection",
            desc: "Connect to PostgreSQL, SQLite or remote database",
            action: WelcomeAction::NewConnection,
        },
        WelcomeItem {
            icon: Icon::FileCode2,
            title: "New SQL Query",
            desc: "Open a blank scratchpad with auto-completion",
            action: WelcomeAction::NewQuery,
        },
        WelcomeItem {
            icon: Icon::Sparkles,
            title: "AI Copilot Assistant",
            desc: "Generate queries, explain schema or review SQL",
            action: WelcomeAction::ToggleAgent,
        },
        WelcomeItem {
            icon: Icon::Workflow,
            title: "ER Diagram Canvas",
            desc: "Visualize schema tables and relationships",
            action: WelcomeAction::OpenDiagram,
        },
    ]
}

fn draw_quick_actions(theme: DbProTheme, ui: &mut egui::Ui) -> Option<WelcomeAction> {
    let avail_width = ui.available_width();
    let is_wide = avail_width >= 660.0;
    let container_width = if is_wide { 620.0 } else { 480.0_f32.min(avail_width - 32.0) };
    let side_margin = ((avail_width - container_width) * 0.5).max(0.0);

    let mut triggered = None;
    ui.horizontal(|ui| {
        ui.add_space(side_margin);
        ui.vertical(|ui| {
            ui.set_width(container_width);
            ui.set_max_width(container_width);
            let items = welcome_items();
            if is_wide {
                triggered = draw_wide_grid(theme, ui, container_width, &items);
            } else {
                triggered = draw_narrow_stack(theme, ui, container_width, &items);
            }
        });
    });
    triggered
}

fn draw_wide_grid(
    theme: DbProTheme,
    ui: &mut egui::Ui,
    container_width: f32,
    items: &[WelcomeItem; 4],
) -> Option<WelcomeAction> {
    let card_w = (container_width - SPACE_SM) * 0.5;
    let mut action = None;

    // Row 1
    ui.horizontal(|ui| {
        if draw_card_item(theme, ui, card_w, items[0].icon, items[0].title, items[0].desc) {
            action = Some(items[0].action);
        }
        ui.add_space(SPACE_SM);
        if draw_card_item(theme, ui, card_w, items[1].icon, items[1].title, items[1].desc) {
            action = Some(items[1].action);
        }
    });

    ui.add_space(SPACE_SM);

    // Row 2
    ui.horizontal(|ui| {
        if draw_card_item(theme, ui, card_w, items[2].icon, items[2].title, items[2].desc) {
            action = Some(items[2].action);
        }
        ui.add_space(SPACE_SM);
        if draw_card_item(theme, ui, card_w, items[3].icon, items[3].title, items[3].desc) {
            action = Some(items[3].action);
        }
    });

    action
}

fn draw_narrow_stack(
    theme: DbProTheme,
    ui: &mut egui::Ui,
    container_width: f32,
    items: &[WelcomeItem; 4],
) -> Option<WelcomeAction> {
    let mut action = None;
    for item in items {
        if draw_card_item(theme, ui, container_width, item.icon, item.title, item.desc) {
            action = Some(item.action);
        }
        ui.add_space(SPACE_XS);
    }
    action
}

fn draw_card_item(
    theme: DbProTheme,
    ui: &mut egui::Ui,
    card_width: f32,
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

    let inner_w = (card_width - SPACE_MD * 2.0).max(60.0);
    const CARD_CONTENT_HEIGHT: f32 = 46.0;

    let response = card_frame
        .show(ui, |ui| {
            ui.set_min_size(egui::vec2(inner_w, CARD_CONTENT_HEIGHT));
            ui.set_max_size(egui::vec2(inner_w, CARD_CONTENT_HEIGHT));
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(char::from(icon).to_string())
                        .font(font_icon(ICON_DEFAULT))
                        .color(theme.accent),
                );
                ui.add_space(SPACE_XS);
                ui.vertical(|ui| {
                    ui.set_width(inner_w - ICON_DEFAULT - SPACE_XS);
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
