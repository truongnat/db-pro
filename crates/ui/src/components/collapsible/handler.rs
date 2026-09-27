use super::config::{BADGE_HEIGHT, BADGE_PAD_H, BADGE_RIGHT_MARGIN, CHEVRON_OPEN_THRESHOLD};
use crate::DbProTheme;
use egui::{Color32, Pos2, Rect, Vec2};
use lucide_icons::Icon;

/// Toggles the collapsible state if clicked and not disabled.
pub fn apply_header_click(open: &mut bool, clicked: bool, disabled: bool) {
    if clicked && !disabled {
        *open = !*open;
    }
}

/// Determines the chevron icon orientation based on animation progress.
pub fn chevron_icon(open_anim_t: f32) -> Icon {
    if open_anim_t > CHEVRON_OPEN_THRESHOLD {
        Icon::ChevronDown
    } else {
        Icon::ChevronRight
    }
}

/// Resolves semantic colors for header chevron/icon and title text.
pub fn resolve_header_colors(theme: &DbProTheme, disabled: bool, hovered: bool) -> (Color32, Color32) {
    let icon_color = if disabled {
        theme.text_disabled
    } else {
        theme.text_secondary
    };

    let title_color = if disabled {
        theme.text_disabled
    } else if hovered {
        theme.text_primary
    } else {
        theme.text_secondary
    };

    (icon_color, title_color)
}

/// Computes the layout bounding rectangle for the trailing badge pill.
pub fn calculate_badge_rect(header_rect: Rect, center_y: f32, badge_text_width: f32) -> Rect {
    let available_width = (header_rect.width() - BADGE_RIGHT_MARGIN).max(0.0);
    let badge_width = (badge_text_width + BADGE_PAD_H).clamp(0.0, available_width);
    let right_edge = (header_rect.right() - BADGE_RIGHT_MARGIN).max(header_rect.left());
    Rect::from_center_size(
        Pos2::new(right_edge - badge_width * 0.5, center_y),
        Vec2::new(badge_width, BADGE_HEIGHT),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_click_toggles_state_when_enabled() {
        let mut open = false;
        apply_header_click(&mut open, true, false);
        assert!(open);

        apply_header_click(&mut open, true, false);
        assert!(!open);
    }

    #[test]
    fn header_click_ignored_when_disabled_or_not_clicked() {
        let mut open = false;
        apply_header_click(&mut open, true, true);
        assert!(!open, "Click must be ignored when disabled");

        apply_header_click(&mut open, false, false);
        assert!(!open, "No toggle when not clicked");
    }

    #[test]
    fn chevron_icon_flips_at_threshold() {
        assert_eq!(char::from(chevron_icon(0.0)), char::from(Icon::ChevronRight));
        assert_eq!(char::from(chevron_icon(0.5)), char::from(Icon::ChevronRight));
        assert_eq!(char::from(chevron_icon(0.51)), char::from(Icon::ChevronDown));
        assert_eq!(char::from(chevron_icon(1.0)), char::from(Icon::ChevronDown));
    }

    #[test]
    fn resolve_header_colors_respects_disabled_and_hover_states() {
        let theme = DbProTheme::light();

        // Disabled state: both colors are text_disabled
        let (icon_col, title_col) = resolve_header_colors(&theme, true, false);
        assert_eq!(icon_col, theme.text_disabled);
        assert_eq!(title_col, theme.text_disabled);

        let (icon_col_hov, title_col_hov) = resolve_header_colors(&theme, true, true);
        assert_eq!(icon_col_hov, theme.text_disabled);
        assert_eq!(title_col_hov, theme.text_disabled);

        // Enabled idle state: secondary colors
        let (icon_col, title_col) = resolve_header_colors(&theme, false, false);
        assert_eq!(icon_col, theme.text_secondary);
        assert_eq!(title_col, theme.text_secondary);

        // Enabled hovered state: title highlights to primary
        let (icon_col, title_col) = resolve_header_colors(&theme, false, true);
        assert_eq!(icon_col, theme.text_secondary);
        assert_eq!(title_col, theme.text_primary);
    }

    #[test]
    fn calculate_badge_rect_matches_expected_dimensions() {
        let header = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(200.0, 36.0));
        let badge_rect = calculate_badge_rect(header, 18.0, 30.0);

        assert_eq!(badge_rect.height(), BADGE_HEIGHT);
        assert_eq!(badge_rect.width(), 40.0);
        assert_eq!(badge_rect.center().y, 18.0);
        // badge right edge should be at header.right() - BADGE_RIGHT_MARGIN
        assert_eq!(badge_rect.right(), 200.0 - BADGE_RIGHT_MARGIN);
    }

    #[test]
    fn badge_rect_stays_inside_narrow_header() {
        let header = Rect::from_min_size(Pos2::new(10.0, 0.0), Vec2::new(12.0, 36.0));
        let badge_rect = calculate_badge_rect(header, 18.0, 40.0);
        assert!(badge_rect.left() >= header.left());
        assert!(badge_rect.right() <= header.right());
    }
}
