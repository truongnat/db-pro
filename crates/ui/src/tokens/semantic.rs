//! Semantic layer — purpose-based roles for the native shell.
//!
//! Layer 2 of `Primitive → Semantic → Component`. Every role has an explicit light
//! and dark treatment built from the primitive palettes, so a theme switch is a
//! remapping of roles, not a search-and-replace of raw values. `DbProTheme` is the
//! flat compatibility facade over this layer: existing callers keep reading
//! `theme.surface_panel`, while new code may read full role detail through
//! `DbProTheme::semantic`.

use egui::Color32;

use super::primitive::{dark, light};

/// Status fill wash applied to plain surfaces (alerts, banners): `solid × this`.
pub const STATUS_FILL_OPACITY: f32 = 0.12;
/// Status border around that wash: `solid × this`.
pub const STATUS_BORDER_OPACITY: f32 = 0.40;

/// Soft wash recipe behind `DbProTheme::soft_tint` and the `*_soft()` status fills
/// (badges, diff rows, editor chips). Dark keeps a slightly stronger tint so the
/// wash stays visible against charcoal surfaces; light stays at ~8% over alpha 18.
pub fn subtle_wash(color: Color32, dark: bool) -> Color32 {
    if dark {
        Color32::from_rgba_premultiplied(
            (color.r() as f32 * 0.12) as u8,
            (color.g() as f32 * 0.12) as u8,
            (color.b() as f32 * 0.12) as u8,
            25,
        )
    } else {
        Color32::from_rgba_premultiplied(
            (color.r() as f32 * 0.08) as u8,
            (color.g() as f32 * 0.08) as u8,
            (color.b() as f32 * 0.08) as u8,
            18,
        )
    }
}

/// Surface roles: where a pixel lives before anything is drawn on it.
#[derive(Debug, Clone, Copy)]
pub struct Background {
    /// App canvas behind every panel.
    pub canvas: Color32,
    /// Persistent chrome: sidebars, toolbars, status bar.
    pub panel: Color32,
    /// Raised in-flow surfaces: cards, empty states.
    pub elevated: Color32,
    /// Transient surfaces: dialogs, popovers, menus.
    pub floating: Color32,
    /// SQL editor / result grid background.
    pub editor: Color32,
    /// Pointer hover wash on interactive rows and controls.
    pub hover: Color32,
    /// Active/pressed widget and selected-row wash.
    pub selected: Color32,
    /// Quiet fill for disabled controls and card icon boxes.
    /// `DbProTheme::surface_2` is the compatibility alias of this role.
    pub subtle: Color32,
    /// Modal backdrop dimmed behind dialogs and sheets.
    pub overlay: Color32,
}

/// Text and glyph roles.
#[derive(Debug, Clone, Copy)]
pub struct Foreground {
    pub primary: Color32,
    pub secondary: Color32,
    pub tertiary: Color32,
    /// Hints, status bars, deprecated copy. Coincides with `tertiary` in light,
    /// but is its own role: the dark palette deliberately lightens it.
    pub muted: Color32,
    pub disabled: Color32,
    /// Text on the inverse (strong) surface.
    pub inverse: Color32,
    /// Text/icons on top of `accent.solid`.
    pub on_accent: Color32,
}

/// Separation roles between surfaces and around controls.
#[derive(Debug, Clone, Copy)]
pub struct Border {
    pub subtle: Color32,
    pub default: Color32,
    pub strong: Color32,
    /// Focus ring color. Both themes draw the ring in the accent color; the role
    /// exists so focus can diverge from accent without touching every consumer.
    pub focus: Color32,
    /// Intra-surface separators (list dividers, table gutters).
    /// Value matches `default` today; kept as its own role for future tuning.
    pub separator: Color32,
}

/// Action accent roles.
#[derive(Debug, Clone, Copy)]
pub struct Accent {
    /// Tinted accent wash (completion selection, selection bg).
    pub subtle: Color32,
    pub solid: Color32,
    pub solid_hover: Color32,
    /// Foreground on `solid`.
    pub foreground: Color32,
}

/// One shipped status color with everything a component needs to render it
/// intentionally instead of multiplying raw values at the paint site.
#[derive(Debug, Clone, Copy)]
pub struct StatusRole {
    /// Solid color: icons, strong text, solid fills.
    pub solid: Color32,
    /// Text/icon color on the `subtle` wash; intentionally equals `solid` in the
    /// shipped palettes and kept as its own role so on-status text can diverge.
    pub foreground: Color32,
    /// `solid × STATUS_FILL_OPACITY` — alert/banner fill on plain surfaces.
    pub fill: Color32,
    /// `solid × STATUS_BORDER_OPACITY` — border around `fill`.
    pub border: Color32,
    /// `subtle_wash(solid)` — badge/diff/chip wash.
    pub subtle: Color32,
}

/// Status roles. Each theme ships all four.
#[derive(Debug, Clone, Copy)]
pub struct Status {
    pub success: StatusRole,
    pub warning: StatusRole,
    pub danger: StatusRole,
    pub info: StatusRole,
}

/// SQL syntax colors (editor-only; not interactive UI roles).
#[derive(Debug, Clone, Copy)]
pub struct Syntax {
    pub keyword: Color32,
    pub string: Color32,
    pub number: Color32,
    pub comment: Color32,
    pub type_: Color32,
    pub function: Color32,
    pub operator: Color32,
    pub punctuation: Color32,
    pub variable: Color32,
}

/// Full role set for one theme. Built once per theme construction; `DbProTheme`
/// flattens it into its existing public fields for compatibility.
#[derive(Debug, Clone, Copy)]
pub struct SemanticTokens {
    pub background: Background,
    pub foreground: Foreground,
    pub border: Border,
    pub accent: Accent,
    pub status: Status,
    pub syntax: Syntax,
}

impl StatusRole {
    fn from_solid(solid: Color32, dark: bool) -> Self {
        Self {
            solid,
            foreground: solid,
            fill: solid.linear_multiply(STATUS_FILL_OPACITY),
            border: solid.linear_multiply(STATUS_BORDER_OPACITY),
            subtle: subtle_wash(solid, dark),
        }
    }
}

impl SemanticTokens {
    /// Codex-aligned light roles.
    // cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
    pub fn light() -> Self {
        Self {
            background: Background {
                canvas: light::NEUTRAL_0,
                panel: light::NEUTRAL_50,
                elevated: light::NEUTRAL_0,
                floating: light::NEUTRAL_0,
                editor: light::NEUTRAL_0,
                hover: light::NEUTRAL_100,
                selected: light::NEUTRAL_125,
                subtle: light::NEUTRAL_75,
                overlay: light::SCRIM,
            },
            foreground: Foreground {
                primary: light::NEUTRAL_950,
                secondary: light::NEUTRAL_600,
                tertiary: light::NEUTRAL_500,
                muted: light::NEUTRAL_500,
                disabled: light::NEUTRAL_450,
                inverse: light::NEUTRAL_0,
                on_accent: light::NEUTRAL_0,
            },
            border: Border {
                subtle: light::NEUTRAL_100,
                default: light::NEUTRAL_150,
                strong: light::NEUTRAL_175,
                focus: light::BLUE_500,
                separator: light::NEUTRAL_150,
            },
            accent: Accent {
                subtle: light::BLUE_50,
                solid: light::BLUE_500,
                solid_hover: light::BLUE_600,
                foreground: light::NEUTRAL_0,
            },
            status: Status {
                success: StatusRole::from_solid(light::GREEN_600, false),
                warning: StatusRole::from_solid(light::AMBER_600, false),
                danger: StatusRole::from_solid(light::RED_600, false),
                info: StatusRole::from_solid(light::BLUE_700, false),
            },
            syntax: Syntax {
                keyword: light::INDIGO_700,
                string: light::GREEN_700,
                number: light::AMBER_700,
                comment: light::NEUTRAL_550,
                type_: light::PURPLE_700,
                function: light::CYAN_700,
                operator: light::NEUTRAL_575,
                punctuation: light::NEUTRAL_600,
                variable: light::NEUTRAL_900,
            },
        }
    }

    /// Codex-aligned dark roles tuned for dense database workspaces.
    // cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
    pub fn dark() -> Self {
        Self {
            background: Background {
                canvas: dark::NEUTRAL_850,
                panel: dark::NEUTRAL_810,
                elevated: dark::NEUTRAL_780,
                floating: dark::NEUTRAL_790,
                editor: dark::NEUTRAL_860,
                hover: dark::NEUTRAL_750,
                selected: dark::NEUTRAL_725,
                subtle: dark::NEUTRAL_775,
                overlay: dark::SCRIM,
            },
            foreground: Foreground {
                primary: dark::NEUTRAL_50,
                secondary: dark::NEUTRAL_300,
                tertiary: dark::NEUTRAL_475,
                muted: dark::NEUTRAL_450,
                disabled: dark::NEUTRAL_500,
                inverse: dark::NEUTRAL_900,
                on_accent: dark::NEUTRAL_0,
            },
            border: Border {
                subtle: dark::NEUTRAL_740,
                default: dark::NEUTRAL_700,
                strong: dark::NEUTRAL_650,
                focus: dark::BLUE_500,
                separator: dark::NEUTRAL_700,
            },
            accent: Accent {
                subtle: dark::BLUE_900,
                solid: dark::BLUE_500,
                solid_hover: dark::BLUE_400,
                foreground: dark::NEUTRAL_900,
            },
            status: Status {
                success: StatusRole::from_solid(dark::GREEN_500, true),
                warning: StatusRole::from_solid(dark::AMBER_400, true),
                danger: StatusRole::from_solid(dark::RED_400, true),
                info: StatusRole::from_solid(dark::BLUE_450, true),
            },
            syntax: Syntax {
                keyword: dark::PURPLE_400,
                string: dark::GREEN_400,
                number: dark::ORANGE_400,
                comment: dark::SLATE_500,
                type_: dark::AMBER_300,
                function: dark::BLUE_300,
                operator: dark::CYAN_300,
                punctuation: dark::CYAN_300,
                variable: dark::NEUTRAL_25,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_group_has_light_and_dark_treatment() {
        let light = SemanticTokens::light();
        let dark = SemanticTokens::dark();

        assert_ne!(light.background.canvas, dark.background.canvas);
        assert_ne!(light.background.subtle, dark.background.subtle);
        assert_ne!(light.foreground.primary, dark.foreground.primary);
        assert_ne!(light.foreground.muted, dark.foreground.muted);
        assert_ne!(light.border.default, dark.border.default);
        assert_ne!(light.border.focus, dark.border.focus);
        assert_ne!(light.accent.solid, dark.accent.solid);
        assert_ne!(light.status.success.solid, dark.status.success.solid);
        assert_ne!(light.status.danger.fill, dark.status.danger.fill);
        assert_ne!(light.syntax.keyword, dark.syntax.keyword);
    }

    #[test]
    fn status_roles_carry_fill_border_subtle_and_foreground() {
        for theme_dark in [false, true] {
            let tokens = if theme_dark {
                SemanticTokens::dark()
            } else {
                SemanticTokens::light()
            };
            for role in [
                tokens.status.success,
                tokens.status.warning,
                tokens.status.danger,
                tokens.status.info,
            ] {
                assert_eq!(role.fill, role.solid.linear_multiply(STATUS_FILL_OPACITY));
                assert_eq!(role.border, role.solid.linear_multiply(STATUS_BORDER_OPACITY));
                assert_eq!(role.subtle, subtle_wash(role.solid, theme_dark));
                assert_eq!(role.foreground, role.solid);
            }
        }
    }

    #[test]
    fn light_muted_coincides_with_tertiary_but_dark_deliberately_differs() {
        let light = SemanticTokens::light();
        let dark = SemanticTokens::dark();
        assert_eq!(light.foreground.muted, light.foreground.tertiary);
        assert_ne!(dark.foreground.muted, dark.foreground.tertiary);
    }
}
