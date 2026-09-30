use super::super::handler::{option_check_icon_pos, option_galley_pos, option_text_rect};
use crate::components::animation::{hover_t, lerp_color};
use crate::tokens::component::table::TABLE_ROW_HEIGHT_COMPACT;
use crate::tokens::{FONT_SIZE_UI_LABEL, ICON_XS};
use crate::DbProTheme;
use egui::{Color32, FontFamily, FontId, Response, Sense, Ui, Vec2};
use lucide_icons::Icon;

pub struct SelectOption<'a> {
    pub label: &'a str,
    pub selected: bool,
    pub theme: DbProTheme,
}

pub fn paint_option(ui: &mut Ui, option: SelectOption<'_>) -> Response {
    let SelectOption { label, selected, theme } = option;
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), TABLE_ROW_HEIGHT_COMPACT),
        Sense::click(),
    );
    let hover = hover_t(ui.ctx(), response.id.with("opt"), response.hovered() && !selected);
    let bg = if selected {
        theme.accent_soft
    } else {
        lerp_color(Color32::TRANSPARENT, theme.surface_hover, hover)
    };
    ui.painter()
        .rect_filled(rect, ui.style().visuals.widgets.inactive.rounding, bg);
    let text_color = if selected { theme.accent } else { theme.text_primary };
    // Measure/wrap text within the helper-computed rectangle, keeping layout policy testable outside egui painting.
    let text_rect = option_text_rect(rect, ui.spacing().button_padding.x, selected);
    let galley = ui.painter().layout(
        label.to_owned(),
        FontId::proportional(FONT_SIZE_UI_LABEL),
        text_color,
        text_rect.width(),
    );
    // Position the measured galley separately so vertical centering remains independent of text measurement.
    let galley_pos = option_galley_pos(text_rect.left(), text_rect.center().y, galley.size().y);
    ui.painter().galley(galley_pos, galley, text_color);
    if selected {
        let check_pos = option_check_icon_pos(rect.right(), rect.center().y);
        ui.painter().text(
            check_pos,
            egui::Align2::RIGHT_CENTER,
            char::from(Icon::Check).to_string(),
            FontId::new(ICON_XS, FontFamily::Name("lucide".into())),
            theme.accent,
        );
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
