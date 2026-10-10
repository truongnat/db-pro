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
    let avail_width = ui.available_width();
    let is_wide = avail_width >= 620.0;
    let container_width = if is_wide { 620.0 } else { (avail_width - 32.0).max(280.0) };
    let side_margin = ((avail_width - container_width) * 0.5).max(0.0);

    let mut triggered = None;
    ui.horizontal(|ui| {
        ui.add_space(side_margin);
        ui.vertical(|ui| {
            ui.set_width(container_width);
            if is_wide {
                triggered = draw_wide_rows(theme, ui, container_width, &items);
            } else {
                triggered = draw_stacked_rows(theme, ui, container_width, &items);
            }
        });
    });
    triggered
}

fn draw_wide_rows(
    theme: DbProTheme,
    ui: &mut egui::Ui,
    width: f32,
    items: &[WelcomeItem; 4],
) -> Option<WelcomeAction> {
    let card_w = ((width - CARD_GAP) * 0.5).floor();
    let card_size = egui::vec2(card_w, CARD_HEIGHT);
    let mut action = None;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = CARD_GAP;
        if let Some(act) = draw_card_item(theme, ui, card_size, &items[0]) {
            action = Some(act);
        }
        if let Some(act) = draw_card_item(theme, ui, card_size, &items[1]) {
            action = Some(act);
        }
    });
    ui.add_space(CARD_GAP);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = CARD_GAP;
        if let Some(act) = draw_card_item(theme, ui, card_size, &items[2]) {
            action = Some(act);
        }
        if let Some(act) = draw_card_item(theme, ui, card_size, &items[3]) {
            action = Some(act);
        }
    });
    action
}

fn draw_stacked_rows(
    theme: DbProTheme,
    ui: &mut egui::Ui,
    width: f32,
    items: &[WelcomeItem; 4],
) -> Option<WelcomeAction> {
    let card_size = egui::vec2(width, CARD_HEIGHT);
    let mut action = None;
    for item in items {
        if let Some(act) = draw_card_item(theme, ui, card_size, item) {
            action = Some(act);
        }
        ui.add_space(CARD_GAP);
    }
    action
}

fn draw_card_item(
    theme: DbProTheme,
    ui: &mut egui::Ui,
    card_size: egui::Vec2,
    item: &WelcomeItem,
) -> Option<WelcomeAction> {
    let (rect, mut response) = ui.allocate_exact_size(card_size, egui::Sense::click());
    response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let hovered = response.hovered();

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

    let icon_char = char::from(item.icon).to_string();
    let icon_pos = egui::pos2(rect.left() + SPACE_MD, rect.center().y - ICON_DEFAULT * 0.5);
    ui.painter().text(
        icon_pos,
        egui::Align2::LEFT_TOP,
        icon_char,
        font_icon(ICON_DEFAULT),
        theme.accent,
    );

    let text_x = rect.left() + SPACE_MD + ICON_DEFAULT + SPACE_SM;
    let max_text_w = (rect.right() - SPACE_MD - text_x).max(10.0);

    let title_job = egui::text::LayoutJob::simple_singleline(
        item.title.to_string(),
        font_ui_label(),
        theme.text_primary,
    );
    let title_galley = ui.fonts_mut(|f| f.layout_job(title_job));

    let mut desc_job = egui::text::LayoutJob::simple(
        item.desc.to_string(),
        font_caption(),
        theme.text_muted,
        max_text_w,
    );
    desc_job.wrap.max_width = max_text_w;
    let desc_galley = ui.fonts_mut(|f| f.layout_job(desc_job));

    let title_h = title_galley.size().y;
    let desc_h = desc_galley.size().y;
    let total_h = title_h + 2.0 + desc_h;
    let text_top = (rect.center().y - total_h * 0.5).round();

    ui.painter().galley(egui::pos2(text_x, text_top), title_galley, theme.text_primary);
    ui.painter().galley(egui::pos2(text_x, text_top + title_h + 2.0), desc_galley, theme.text_muted);

    if response.clicked() {
        Some(item.action)
    } else {
        None
    }
}
