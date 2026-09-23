//! Pure visual rendering for a result-grid column header.

use super::*;
use egui::{FontId, Pos2, Rect, Rounding, Vec2};

pub(super) struct GridHeaderContentContext<'a> {
    pub(super) column: &'a crate::UiColumn,
    pub(super) col_rect: Rect,
    pub(super) is_primary_key: bool,
    pub(super) is_foreign_key: bool,
    pub(super) sort_marker: &'a str,
    pub(super) sort_active: bool,
    pub(super) theme: DbProTheme,
}

struct HeaderTextLayout {
    header_rect: Rect,
    text_x: f32,
}

pub(super) fn draw_header_content(context: &GridHeaderContentContext<'_>, ui: &egui::Ui) {
    let header_text_rect = context.col_rect.shrink2(egui::vec2(8.0, 4.0));
    let painter = ui.painter().with_clip_rect(header_text_rect);
    let text_x = draw_key_badge(
        &painter,
        header_text_rect,
        context.is_primary_key,
        context.is_foreign_key,
        context.theme,
    );
    draw_header_text(
        &painter,
        context,
        HeaderTextLayout {
            header_rect: header_text_rect,
            text_x,
        },
    );
}

fn draw_key_badge(
    painter: &egui::Painter,
    header_rect: Rect,
    is_primary_key: bool,
    is_foreign_key: bool,
    theme: DbProTheme,
) -> f32 {
    let (label, color) = if is_primary_key {
        ("PK", theme.warning)
    } else if is_foreign_key {
        ("FK", theme.accent)
    } else {
        return header_rect.left();
    };
    let galley = painter.layout_no_wrap(
        label.to_owned(),
        FontId::new(9.5, egui::FontFamily::Proportional),
        color,
    );
    let badge_rect = Rect::from_min_size(
        Pos2::new(header_rect.left(), header_rect.center().y - 7.0),
        Vec2::new(galley.size().x + 6.0, 14.0),
    );
    painter.rect_filled(badge_rect, Rounding::same(3.0), color.linear_multiply(0.18));
    painter.galley(
        Pos2::new(badge_rect.left() + 3.0, badge_rect.center().y - galley.size().y * 0.5),
        galley,
        color,
    );
    badge_rect.right() + 4.0
}

fn draw_header_text(painter: &egui::Painter, context: &GridHeaderContentContext<'_>, layout: HeaderTextLayout) {
    let name_galley = painter.layout_no_wrap(
        context.column.name.clone(),
        DbProTheme::ui_medium_font(12.5),
        context.theme.text_primary,
    );
    let name_width = name_galley.size().x;
    painter.galley(
        Pos2::new(
            layout.text_x,
            layout.header_rect.center().y - name_galley.size().y * 0.5,
        ),
        name_galley,
        context.theme.text_primary,
    );
    let color = if context.sort_active {
        context.theme.accent
    } else {
        context.theme.text_muted
    };
    let type_galley = painter.layout_no_wrap(
        format!(" {}{}", context.column.data_type, context.sort_marker),
        FontId::monospace(10.5),
        color,
    );
    painter.galley(
        Pos2::new(
            layout.text_x + name_width + 4.0,
            layout.header_rect.center().y - type_galley.size().y * 0.5,
        ),
        type_galley,
        color,
    );
}
