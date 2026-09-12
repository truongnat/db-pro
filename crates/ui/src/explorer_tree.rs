//! Shared rendering primitives for the Codex / DBeaver style navigator tree.
//!
//! Everything here is presentation-only: row geometry, the hover/selection wash,
//! the chevron slot, and the trailing badge/count/detail cluster. The tree row is
//! painted with a single allocated hit-box so later widgets are not displaced
//! (see the `allocate_exact_size` + `interact` note in `components.rs`).

use super::*;
use egui::{pos2, vec2, Align2, Color32, FontFamily, FontId, Rect, Rounding};
use lucide_icons::Icon;

/// Row height in pixels. Also the height of the chevron hit-box.
const CODEX_ROW_HEIGHT: f32 = 26.0;
/// Horizontal indent added per tree depth level.
const CODEX_ROW_INDENT: f32 = 10.0;
/// Width of the leading chevron slot.
const CODEX_CHEVRON_SLOT: f32 = 14.0;
/// Horizontal padding applied at both row edges.
const CODEX_ROW_PADDING: f32 = 8.0;
/// Gap inserted before each trailing item.
const CODEX_TRAILING_GAP: f32 = 5.0;

/// Shortens verbose database types into clean, compact identifiers (e.g. DBeaver style).
pub(super) fn shorten_data_type(data_type: &str) -> String {
    let lower = data_type.to_ascii_lowercase();
    let trimmed = lower.trim();
    if trimmed == "timestamp with time zone"
        || (trimmed.starts_with("timestamp(") && trimmed.contains("with time zone"))
    {
        "timestamptz".to_string()
    } else if trimmed == "timestamp without time zone"
        || (trimmed.starts_with("timestamp(") && trimmed.contains("without time zone"))
    {
        "timestamp".to_string()
    } else if trimmed == "time with time zone" {
        "timetz".to_string()
    } else if trimmed == "time without time zone" {
        "time".to_string()
    } else if trimmed.starts_with("character varying") {
        trimmed.replace("character varying", "varchar")
    } else if trimmed.starts_with("double precision") {
        "float8".to_string()
    } else if trimmed == "integer" {
        "int4".to_string()
    } else if trimmed == "bigint" {
        "int8".to_string()
    } else if trimmed == "smallint" {
        "int2".to_string()
    } else if trimmed == "boolean" {
        "bool".to_string()
    } else if data_type.len() > 14 {
        format!("{}…", &data_type[..13])
    } else {
        data_type.to_string()
    }
}

/// Properties for rendering an ultra-clean Codex-style tree row.
pub(super) struct CodexTreeRow<'a> {
    pub(super) depth: usize,
    pub(super) is_expandable: bool,
    pub(super) is_expanded: bool,
    pub(super) icon: Icon,
    pub(super) icon_color: Color32,
    pub(super) label: &'a str,
    pub(super) is_selected: bool,
    pub(super) is_dimmed: bool,
    pub(super) status_dot: Option<Color32>,
    pub(super) badge_text: Option<&'a str>,
    pub(super) badge_accent: bool,
    pub(super) count_text: Option<String>,
    pub(super) detail_text: Option<&'a str>,
}

/// Renders a single pixel-aligned, elegant tree row inspired by OpenAI Codex and modern developer tools.
///
/// Returns the row response plus whether the *chevron* (not the label) was clicked,
/// which callers use to distinguish "expand" from "activate".
pub(super) fn draw_codex_tree_row(
    ui: &mut egui::Ui,
    theme: &DbProTheme,
    row: CodexTreeRow<'_>,
) -> (egui::Response, bool) {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, CODEX_ROW_HEIGHT), egui::Sense::click());
    let is_hovered = response.hovered();

    let painter = ui.painter().with_clip_rect(rect);

    paint_row_background(&painter, rect, theme, row.is_selected, is_hovered);

    let center_y = rect.center().y;
    let mut curr_x = rect.min.x + CODEX_ROW_PADDING + (row.depth as f32) * CODEX_ROW_INDENT;

    // 1. Chevron slot
    let chevron_rect = Rect::from_min_size(pos2(curr_x, rect.min.y), vec2(CODEX_CHEVRON_SLOT, CODEX_ROW_HEIGHT));
    paint_chevron_slot(&painter, chevron_rect, theme, &row, is_hovered);
    curr_x += CODEX_CHEVRON_SLOT;

    // 2. Status dot (e.g. connection status)
    if let Some(dot_color) = row.status_dot {
        painter.circle_filled(pos2(curr_x + 3.0, center_y), 3.0, dot_color);
        curr_x += 10.0;
    }

    // 3. Node icon (Lucide vector icon)
    painter.text(
        pos2(curr_x + 7.0, center_y),
        Align2::CENTER_CENTER,
        char::from(row.icon).to_string(),
        FontId::new(13.0, FontFamily::Name("lucide".into())),
        row.icon_color,
    );
    curr_x += 17.0;

    // 4. Trailing cluster, then the clipped label in whatever gap remains.
    let right_x = paint_row_trailing_items(&painter, rect, center_y, theme, &row);
    paint_row_label(&painter, rect, center_y, curr_x, right_x, theme, &row);

    let chevron_clicked = response.clicked()
        && ui
            .input(|i| i.pointer.hover_pos())
            .is_some_and(|click_pos| chevron_rect.expand(2.0).contains(click_pos));

    (response, chevron_clicked)
}

/// Paints the row background: the selection wash plus its left-edge accent pill,
/// or a subtle hover wash, or nothing.
fn paint_row_background(painter: &egui::Painter, rect: Rect, theme: &DbProTheme, is_selected: bool, is_hovered: bool) {
    if is_selected {
        painter.rect_filled(rect, Rounding::same(4.0), theme.surface_active);
        // Signature Codex active pill indicator on left edge
        let pill_rect = Rect::from_min_max(
            pos2(rect.min.x + 1.0, rect.min.y + 4.0),
            pos2(rect.min.x + 3.5, rect.max.y - 4.0),
        );
        painter.rect_filled(pill_rect, Rounding::same(1.2), theme.accent);
    } else if is_hovered {
        painter.rect_filled(rect, Rounding::same(4.0), theme.surface_hover);
    }
}

/// Paints the expand/collapse chevron into its reserved slot. Non-expandable rows
/// still reserve the slot so labels stay aligned across sibling levels.
fn paint_chevron_slot(
    painter: &egui::Painter,
    chevron_rect: Rect,
    theme: &DbProTheme,
    row: &CodexTreeRow<'_>,
    is_hovered: bool,
) {
    if !row.is_expandable {
        return;
    }
    let chevron_icon = if row.is_expanded {
        Icon::ChevronDown
    } else {
        Icon::ChevronRight
    };
    let chevron_color = if is_hovered {
        theme.text_secondary
    } else {
        theme.text_muted
    };
    painter.text(
        chevron_rect.center(),
        Align2::CENTER_CENTER,
        char::from(chevron_icon).to_string(),
        FontId::new(10.5, FontFamily::Name("lucide".into())),
        chevron_color,
    );
}

/// Paints badge and count text from right to left.
/// Returns the left edge of the trailing cluster, used to clip the label.
fn paint_row_trailing_items(
    painter: &egui::Painter,
    rect: Rect,
    center_y: f32,
    theme: &DbProTheme,
    row: &CodexTreeRow<'_>,
) -> f32 {
    let mut right_x = rect.max.x - CODEX_ROW_PADDING;

    // Driver badge (e.g. PG / SQLITE)
    if let Some(badge) = row.badge_text {
        right_x -= paint_badge(painter, right_x, center_y, theme, badge, row.badge_accent);
    }

    // Count text (e.g. 68)
    if let Some(count) = row.count_text.as_deref() {
        let count_rect = painter.text(
            pos2(right_x, center_y),
            Align2::RIGHT_CENTER,
            count,
            FontId::proportional(11.0),
            theme.text_muted,
        );
        right_x -= count_rect.width() + CODEX_TRAILING_GAP;
    }

    right_x
}

/// Paints a small driver badge ending at `right_x`. Returns the width consumed
/// including the gap that follows it.
fn paint_badge(
    painter: &egui::Painter,
    right_x: f32,
    center_y: f32,
    theme: &DbProTheme,
    badge: &str,
    accent: bool,
) -> f32 {
    let badge_color = if accent { theme.accent } else { theme.text_muted };
    let badge_bg = if accent { theme.accent_soft } else { theme.surface_hover };
    let badge_w = (badge.len() as f32) * 6.5 + 8.0;
    let badge_h = 16.0;
    let badge_rect = Rect::from_min_size(
        pos2(right_x - badge_w, center_y - badge_h * 0.5),
        vec2(badge_w, badge_h),
    );
    painter.rect_filled(badge_rect, Rounding::same(3.0), badge_bg);
    painter.text(
        badge_rect.center(),
        Align2::CENTER_CENTER,
        badge,
        FontId::proportional(9.5),
        badge_color,
    );
    badge_w + CODEX_TRAILING_GAP
}

/// Paints the row label, followed by any inline detail text (e.g. data types),
/// cleanly clipped so it can never overrun the trailing cluster.
fn paint_row_label(
    painter: &egui::Painter,
    rect: Rect,
    center_y: f32,
    curr_x: f32,
    right_x: f32,
    theme: &DbProTheme,
    row: &CodexTreeRow<'_>,
) {
    let label_color = if row.is_selected {
        theme.accent
    } else if row.is_dimmed {
        theme.text_muted
    } else {
        theme.text_primary
    };

    let clip_max_x = (right_x - 4.0).max(curr_x);
    let clip_rect = Rect::from_min_max(pos2(curr_x, rect.min.y), pos2(clip_max_x, rect.max.y));
    let label_galley = painter.layout_no_wrap(row.label.to_string(), FontId::proportional(12.5), label_color);
    let label_w = label_galley.size().x;
    painter.with_clip_rect(clip_rect).galley(
        pos2(curr_x, center_y - label_galley.size().y * 0.5),
        label_galley,
        label_color,
    );

    // Detail text (e.g. data type: varchar, int4, timestamptz) painted inline right after label
    if let Some(detail) = row.detail_text {
        let detail_x = curr_x + label_w + 6.0;
        if detail_x < clip_max_x {
            let detail_galley = painter.layout_no_wrap(detail.to_string(), FontId::monospace(10.5), theme.text_muted);
            painter.with_clip_rect(clip_rect).galley(
                pos2(detail_x, center_y - detail_galley.size().y * 0.5),
                detail_galley,
                theme.text_muted,
            );
        }
    }
}

/// A dimmed, non-interactive hint row such as "No views in schema" or
/// "Disconnected — click to connect". Returns the row response so callers can
/// still make the hint clickable.
pub(super) fn draw_hint_row(
    ui: &mut egui::Ui,
    theme: &DbProTheme,
    depth: usize,
    icon: Icon,
    label: &str,
) -> egui::Response {
    let (response, _) = draw_codex_tree_row(
        ui,
        theme,
        CodexTreeRow {
            depth,
            is_expandable: false,
            is_expanded: false,
            icon,
            icon_color: theme.text_muted,
            label,
            is_selected: false,
            is_dimmed: true,
            status_dot: None,
            badge_text: None,
            badge_accent: false,
            count_text: None,
            detail_text: None,
        },
    );
    response.on_hover_text(label)
}

/// Header description for a navigator category folder.
pub(super) struct CategoryFolder<'a> {
    pub(super) depth: usize,
    pub(super) id: egui::Id,
    pub(super) icon: Icon,
    pub(super) icon_color: Color32,
    pub(super) label: &'a str,
    pub(super) count: usize,
    /// Hint shown when the folder is empty; `None` renders nothing.
    pub(super) empty_label: Option<&'a str>,
}

/// Renders a navigator category folder header (Views / Functions / Triggers / table
/// detail folders) with shared expand/collapse persistence. `body` renders the
/// children when the folder is open and non-empty.
pub(super) fn draw_category_folder(
    ui: &mut egui::Ui,
    theme: &DbProTheme,
    folder: CategoryFolder<'_>,
    body: impl FnOnce(&mut egui::Ui),
) {
    let mut collapsing = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), folder.id, false);
    let is_open = collapsing.is_open();

    let (response, chevron_clicked) = draw_codex_tree_row(
        ui,
        theme,
        CodexTreeRow {
            depth: folder.depth,
            is_expandable: true,
            is_expanded: is_open,
            icon: folder.icon,
            icon_color: folder.icon_color,
            label: folder.label,
            is_selected: false,
            is_dimmed: folder.count == 0,
            status_dot: None,
            badge_text: None,
            badge_accent: false,
            count_text: Some(folder.count.to_string()),
            detail_text: None,
        },
    );

    if response.clicked() || chevron_clicked {
        collapsing.set_open(!is_open);
        collapsing.store(ui.ctx());
    }

    if !collapsing.is_open() {
        return;
    }
    if folder.count > 0 {
        body(ui);
    } else if let Some(empty_label) = folder.empty_label {
        draw_hint_row(ui, theme, folder.depth + 1, Icon::Info, empty_label);
    }
}

/// Determines the best semantic Lucide icon and color for a column.
pub(super) fn column_icon_and_color(data_type: &str, is_pk: bool, is_fk: bool, theme: &DbProTheme) -> (Icon, Color32) {
    if is_pk {
        (Icon::Key, Color32::from_rgb(217, 119, 6)) // amber
    } else if is_fk {
        (Icon::Link, Color32::from_rgb(37, 99, 235)) // blue
    } else {
        let dt = data_type.to_ascii_lowercase();
        if dt.contains("int")
            || dt.contains("serial")
            || dt.contains("num")
            || dt.contains("dec")
            || dt.contains("float")
            || dt.contains("double")
        {
            (Icon::Hash, theme.text_muted)
        } else if dt.contains("char") || dt.contains("text") || dt.contains("uuid") {
            (Icon::Type, theme.text_muted)
        } else if dt.contains("date") || dt.contains("time") {
            (Icon::Calendar, theme.text_muted)
        } else if dt.contains("bool") {
            (Icon::ToggleLeft, theme.text_muted)
        } else {
            (Icon::Columns3, theme.text_muted)
        }
    }
}
