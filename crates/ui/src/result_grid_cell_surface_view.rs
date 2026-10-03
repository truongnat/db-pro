//! Pure result-grid cell surface and value rendering.

use super::*;
use egui::{Align2, FontId, Pos2, Rect, Rounding, Stroke};

pub(super) struct GridCellSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) row_hovered: bool,
    pub(super) row_selected: bool,
    pub(super) cell_selected: bool,
    pub(super) row_dirty: bool,
    pub(super) validation_error: bool,
    pub(super) conflict_error: bool,
    pub(super) cell_mutation_error: bool,
    pub(super) row_mutation_error: bool,
    pub(super) edit_error: Option<&'a str>,
    pub(super) mutation_error: Option<&'a str>,
}

pub(super) fn draw_surface(
    context: &GridCellSurfaceContext<'_>,
    ui: &mut egui::Ui,
    cell_rect: Rect,
    cell_response: &egui::Response,
) {
    let fill = if context.validation_error || (context.cell_mutation_error && !context.conflict_error) {
        context.theme.danger.linear_multiply(0.14)
    } else if context.conflict_error {
        context.theme.warning.linear_multiply(0.16)
    } else if context.row_mutation_error {
        context.theme.danger.linear_multiply(0.08)
    } else if context.cell_selected {
        // the active cell keeps the deeper wash; the spec shows row and cell
        // selection as two distinct levels
        context.theme.accent_soft
    } else if context.row_selected {
        context.theme.soft_tint(context.theme.accent)
    } else if context.row_dirty {
        context.theme.warning.linear_multiply(0.12)
    } else if context.row_hovered {
        context.theme.surface_hover
    } else {
        context.theme.surface_editor
    };
    ui.painter().rect_filled(cell_rect, Rounding::ZERO, fill);
    let border = Stroke::new(1.0, context.theme.border_subtle);
    ui.painter().hline(cell_rect.x_range(), cell_rect.bottom(), border);
    ui.painter().vline(cell_rect.right(), cell_rect.y_range(), border);
    if context.cell_selected {
        // `ring-inset` from the spec — the outline stays inside the cell so it
        // never bleeds over neighboring borders
        ui.painter().rect_stroke(
            cell_rect.shrink(1.0),
            Rounding::ZERO,
            Stroke::new(1.5, context.theme.accent),
        );
    }
    if context.validation_error {
        ui.painter().rect_stroke(
            cell_rect.shrink(1.0),
            Rounding::ZERO,
            Stroke::new(1.5, context.theme.danger),
        );
        if let Some(error) = context.edit_error {
            cell_response.clone().on_hover_text(error);
        }
    }
    if context.cell_mutation_error || context.row_mutation_error {
        ui.painter().rect_stroke(
            cell_rect.shrink(1.0),
            Rounding::ZERO,
            Stroke::new(
                1.5,
                if context.conflict_error {
                    context.theme.warning
                } else {
                    context.theme.danger
                },
            ),
        );
        if let Some(error) = context.mutation_error {
            cell_response.clone().on_hover_text(error);
        }
    }
}

pub(super) fn draw_value(
    theme: DbProTheme,
    ui: &mut egui::Ui,
    cell_rect: Rect,
    cell_response: &egui::Response,
    display_cell: &UiCell,
) {
    let content_rect = cell_rect.shrink2(egui::vec2(10.0, 3.0));
    let raw_value = crate::result_grid::cell_text_as_str(display_cell);
    let painter = ui.painter().with_clip_rect(content_rect);
    match display_cell {
        UiCell::Null => draw_badge(
            &painter,
            content_rect,
            BadgeAlign::Start,
            "NULL",
            FontId::monospace(10.0),
            BadgeStyle {
                foreground: theme.text_muted,
                fill: theme.surface_hover,
                border: theme.border_default,
            },
        ),
        UiCell::Boolean(value) => {
            let status = if *value {
                theme.semantic.status.success
            } else {
                theme.semantic.status.danger
            };
            draw_badge(
                &painter,
                content_rect,
                BadgeAlign::Center,
                if *value { "TRUE" } else { "FALSE" },
                FontId::proportional(10.5),
                BadgeStyle {
                    foreground: status.foreground,
                    fill: status.subtle,
                    border: status.border,
                },
            );
        }
        _ => {
            let text_color = match display_cell {
                UiCell::Number(_) | UiCell::Text(_) => theme.text_primary,
                _ => theme.text_secondary,
            };
            painter.text(
                Pos2::new(content_rect.left(), content_rect.center().y),
                Align2::LEFT_CENTER,
                raw_value,
                FontId::monospace(12.0),
                text_color,
            );
        }
    }
    if cell_response.hovered() && raw_value.len() > 36 {
        cell_response.clone().on_hover_text(raw_value);
    }
}

#[derive(Clone, Copy)]
enum BadgeAlign {
    Start,
    Center,
}

#[derive(Clone, Copy)]
struct BadgeStyle {
    foreground: egui::Color32,
    fill: egui::Color32,
    border: egui::Color32,
}

fn draw_badge(
    painter: &egui::Painter,
    content_rect: Rect,
    align: BadgeAlign,
    label: &str,
    font: FontId,
    style: BadgeStyle,
) {
    let galley = painter.layout_no_wrap(label.to_owned(), font, style.foreground);
    let pad = egui::vec2(7.0, 2.0);
    let size = galley.size() + pad * 2.0;
    let origin = Pos2::new(
        match align {
            BadgeAlign::Start => content_rect.left(),
            BadgeAlign::Center => content_rect.center().x - size.x * 0.5,
        },
        content_rect.center().y - size.y * 0.5,
    );
    let badge_rect = Rect::from_min_size(origin, size);
    painter.rect_filled(badge_rect, Rounding::same(4.0), style.fill);
    painter.rect_stroke(badge_rect, Rounding::same(4.0), Stroke::new(1.0, style.border));
    painter.galley(origin + pad, galley, style.foreground);
}
