use crate::DbProTheme;
use egui::{Pos2, Rect};

use super::config::{
    INPUT_ICON_INSET_X, INPUT_TEXT_GAP, INPUT_TEXT_INSET_Y, INPUT_TEXT_RIGHT_INSET, ITEM_DEFAULT_TITLE_ALPHA,
    ITEM_HOVER_VISIBLE_THRESHOLD, ITEM_LEFT_INSET, ITEM_SUBTITLE_GAP, SHORTCUT_INSET_Y, SHORTCUT_REGION_WIDTH,
    SHORTCUT_RIGHT_INSET,
};

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
mod tests {
    use super::*;
    use egui::{Pos2, Rect, Vec2};

    #[test]
    fn disabled_items_cannot_trigger_hover_highlight_or_actions() {
        assert!(!item_is_actionable(true));
        assert!(!item_highlight_target(true, true, true));
        assert!(!item_highlight_target(false, false, true));
    }

    #[test]
    fn enabled_selection_and_hover_drive_the_highlight_target() {
        assert!(item_is_actionable(false));
        assert!(item_highlight_target(false, true, false));
        assert!(item_highlight_target(true, false, false));
        assert!(!item_highlight_target(false, false, false));
    }

    #[test]
    fn selected_rows_keep_the_background_and_hover_uses_a_threshold() {
        assert!(item_has_background(true, false, 0.0));
        assert!(!item_has_background(false, false, 0.001));
        assert!(item_has_background(false, false, 0.002));
        assert!(!item_has_background(true, true, 0.0));
    }

    #[test]
    fn title_and_icon_colors_match_disabled_and_highlight_states() {
        let theme = DbProTheme::light();

        assert_eq!(item_title_color(&theme, true, true), theme.text_disabled);
        assert_eq!(item_title_color(&theme, false, true), theme.text_primary);
        assert_eq!(
            item_title_color(&theme, false, false),
            theme.text_primary.linear_multiply(ITEM_DEFAULT_TITLE_ALPHA)
        );
        assert_eq!(item_icon_color(&theme, true, true), theme.text_disabled);
        assert_eq!(item_icon_color(&theme, false, true), theme.text_primary);
        assert_eq!(item_icon_color(&theme, false, false), theme.text_secondary);
    }

    #[test]
    fn input_and_shortcut_geometry_preserve_the_existing_insets() {
        let input = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(300.0, 44.0));
        let icon_x = command_input_icon_x(input);
        let text = command_input_text_rect(input, icon_x);
        let row = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(300.0, 36.0));
        let shortcut = shortcut_rect(row);
        let text_clip = command_text_clip_rect(row, 10.0, true);
        let narrow_row = Rect::from_min_size(Pos2::ZERO, Vec2::new(24.0, 36.0));
        let narrow_clip = command_text_clip_rect(narrow_row, 20.0, true);

        assert_eq!(icon_x, 24.0);
        assert_eq!(text.min, Pos2::new(48.0, 28.0));
        assert_eq!(text.max, Pos2::new(278.0, 56.0));
        assert_eq!(shortcut.min, Pos2::new(220.0, 6.0));
        assert_eq!(shortcut.max, Pos2::new(292.0, 30.0));
        assert_eq!(text_clip.max.x, 212.0);
        assert_eq!(narrow_clip.width(), 0.0);
        assert_eq!(subtitle_x(30.0, 80.0, 8.0), 118.0);
        assert_eq!(
            accessible_item_label("Open query", Some("Recent"), Some("⌘K")),
            "Open query, Recent, shortcut ⌘K"
        );
    }
}
