use egui::{Color32, FontId, Margin, Pos2, Rect, Ui};

use super::config::{
    DOUBLE_FACTOR, MAX_VISIBLE_ITEMS, MENU_MIN_SCREEN_WIDTH, MIN_TRIGGER_TEXT_WIDTH, MIN_TRIGGER_WIDTH,
    OPTION_ICON_SIDE, TRIGGER_TEXT_RESERVED_WIDTH, VERTICAL_CENTER_FACTOR,
};
use crate::tokens::{
    component::table::TABLE_ROW_HEIGHT_COMPACT, FONT_SIZE_UI_LABEL, ICON_LG, ICON_TEXT_GAP, SPACE_SM, SPACE_XS,
};

pub fn dropdown_should_open_above(space_below: f32, space_above: f32, menu_h: f32) -> bool {
    space_below < menu_h && space_above > space_below
}

pub struct DropdownGeometry {
    pub menu_pos: Pos2,
    pub menu_width: f32,
    pub max_height: f32,
}

/// Widest popup that still fits the screen; anything measured beyond this is
/// clamped away by [`calculate_menu_geometry`], so it also bounds label scanning.
pub fn menu_max_width(screen: Rect) -> f32 {
    (screen.width() - SPACE_SM * DOUBLE_FACTOR).max(MENU_MIN_SCREEN_WIDTH)
}

pub fn calculate_menu_geometry(
    screen: Rect,
    parent_rect: Rect,
    option_count: usize,
    has_more: bool,
    content_width: f32,
) -> DropdownGeometry {
    // Count real options plus the optional action row, saturate overflow, then cap the visible row count.
    let extra = usize::from(has_more);
    let visible = option_count.saturating_add(extra).min(MAX_VISIBLE_ITEMS);
    let menu_h = visible as f32 * TABLE_ROW_HEIGHT_COMPACT + SPACE_SM * DOUBLE_FACTOR;
    // Measure nonnegative space from the trigger to each screen edge before choosing a direction.
    let space_below = (screen.bottom() - parent_rect.bottom()).max(0.0);
    let space_above = (parent_rect.top() - screen.top()).max(0.0);
    // Flip upward only if the menu does not fit below and above has more room; otherwise keep downward placement.
    let open_up = dropdown_should_open_above(space_below, space_above, menu_h);
    // Clamp width in order: at least the trigger or the widest option row (so no label
    // ellipsizes), at most usable screen width (with minimum screen fallback).
    let menu_width = parent_rect
        .width()
        .max(content_width)
        .max(MIN_TRIGGER_WIDTH)
        .min(menu_max_width(screen));
    let menu_left = parent_rect.left().clamp(
        screen.left() + SPACE_SM,
        (screen.right() - menu_width - SPACE_SM).max(screen.left() + SPACE_SM),
    );
    let menu_pos = if open_up {
        Pos2::new(menu_left, parent_rect.top() - SPACE_XS - menu_h)
    } else {
        Pos2::new(menu_left, parent_rect.bottom() + SPACE_XS)
    };

    DropdownGeometry {
        menu_pos,
        menu_width,
        max_height: MAX_VISIBLE_ITEMS as f32 * TABLE_ROW_HEIGHT_COMPACT,
    }
}

pub fn selected_label(options: &[String], selected: usize) -> &str {
    options
        .get(selected)
        .map(String::as_str)
        .unwrap_or("Select an option...")
}

pub fn trigger_accessibility_label(label: Option<&str>, selected: &str) -> String {
    label.map_or_else(|| selected.to_owned(), |label| format!("{label}: {selected}"))
}

pub fn trigger_content_width(width: f32, available: f32) -> f32 {
    width.max(MIN_TRIGGER_WIDTH).min(available).max(0.0)
}

/// Width the trigger needs to show `label_width` without ellipsis: the label plus
/// the reserved chevron column and the frame's own padding and stroke.
pub fn trigger_preferred_width(label_width: f32, margin: Margin, stroke_extra: f32) -> f32 {
    label_width + TRIGGER_TEXT_RESERVED_WIDTH + margin.leftf() + margin.rightf() + stroke_extra
}

/// Width a menu row needs to show its label without ellipsizing: symmetric button
/// padding, the optional leading icon column, and the trailing check/`soon`
/// indicator column that `paint_option` reserves for the selected row.
pub fn menu_row_width(label_width: f32, has_icon: bool, selected: bool, button_padding_x: f32) -> f32 {
    let icon = if has_icon { OPTION_ICON_SIDE + SPACE_XS } else { 0.0 };
    let indicator = if selected { SPACE_SM + ICON_LG } else { 0.0 };
    button_padding_x * DOUBLE_FACTOR + icon + label_width + indicator
}

pub fn trigger_text_width(available: f32) -> f32 {
    (available - TRIGGER_TEXT_RESERVED_WIDTH).max(MIN_TRIGGER_TEXT_WIDTH)
}

/// Widest one-line label at `font_id`, stopping as soon as a label reaches `cap`.
/// Callers clamp the result to `cap` anyway, so measuring further cannot change the
/// layout — this keeps the scan cheap for long option lists. Measurement only: the
/// text is never painted, so the colour argument is a placeholder.
pub fn widest_label_width(ui: &Ui, labels: &[&str], font_id: &FontId, cap: f32) -> f32 {
    let mut widest: f32 = 0.0;
    for label in labels {
        let width = ui
            .painter()
            .layout_no_wrap((*label).to_owned(), font_id.clone(), Color32::WHITE)
            .size()
            .x;
        widest = widest.max(width);
        if widest >= cap {
            return cap;
        }
    }
    widest
}

pub fn trigger_inner_margin(padding_x: f32, padding_y: f32) -> Margin {
    Margin::symmetric(padding_x as i8, (padding_y + crate::tokens::SPACE_XXS) as i8)
}

pub fn menu_surface_margin(menu_margin_left: f32) -> Margin {
    Margin::symmetric(menu_margin_left as i8, SPACE_SM as i8)
}

pub fn menu_min_content_width(menu_width: f32) -> f32 {
    (menu_width - ICON_TEXT_GAP).max(0.0)
}

pub fn option_text_rect(rect: Rect, button_padding_x: f32, selected: bool) -> Rect {
    // Reserve trailing space only for selected rows, preventing text from colliding with the checkmark.
    let indicator_offset = if selected { SPACE_SM + ICON_LG } else { 0.0 };
    let text_left = rect.left() + button_padding_x;
    let text_right = rect.right() - button_padding_x - indicator_offset;
    Rect::from_min_max(Pos2::new(text_left, rect.top()), Pos2::new(text_right, rect.bottom()))
}

/// Lay a menu row label out as one truncated line. The row height is fixed, so
/// wrapping to a second line would overflow into the next option.
pub fn option_label_job(label: String, color: Color32, max_width: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::single_section(
        label,
        egui::TextFormat {
            font_id: FontId::proportional(FONT_SIZE_UI_LABEL),
            color,
            ..Default::default()
        },
    );
    job.wrap = egui::text::TextWrapping {
        max_width,
        max_rows: 1,
        break_anywhere: true,
        overflow_character: Some('…'),
    };
    job
}

pub fn option_galley_pos(label_left: f32, vertical_center: f32, galley_height: f32) -> Pos2 {
    Pos2::new(label_left, vertical_center - galley_height * VERTICAL_CENTER_FACTOR)
}

pub fn option_check_icon_pos(rect_right: f32, vertical_center: f32) -> Pos2 {
    Pos2::new(rect_right - SPACE_SM, vertical_center)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavigationKeys {
    pub arrow_down: bool,
    pub arrow_up: bool,
}

pub fn navigate_selection(selected: usize, option_count: usize, keys: NavigationKeys) -> usize {
    // Clamp first: a stale index (options shrank while the menu stayed open)
    // must land on the last valid option before arrows apply. Empty lists
    // clamp to 0; render paths read via `options.get`, so 0 is inert.
    let selected = selected.min(option_count.saturating_sub(1));
    // Clamp movement at both ends; the optional load-more action is deliberately outside option_count.
    let selected = if keys.arrow_down && selected.saturating_add(1) < option_count {
        selected + 1
    } else {
        selected
    };

    if keys.arrow_up && selected > 0 {
        selected - 1
    } else {
        selected
    }
}

/// What an open menu does with this frame's activation keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuKeyAction {
    None,
    Commit,
    Cancel,
}

pub fn menu_key_action(escape_pressed: bool, enter_pressed: bool) -> MenuKeyAction {
    if enter_pressed {
        MenuKeyAction::Commit
    } else if escape_pressed {
        MenuKeyAction::Cancel
    } else {
        MenuKeyAction::None
    }
}

pub fn selection_from_click(clicked: bool, option_index: usize) -> Option<usize> {
    clicked.then_some(option_index)
}

pub fn should_request_load_more(clicked: bool, has_more: bool) -> bool {
    clicked && has_more
}

#[cfg(test)]
#[path = "tests/handler_tests.rs"]
mod tests;
