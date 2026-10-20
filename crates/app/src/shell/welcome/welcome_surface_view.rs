//! A modern ChatGPT-inspired introduction with interactive quick-action cards.
use std::borrow::Cow;

use super::*;

#[path = "constants.rs"]
mod constants;
pub(crate) use constants::WelcomeAction;
use constants::*;
use egui::{text::LayoutJob, RichText};
use lucide_icons::Icon;

/// egui temp store key: the translated card items, invalidated per locale so a
/// language switch re-reads the catalog without allocating four Strings and
/// eight `t!` lookups every frame.
const ITEMS_CACHE_ID: &str = "welcome/items";

struct WelcomeItem {
    icon: Icon,
    title: Cow<'static, str>,
    desc: Cow<'static, str>,
    action: WelcomeAction,
}

impl Clone for WelcomeItem {
    fn clone(&self) -> Self {
        Self {
            icon: self.icon,
            title: self.title.clone(),
            desc: self.desc.clone(),
            action: self.action,
        }
    }
}

/// Rebuild the card list only when the active locale changes; the translated
/// strings are cheap to clone (`Cow`) after the first lookup.
fn welcome_items(ctx: &egui::Context) -> [WelcomeItem; 4] {
    let locale = rust_i18n::locale().to_string();
    ctx.data_mut(|data| {
        let cached = data
            .get_temp::<(Cow<'static, str>, [WelcomeItem; 4])>(egui::Id::new(ITEMS_CACHE_ID))
            .filter(|(cached_locale, _)| *cached_locale == locale);
        match cached {
            Some((_, items)) => items,
            None => {
                let items = [
                    WelcomeItem {
                        icon: Icon::Plus,
                        title: t!("welcome.new_connection"),
                        desc: t!("welcome.new_connection_desc"),
                        action: WelcomeAction::NewConnection,
                    },
                    WelcomeItem {
                        icon: Icon::FileCode2,
                        title: t!("welcome.new_query"),
                        desc: t!("welcome.new_query_desc"),
                        action: WelcomeAction::NewQuery,
                    },
                    WelcomeItem {
                        icon: Icon::Sparkles,
                        title: t!("welcome.ai_copilot"),
                        desc: t!("welcome.ai_copilot_desc"),
                        action: WelcomeAction::ToggleAgent,
                    },
                    WelcomeItem {
                        icon: Icon::Workflow,
                        title: t!("welcome.er_diagram"),
                        desc: t!("welcome.er_diagram_desc"),
                        action: WelcomeAction::OpenDiagram,
                    },
                ];
                data.insert_temp(egui::Id::new(ITEMS_CACHE_ID), (locale.clone(), items.clone()));
                items
            }
        }
    })
}

pub(super) fn draw(theme: DbProTheme, ui: &mut egui::Ui) -> Option<WelcomeAction> {
    let top_space = (ui.available_height() * HEADER_TOP_RATIO).clamp(SPACE_MD, SPACE_4XL);
    crate::components::scroll_area::ScrollArea::new(theme)
        .id_salt("welcome-actions")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(top_space);
            ui.vertical_centered(|ui| draw_header(theme, ui));
            ui.add_space(SPACE_2XL);
            draw_quick_actions(theme, ui)
        })
}

fn draw_header(theme: DbProTheme, ui: &mut egui::Ui) {
    ui.label(
        RichText::new(char::from(Icon::Database).to_string())
            .font(font_icon(ICON_XL))
            .color(theme.accent),
    );
    ui.add_space(SPACE_MD);
    ui.label(
        RichText::new(t!("welcome.title"))
            .font(font_display())
            .color(theme.text_primary),
    );
    ui.add_space(SPACE_XS);
    ui.label(
        RichText::new(t!("welcome.tagline"))
            .font(font_body())
            .color(theme.text_secondary),
    );
    ui.add_space(SPACE_XS);
    ui.label(
        RichText::new(t!("welcome.drivers"))
            .font(font_caption())
            .color(theme.text_muted),
    );
}

fn draw_quick_actions(theme: DbProTheme, ui: &mut egui::Ui) -> Option<WelcomeAction> {
    let items = welcome_items(ui.ctx());
    let avail_width = ui.available_width();
    let is_wide = avail_width >= WIDE_GRID_MIN_WIDTH;
    let container_width = avail_width.clamp(0.0, WIDE_GRID_WIDTH);
    let side_margin = ((avail_width - container_width) * 0.5).max(0.0);

    let mut triggered = None;
    ui.horizontal(|ui| {
        ui.add_space(side_margin);
        ui.vertical(|ui| {
            ui.set_width(container_width);
            ui.spacing_mut().item_spacing = egui::vec2(SPACE_MD, SPACE_MD);
            triggered = if is_wide {
                draw_wide_rows(theme, ui, container_width, &items)
            } else {
                draw_stacked_rows(theme, ui, container_width, &items)
            };
        });
    });
    triggered
}

fn draw_wide_rows(theme: DbProTheme, ui: &mut egui::Ui, width: f32, items: &[WelcomeItem; 4]) -> Option<WelcomeAction> {
    let card_w = ((width - SPACE_MD) * 0.5).floor();
    let card_size = egui::vec2(card_w, CARD_HEIGHT);
    let mut action = None;
    for row in items.chunks(2) {
        ui.horizontal(|ui| {
            for item in row {
                if let Some(act) = draw_card_item(theme, ui, card_size, item) {
                    action = Some(act);
                }
            }
        });
    }
    action
}

fn draw_stacked_rows(
    theme: DbProTheme,
    ui: &mut egui::Ui,
    width: f32,
    items: &[WelcomeItem; 4],
) -> Option<WelcomeAction> {
    let card_size = egui::vec2(width, CARD_HEIGHT);
    items.iter().find_map(|item| draw_card_item(theme, ui, card_size, item))
}

fn draw_card_item(
    theme: DbProTheme,
    ui: &mut egui::Ui,
    card_size: egui::Vec2,
    item: &WelcomeItem,
) -> Option<WelcomeAction> {
    let (rect, mut response) = ui.allocate_exact_size(card_size, egui::Sense::click());
    response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let title = item.title.as_ref();
    // Screen-reader name: "Title — description"; mirrors the visual card text.
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, format!("{title} — {}", item.desc)));
    // Keyboard activation: egui focus + Enter/Space behaves like a click.
    let keyboard_activated =
        response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));
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
    if response.has_focus() {
        crate::components::interact::paint_focus_ring(ui, rect, RADIUS_MD, theme);
    }

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
    let max_text_w = (rect.right() - SPACE_MD - text_x).max(1.0);

    let title_galley = ui.fonts_mut(|f| {
        let mut job = LayoutJob::simple_singleline(title.to_owned(), font_ui_label(), theme.text_primary);
        job.wrap.max_width = max_text_w;
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = true;
        f.layout_job(job)
    });
    let desc_galley = ui.fonts_mut(|f| {
        f.layout_job(LayoutJob::simple(
            item.desc.to_string(),
            font_caption(),
            theme.text_secondary,
            max_text_w,
        ))
    });

    let title_h = title_galley.size().y;
    let desc_h = desc_galley.size().y;
    let total_h = title_h + SPACE_XXS + desc_h;
    let text_top = (rect.center().y - total_h * 0.5).round();

    ui.painter()
        .galley(egui::pos2(text_x, text_top), title_galley, theme.text_primary);
    ui.painter().galley(
        egui::pos2(text_x, text_top + title_h + SPACE_XXS),
        desc_galley,
        theme.text_secondary,
    );

    if response.clicked() || keyboard_activated {
        Some(item.action)
    } else {
        None
    }
}

#[cfg(test)]
#[path = "tests/welcome_surface_view_tests.rs"]
mod tests;
