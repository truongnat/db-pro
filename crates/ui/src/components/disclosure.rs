//! Shared visual language for collapsible and accordion disclosures.

use crate::components::animation::{hover_t, lerp_color};
use crate::components::interact::paint_focus_ring;
use crate::tokens::{font_icon, ICON_SM, RADIUS_SM};
use crate::DbProTheme;
use egui::{Align2, Color32, Id, Pos2, Response, Ui};
use lucide_icons::Icon;

pub(crate) const HEADER_HEIGHT: f32 = 32.0;
pub(crate) const CONTENT_MARGIN: egui::Margin = egui::Margin {
    left: 32.0,
    right: 12.0,
    top: 8.0,
    bottom: 12.0,
};

pub(crate) fn show_body<R>(
    ui: &mut Ui,
    state: &mut egui::collapsing_header::CollapsingState,
    content: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    let openness = state.openness(ui.ctx());
    state
        .show_body_unindented(ui, |ui| {
            egui::Frame::none()
                .inner_margin(CONTENT_MARGIN)
                .show(ui, |ui| {
                    ui.set_opacity(openness);
                    content(ui)
                })
                .inner
        })
        .map(|response| response.inner)
}

pub(crate) fn paint_header_surface(
    ui: &mut Ui,
    id: Id,
    response: &Response,
    theme: DbProTheme,
    open_t: f32,
    disabled: bool,
) -> f32 {
    let hover = hover_t(ui.ctx(), id.with("hover"), response.hovered() && !disabled);
    if !disabled {
        let hover_fill = lerp_color(Color32::TRANSPARENT, theme.surface_hover, hover);
        let fill = lerp_color(hover_fill, theme.surface_active, open_t);
        if fill != Color32::TRANSPARENT {
            ui.painter()
                .rect_filled(response.rect, egui::Rounding::same(RADIUS_SM), fill);
        }
    }
    if response.has_focus() && !disabled {
        paint_focus_ring(ui, response.rect, RADIUS_SM, theme);
    }
    hover
}

pub(crate) struct HeaderColorState<'a> {
    pub(crate) theme: &'a DbProTheme,
    pub(crate) disabled: bool,
    pub(crate) emphasized: bool,
}

pub(crate) fn header_colors(state: HeaderColorState<'_>) -> (Color32, Color32) {
    if state.disabled {
        (state.theme.text_disabled, state.theme.text_disabled)
    } else if state.emphasized {
        (state.theme.text_primary, state.theme.text_primary)
    } else {
        (state.theme.text_secondary, state.theme.text_secondary)
    }
}

pub(crate) struct ChevronPaint {
    pub(crate) position: Pos2,
    pub(crate) align: Align2,
    pub(crate) color: Color32,
    pub(crate) open_t: f32,
}

pub(crate) fn paint_chevron(ui: &Ui, paint: ChevronPaint) {
    let closed_alpha = (1.0 - paint.open_t).clamp(0.0, 1.0);
    let open_alpha = paint.open_t.clamp(0.0, 1.0);
    ui.painter().text(
        paint.position,
        paint.align,
        char::from(Icon::ChevronRight).to_string(),
        font_icon(ICON_SM),
        paint.color.linear_multiply(closed_alpha),
    );
    ui.painter().text(
        paint.position,
        paint.align,
        char::from(Icon::ChevronDown).to_string(),
        font_icon(ICON_SM),
        paint.color.linear_multiply(open_alpha),
    );
}
