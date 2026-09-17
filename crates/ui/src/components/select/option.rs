use super::config::ITEM_HEIGHT;
use crate::components::animation::{hover_t, lerp_color};
use crate::DbProTheme;
use egui::{Color32, FontFamily, FontId, Pos2, Rect, Response, RichText, Rounding, Sense, Ui, Vec2};
use lucide_icons::Icon;

pub struct SelectOption<'a> {
    pub label: &'a str,
    pub selected: bool,
    pub theme: DbProTheme,
}

pub fn paint_option(ui: &mut Ui, option: SelectOption<'_>) -> Response {
    let SelectOption { label, selected, theme } = option;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), ITEM_HEIGHT), Sense::click());
    let hover = hover_t(ui.ctx(), response.id.with("opt"), response.hovered() && !selected);
    let bg = if selected {
        theme.accent_soft
    } else {
        lerp_color(Color32::TRANSPARENT, theme.surface_hover, hover)
    };
    ui.painter().rect_filled(rect, Rounding::same(6.0), bg);
    let text_color = if selected { theme.accent } else { theme.text_primary };
    let text_right = if selected {
        rect.right() - 28.0
    } else {
        rect.right() - 8.0
    };
    ui.put(
        Rect::from_min_max(
            Pos2::new(rect.left() + 8.0, rect.top()),
            Pos2::new(text_right, rect.bottom()),
        ),
        egui::Label::new(RichText::new(label).font(FontId::proportional(13.0)).color(text_color)).truncate(),
    );
    if selected {
        ui.painter().text(
            Pos2::new(rect.right() - 8.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            char::from(Icon::Check).to_string(),
            FontId::new(12.0, FontFamily::Name("lucide".into())),
            theme.accent,
        );
    }
    response
}
