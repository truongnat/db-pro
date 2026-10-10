//! A modern ChatGPT-inspired introduction with interactive quick-action cards.
use super::*;
use crate::components::{Container, ResponsiveGrid};
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

    ui.vertical_centered(|ui| {
        draw_header(theme, ui);
    });
    ui.add_space(SPACE_2XL);

    draw_quick_actions(theme, ui)
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

const CARD_GAP: f32 = 12.0;
const CARD_HEIGHT: f32 = 64.0;

fn draw_quick_actions(theme: DbProTheme, ui: &mut egui::Ui) -> Option<WelcomeAction> {
    let items = welcome_items();
    let mut triggered = None;

    Container::new()
        .max_width(620.0)
        .gutter(SPACE_MD)
        .show(ui, |ui| {
            let actions = ResponsiveGrid::new(280.0)
                .gap(CARD_GAP)
                .max_columns(2)
                .show(ui, &items, |cell_ui, item| {
                    draw_card_item(theme, cell_ui, item)
                });
            for action in actions.into_iter().flatten() {
                triggered = Some(action);
            }
        });

    triggered
}

fn draw_card_item(
    theme: DbProTheme,
    ui: &mut egui::Ui,
    item: &WelcomeItem,
) -> Option<WelcomeAction> {
    let card_size = egui::vec2(ui.available_width(), CARD_HEIGHT);
    let (rect, resp) = ui.allocate_exact_size(card_size, egui::Sense::click());
    let hovered = resp.hovered();

    let bg = if hovered {
        theme.surface_hover
    } else {
        theme.surface_elevated
    };
    let stroke_color = if hovered {
        theme.border_strong
    } else {
        theme.border_subtle
    };
    let corner_radius = egui::CornerRadius::same(RADIUS_MD as u8);

    ui.painter().rect_filled(rect, corner_radius, bg);
    ui.painter().rect_stroke(
        rect,
        corner_radius,
        egui::Stroke::new(STROKE_THIN, stroke_color),
        egui::StrokeKind::Inside,
    );

    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let content_rect = rect.shrink2(egui::vec2(SPACE_MD, SPACE_SM));
    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(content_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    child_ui.label(
        RichText::new(char::from(item.icon).to_string())
            .font(font_icon(ICON_DEFAULT))
            .color(theme.accent),
    );
    child_ui.add_space(SPACE_XS);

    child_ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 1.0;
        ui.label(
            RichText::new(item.title)
                .font(font_ui_label())
                .strong()
                .color(theme.text_primary),
        );
        ui.add(
            egui::Label::new(RichText::new(item.desc).font(font_caption()).color(theme.text_muted)).wrap(),
        );
    });

    if resp.clicked() {
        Some(item.action)
    } else {
        None
    }
}
