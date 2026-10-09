//! Pure visual rendering for a result-grid column header.

use super::*;
use egui::{Color32, FontId, Pos2, Rect, CornerRadius, Stroke, Vec2};

pub(super) struct GridHeaderContentContext<'a> {
    pub(super) column: &'a crate::UiColumn,
    pub(super) col_rect: Rect,
    pub(super) is_primary_key: bool,
    pub(super) is_foreign_key: bool,
    /// `Some(true)` = descending, `Some(false)` = ascending, `None` = unsorted.
    pub(super) sort_desc: Option<bool>,
    /// Multi-sort order (1-based) shown next to the arrow.
    pub(super) sort_priority: Option<usize>,
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
    painter.rect_filled(badge_rect, CornerRadius::same(3.0 as u8), color.linear_multiply(0.18));
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
        FontId::monospace(11.5),
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
    let type_color = context.theme.text_tertiary;
    let type_galley = painter.layout_no_wrap(
        format!(" {}", context.column.data_type),
        FontId::monospace(10.0),
        type_color,
    );
    let type_x = layout.text_x + name_width + 4.0;
    let type_w = type_galley.size().x;
    painter.galley(
        Pos2::new(type_x, layout.header_rect.center().y - type_galley.size().y * 0.5),
        type_galley,
        type_color,
    );
    if let Some(desc) = context.sort_desc {
        // Sort glyph pins to the header's right edge (demo spec).
        let arrow_x = (layout.header_rect.right() - 9.0).max(type_x + type_w + 4.0);
        let next_x = draw_sort_arrow(painter, arrow_x, layout.header_rect, desc, context.theme.accent);
        if let Some(priority) = context.sort_priority {
            let prio = painter.layout_no_wrap(priority.to_string(), FontId::monospace(9.0), context.theme.accent);
            painter.galley(
                Pos2::new(next_x, layout.header_rect.center().y - prio.size().y * 0.5),
                prio,
                context.theme.accent,
            );
        }
    }
}

/// Tiny filled triangle — the `↑`/`↓` glyphs are missing from the bundled
/// fonts and render as `+`/tofu in headers.
// cc-scan:allow TOO_MANY_PARAMS — context params passed through
fn draw_sort_arrow(painter: &egui::Painter, x: f32, header_rect: Rect, desc: bool, color: Color32) -> f32 {
    let (w, h) = (7.0_f32, 4.5_f32);
    let cy = header_rect.center().y;
    let (edge_y, apex_y) = if desc {
        (cy - h * 0.5, cy + h * 0.5)
    } else {
        (cy + h * 0.5, cy - h * 0.5)
    };
    painter.add(egui::Shape::convex_polygon(
        vec![
            Pos2::new(x, edge_y),
            Pos2::new(x + w, edge_y),
            Pos2::new(x + w * 0.5, apex_y),
        ],
        color,
        Stroke::NONE,
    ));
    w + 4.0
}
