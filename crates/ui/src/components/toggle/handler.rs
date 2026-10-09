use super::config;
use crate::DbProTheme;
use egui::{Color32, CornerRadius, Stroke};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleVariant {
    Default,
    Outline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleSize {
    Sm,
    Default,
    Lg,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToggleSizeTokens {
    pub height: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub padding_x: f32,
}

impl ToggleSize {
    pub fn tokens(self) -> ToggleSizeTokens {
        match self {
            Self::Sm => ToggleSizeTokens {
                height: config::SM_HEIGHT,
                font_size: config::SM_FONT_SIZE,
                icon_size: config::SM_ICON_SIZE,
                padding_x: config::SM_PADDING_X,
            },
            Self::Default => ToggleSizeTokens {
                height: config::DEFAULT_HEIGHT,
                font_size: config::DEFAULT_FONT_SIZE,
                icon_size: config::DEFAULT_ICON_SIZE,
                padding_x: config::DEFAULT_PADDING_X,
            },
            Self::Lg => ToggleSizeTokens {
                height: config::LG_HEIGHT,
                font_size: config::LG_FONT_SIZE,
                icon_size: config::LG_ICON_SIZE,
                padding_x: config::LG_PADDING_X,
            },
        }
    }

    pub fn height(self) -> f32 {
        self.tokens().height
    }

    pub fn font_size(self) -> f32 {
        self.tokens().font_size
    }

    pub fn icon_size(self) -> f32 {
        self.tokens().icon_size
    }

    pub fn padding_x(self) -> f32 {
        self.tokens().padding_x
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToggleAppearance {
    pub fill: Color32,
    pub stroke: Stroke,
    pub text_color: Color32,
}

/// Calculates the intrinsic content width before horizontal padding is added.
/// The UI layer supplies the measured label width because only egui can create a galley.
pub fn content_width(icon_present: bool, label_width: Option<f32>, icon_size: f32) -> f32 {
    let icon_width = if icon_present { icon_size } else { 0.0 };
    let icon_label_gap = if icon_present && label_width.is_some() {
        config::ICON_TEXT_GAP
    } else {
        0.0
    };
    icon_width + icon_label_gap + label_width.unwrap_or(0.0)
}

/// Applies the existing minimum-height rule to an intrinsic toggle width.
pub fn control_width(content_width: f32, padding_x: f32, height: f32) -> f32 {
    (content_width + padding_x * config::HORIZONTAL_PADDING_FACTOR).max(height)
}

/// Applies an enabled click to the caller-owned pressed state.
///
/// Returns `true` only when the click changes state, allowing the UI layer to mark the
/// egui response as changed without duplicating interaction rules.
pub fn apply_toggle_click(pressed: &mut bool, clicked: bool, enabled: bool) -> bool {
    if !enabled || !clicked {
        return false;
    }

    *pressed = toggled_state(*pressed);
    true
}

/// Returns the next pressed state after an enabled click.
pub fn toggled_state(pressed: bool) -> bool {
    !pressed
}

/// Resolves standalone toggle colors from semantic state and the shared theme.
pub fn standalone_appearance(
    variant: ToggleVariant,
    pressed: bool,
    enabled: bool,
    hover: f32,
    theme: DbProTheme,
) -> ToggleAppearance {
    match variant {
        ToggleVariant::Default => {
            if pressed {
                return ToggleAppearance {
                    fill: theme.accent,
                    stroke: Stroke::NONE,
                    text_color: theme.accent_foreground,
                };
            }

            let fill = if hover > config::HOVER_THRESHOLD {
                theme.surface_hover.linear_multiply(hover)
            } else {
                Color32::TRANSPARENT
            };
            let text_color = if enabled {
                theme.text_secondary
            } else {
                theme.text_disabled
            };
            ToggleAppearance {
                fill,
                stroke: Stroke::NONE,
                text_color,
            }
        }
        ToggleVariant::Outline => {
            if pressed {
                return ToggleAppearance {
                    fill: theme.surface_hover,
                    stroke: Stroke::new(config::OUTLINE_STROKE_WIDTH, theme.border_strong),
                    text_color: theme.text_primary,
                };
            }

            let stroke_color = if hover > config::HOVER_THRESHOLD {
                theme.border_strong
            } else {
                theme.border_default
            };
            let fill = if hover > config::HOVER_THRESHOLD {
                theme.surface_hover.linear_multiply(hover * config::OUTLINE_FILL_FACTOR)
            } else {
                Color32::TRANSPARENT
            };
            let text_color = if enabled {
                theme.text_secondary
            } else {
                theme.text_disabled
            };
            ToggleAppearance {
                fill,
                stroke: Stroke::new(config::OUTLINE_STROKE_WIDTH, stroke_color),
                text_color,
            }
        }
    }
}

/// Calculates the outer corner radii for an item in a contiguous single-select group.
pub fn group_rounding(index: usize, count: usize) -> CornerRadius {
    if count <= 1 {
        return CornerRadius::same(config::TOGGLE_ROUNDING as u8);
    }
    if index == 0 {
        return CornerRadius {
            nw: (config::TOGGLE_ROUNDING) as u8,
            sw: (config::TOGGLE_ROUNDING) as u8,
            ne: 0.0 as u8,
            se: 0.0 as u8,
        };
    }
    if index + 1 == count {
        return CornerRadius {
            nw: 0.0 as u8,
            sw: 0.0 as u8,
            ne: (config::TOGGLE_ROUNDING) as u8,
            se: (config::TOGGLE_ROUNDING) as u8,
        };
    }
    CornerRadius::ZERO
}

/// Resolves the fill for a group item without mutating selection state.
pub fn group_fill(selected: bool, hover: f32, theme: DbProTheme) -> Color32 {
    if selected {
        return theme.accent;
    }
    if hover > config::HOVER_THRESHOLD {
        return theme.surface_hover.linear_multiply(hover);
    }
    theme.surface_panel
}

/// Resolves group text color from selection and the egui hover signal.
pub fn group_text_color(selected: bool, hovered: bool, theme: DbProTheme) -> Color32 {
    if selected {
        theme.accent_foreground
    } else if hovered {
        theme.text_primary
    } else {
        theme.text_secondary
    }
}

/// Returns a cloned candidate only when a click changes the selected value.
pub fn selection_after_click<T: Clone + PartialEq>(selected: &T, candidate: &T) -> Option<T> {
    if selected == candidate {
        None
    } else {
        Some(candidate.clone())
    }
}

/// Applies a clicked group item to caller-owned selection state.
///
/// The active item is intentionally inert. Returning the cloned value lets the UI layer
/// expose the existing `show_single` change result while this handler owns the transition.
pub fn apply_selection_click<T: Clone + PartialEq>(selected: &mut T, candidate: &T, clicked: bool) -> Option<T> {
    if !clicked {
        return None;
    }

    let next = selection_after_click(selected, candidate)?;
    *selected = next.clone();
    Some(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_tokens_preserve_public_dimensions() {
        assert_eq!(ToggleSize::Sm.height(), config::SM_HEIGHT);
        assert_eq!(ToggleSize::Default.font_size(), config::DEFAULT_FONT_SIZE);
        assert_eq!(ToggleSize::Lg.icon_size(), config::LG_ICON_SIZE);
        assert_eq!(ToggleSize::Default.padding_x(), config::DEFAULT_PADDING_X);
    }

    #[test]
    fn content_width_accounts_for_icon_label_gap_only_when_needed() {
        assert_eq!(content_width(false, None, 14.0), 0.0);
        assert_eq!(content_width(true, None, 14.0), 14.0);
        assert_eq!(
            content_width(true, Some(30.0), 14.0),
            14.0 + config::ICON_TEXT_GAP + 30.0
        );
    }

    #[test]
    fn control_width_keeps_the_size_minimum() {
        assert_eq!(control_width(4.0, 11.0, 32.0), 32.0);
        assert_eq!(control_width(30.0, 11.0, 32.0), 52.0);
    }

    #[test]
    fn selection_and_pressed_transitions_are_stable() {
        assert!(toggled_state(false));
        assert!(!toggled_state(true));

        let mut pressed = false;
        assert!(apply_toggle_click(&mut pressed, true, true));
        assert!(pressed);
        assert!(!apply_toggle_click(&mut pressed, false, true));
        assert!(!apply_toggle_click(&mut pressed, true, false));

        assert_eq!(selection_after_click(&1, &1), None);
        assert_eq!(selection_after_click(&1, &2), Some(2));

        let mut selected = 1;
        assert_eq!(apply_selection_click(&mut selected, &1, true), None);
        assert_eq!(apply_selection_click(&mut selected, &2, true), Some(2));
        assert_eq!(selected, 2);
        assert_eq!(apply_selection_click(&mut selected, &3, false), None);
    }

    #[test]
    fn group_rounding_only_rounds_the_outer_edges() {
        assert_eq!(group_rounding(0, 1), CornerRadius::same(config::TOGGLE_ROUNDING as u8));
        assert_eq!(group_rounding(0, 3).ne, 0);
        assert_eq!(group_rounding(1, 3), CornerRadius::ZERO);
        assert_eq!(group_rounding(2, 3).se, config::TOGGLE_ROUNDING as u8);
    }

    #[test]
    fn appearance_preserves_outline_and_disabled_states() {
        let theme = DbProTheme::dark();
        let outline = standalone_appearance(ToggleVariant::Outline, false, true, 0.0, theme);
        assert_eq!(
            outline.stroke,
            Stroke::new(config::OUTLINE_STROKE_WIDTH, theme.border_default)
        );
        let disabled = standalone_appearance(ToggleVariant::Default, false, false, 0.0, theme);
        assert_eq!(disabled.text_color, theme.text_disabled);
    }
}
