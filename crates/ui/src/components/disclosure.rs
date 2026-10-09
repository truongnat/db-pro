//! Shared visual language for collapsible and accordion disclosures.

use crate::components::animation::{hover_t, lerp_color};
use crate::components::interact::paint_focus_ring;
use crate::tokens::{font_icon, ICON_SM, RADIUS_SM};
use crate::DbProTheme;
use egui::{Align2, Color32, Id, Pos2, Response, Ui};
use lucide_icons::Icon;

pub(crate) const HEADER_HEIGHT: f32 = 32.0;
pub(crate) const CONTENT_MARGIN: egui::Margin = egui::Margin {
    left: 32.0 as i8,
    right: 12.0 as i8,
    top: 8.0 as i8,
    bottom: 12.0 as i8,
};

pub(crate) fn disclosure_progress(is_open: bool, reduce_motion: bool, animate: impl FnOnce() -> f32) -> f32 {
    if reduce_motion {
        if is_open {
            1.0
        } else {
            0.0
        }
    } else {
        animate()
    }
}

pub(crate) fn show_body<R>(
    ui: &mut Ui,
    state: &mut egui::collapsing_header::CollapsingState,
    reduce_motion: bool,
    content: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    if reduce_motion {
        // Bypass CollapsingState's animated clipping so the preference also stops body motion.
        let open = state.is_open();
        state.store(ui.ctx());
        return open.then(|| {
            ui.scope(|ui| {
                egui::Frame::NONE
                    .inner_margin(CONTENT_MARGIN)
                    .show(ui, |ui| {
                        ui.set_opacity(1.0);
                        content(ui)
                    })
                    .inner
            })
            .inner
        });
    }

    let openness = state.openness(ui.ctx());
    state
        .show_body_unindented(ui, |ui| {
            egui::Frame::NONE
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
    let is_hovered = response.hovered() && !disabled;
    let hover = if theme.reduce_motion {
        if is_hovered {
            1.0
        } else {
            0.0
        }
    } else {
        hover_t(ui.ctx(), id.with("hover"), is_hovered)
    };
    if !disabled {
        let hover_fill = lerp_color(Color32::TRANSPARENT, theme.surface_hover, hover);
        let fill = lerp_color(hover_fill, theme.surface_active, open_t);
        if fill != Color32::TRANSPARENT {
            ui.painter()
                .rect_filled(response.rect, egui::CornerRadius::same(RADIUS_SM as u8), fill);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduced_motion_disclosure_progress_is_immediate() {
        let mut animated = false;
        assert_eq!(
            disclosure_progress(true, true, || {
                animated = true;
                0.2
            }),
            1.0
        );
        assert!(!animated);
        assert_eq!(disclosure_progress(false, true, || 0.8), 0.0);
        assert_eq!(disclosure_progress(true, false, || 0.2), 0.2);
    }
}
