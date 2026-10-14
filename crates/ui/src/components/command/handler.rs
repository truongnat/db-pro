use egui::{Pos2, Rect};

use super::config::{
    INPUT_ICON_INSET_X, INPUT_TEXT_GAP, INPUT_TEXT_INSET_Y, INPUT_TEXT_RIGHT_INSET, ITEM_DEFAULT_TITLE_ALPHA,
    ITEM_HOVER_VISIBLE_THRESHOLD, ITEM_LEFT_INSET, ITEM_SUBTITLE_GAP, SHORTCUT_INSET_Y, SHORTCUT_REGION_WIDTH,
    SHORTCUT_RIGHT_INSET,
};
use crate::DbProTheme;

pub(super) fn command_input_icon_x(input_rect: Rect) -> f32 {
    input_rect.left() + INPUT_ICON_INSET_X
}

pub(super) fn command_input_text_rect(input_rect: Rect, icon_x: f32) -> Rect {
    Rect::from_min_max(
        Pos2::new(icon_x + INPUT_TEXT_GAP, input_rect.top() + INPUT_TEXT_INSET_Y),
        Pos2::new(
            input_rect.right() - INPUT_TEXT_RIGHT_INSET,
            input_rect.bottom() - INPUT_TEXT_INSET_Y,
        ),
    )
}

pub(super) fn item_highlight_target(hovered: bool, selected: bool, disabled: bool) -> bool {
    (hovered || selected) && !disabled
}

pub(super) fn item_is_actionable(disabled: bool) -> bool {
    !disabled
}

pub(super) fn item_has_background(selected: bool, disabled: bool, hover_amount: f32) -> bool {
    (selected && !disabled) || hover_amount > ITEM_HOVER_VISIBLE_THRESHOLD
}

pub(super) fn item_title_color(theme: &DbProTheme, disabled: bool, highlighted: bool) -> egui::Color32 {
    if disabled {
        return theme.text_disabled;
    }
    if highlighted {
        return theme.text_primary;
    }

    theme.text_primary.linear_multiply(ITEM_DEFAULT_TITLE_ALPHA)
}

pub(super) fn item_icon_color(theme: &DbProTheme, disabled: bool, highlighted: bool) -> egui::Color32 {
    if disabled {
        return theme.text_disabled;
    }
    if highlighted {
        return theme.text_primary;
    }

    theme.text_secondary
}

pub(super) fn subtitle_x(title_x: f32, title_width: f32, gap: f32) -> f32 {
    title_x + title_width + gap
}

pub(super) fn command_text_clip_rect(row_rect: Rect, text_left: f32, has_shortcut: bool) -> Rect {
    let text_right = if has_shortcut {
        shortcut_rect(row_rect).left() - ITEM_SUBTITLE_GAP
    } else {
        row_rect.right() - ITEM_LEFT_INSET
    };

    Rect::from_min_max(
        Pos2::new(text_left, row_rect.top()),
        Pos2::new(text_right.max(text_left), row_rect.bottom()),
    )
}

pub(super) fn accessible_item_label(title: &str, subtitle: Option<&str>, shortcut: Option<&str>) -> String {
    let mut label = title.to_owned();
    if let Some(subtitle) = subtitle {
        label.push_str(", ");
        label.push_str(subtitle);
    }
    if let Some(shortcut) = shortcut {
        label.push_str(", shortcut ");
        label.push_str(shortcut);
    }
    label
}

pub(super) fn shortcut_rect(row_rect: Rect) -> Rect {
    Rect::from_min_max(
        Pos2::new(
            row_rect.right() - SHORTCUT_REGION_WIDTH,
            row_rect.top() + SHORTCUT_INSET_Y,
        ),
        Pos2::new(
            row_rect.right() - SHORTCUT_RIGHT_INSET,
            row_rect.bottom() - SHORTCUT_INSET_Y,
        ),
    )
}

#[cfg(test)]
#[path = "tests/handler_tests.rs"]
mod tests;
