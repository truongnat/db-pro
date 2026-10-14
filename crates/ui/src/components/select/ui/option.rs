use super::super::handler::{option_check_icon_pos, option_galley_pos, option_text_rect};
use crate::components::animation::{hover_t, lerp_color};
use crate::tokens::component::table::TABLE_ROW_HEIGHT_COMPACT;
use crate::tokens::{FONT_SIZE_UI_LABEL, ICON_XS};
use crate::DbProTheme;
use egui::{pos2, Color32, FontFamily, FontId, Response, Sense, Ui, Vec2};
use lucide_icons::Icon;

pub struct SelectOption<'a> {
    pub label: &'a str,
    pub selected: bool,
    pub theme: DbProTheme,
    /// Optional row icon painted before the label (e.g. engine brand mark).
    pub icon: Option<egui::TextureId>,
    /// Rows flagged `disabled` render muted and don't accept clicks.
    pub disabled: bool,
}

pub fn paint_option(ui: &mut Ui, option: SelectOption<'_>) -> Response {
    let SelectOption {
        label,
        selected,
        theme,
        icon,
        disabled,
    } = option;
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), TABLE_ROW_HEIGHT_COMPACT),
        if disabled { Sense::hover() } else { Sense::click() },
    );
    let hover = hover_t(
        ui.ctx(),
        response.id.with("opt"),
        response.hovered() && !selected && !disabled,
    );
    let bg = if selected {
        theme.accent_soft
    } else {
        lerp_color(Color32::TRANSPARENT, theme.surface_hover, hover)
    };
    ui.painter()
        .rect_filled(rect, ui.style().visuals.widgets.inactive.corner_radius, bg);

    // Optional row icon sits left of the label at 16px.
    let mut text_left = rect.left();
    if let Some(tex) = icon {
        let icon_side = 16.0;
        let icon_rect = egui::Rect::from_center_size(
            pos2(
                text_left + ui.spacing().button_padding.x + icon_side * 0.5,
                rect.center().y,
            ),
            egui::vec2(icon_side, icon_side),
        );
        ui.painter().image(
            tex,
            icon_rect,
            egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            if disabled { theme.text_disabled } else { Color32::WHITE },
        );
        text_left += ui.spacing().button_padding.x + icon_side + crate::tokens::SPACE_XS;
    }

    let text_color = if selected {
        theme.accent
    } else if disabled {
        theme.text_disabled
    } else {
        theme.text_primary
    };
    // Measure/wrap text within the helper-computed rectangle, keeping layout policy testable outside egui painting.
    let text_rect = option_text_rect(rect.with_min_x(text_left), ui.spacing().button_padding.x, selected);
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
    } else if disabled {
        ui.painter().text(
            option_check_icon_pos(rect.right(), rect.center().y),
            egui::Align2::RIGHT_CENTER,
            "soon",
            FontId::proportional(FONT_SIZE_UI_LABEL * 0.8),
            theme.text_disabled,
        );
    }
    let response = response.on_hover_cursor(if disabled {
        egui::CursorIcon::NotAllowed
    } else {
        egui::CursorIcon::PointingHand
    });
    // Disabled rows never become clickable — no swallow hack needed.
    response
}
