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
    /// Resting fill of a text field, textarea, search box or select trigger.
    /// Its own role because a field is a control, not a page: it must separate
    /// from `elevated` (the card it sits on) by fill *and* a resting stroke.
    pub field: Color32,
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
    /// Deep fill for controls that carry white text on a solid status color
    /// (destructive confirm). The accent `solid` reads as black-on-status in the
    /// dark theme, which is illegible as a fill.
    pub fill_strong: Color32,
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

/// Categorical chart series colors (data viz; not interactive UI roles).
/// Ordered: the first unnamed series takes `series[0]`, the next `series[1]`,
/// and so on; renderers index it with `i % series.len()`.
#[derive(Debug, Clone, Copy)]
pub struct Chart {
    /// Categorical series palette, legible on the theme's chart surface.
    pub series: [Color32; 6],
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
    pub chart: Chart,
}

impl StatusRole {
    fn from_solid(solid: Color32, dark: bool) -> Self {
        Self {
            solid,
            foreground: solid,
            fill: solid.linear_multiply(STATUS_FILL_OPACITY),
            border: solid.linear_multiply(STATUS_BORDER_OPACITY),
            subtle: subtle_wash(solid, dark),
            fill_strong: solid,
        }
    }
}

impl SemanticTokens {
    /// Codex-aligned light roles.
    // cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
    pub fn light() -> Self {
        Self {
            background: Background {
                // The light ladder is a paper stack: pure white is the top plane
                // (cards, popovers, editor) and every plane below it steps down by a
                // visible amount — canvas → panel → subtle → hover → selected.
                // Before this ladder existed, canvas/elevated/floating/editor were all
                // #ffffff, so a card on the page measured 1.00–1.08:1 and read as a
                // hairline-only edge. Chrome must stay above `hover`, otherwise a
                // hovered rail row looks like the base fill it sits on.
                canvas: light::NEUTRAL_40,
                panel: light::NEUTRAL_85,
                elevated: light::NEUTRAL_0,
                floating: light::NEUTRAL_0,
                editor: light::NEUTRAL_0,
                field: light::NEUTRAL_75,
                hover: light::NEUTRAL_135,
                selected: light::NEUTRAL_155,
                subtle: light::NEUTRAL_110,
                overlay: light::SCRIM,
            },
            foreground: Foreground {
                primary: light::NEUTRAL_950,
                secondary: light::NEUTRAL_600,
                tertiary: light::NEUTRAL_525,
                muted: light::NEUTRAL_525,
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
                solid_hover: light::BLUE_650,
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
            chart: Chart {
                series: [
                    light::BLUE_700,
                    light::CHART_RED_600,
                    light::AMBER_600,
                    light::GREEN_600,
                    light::CHART_PURPLE_600,
                    light::CYAN_700,
                ],
            },
        }
    }

    /// ChatGPT macOS aligned dark roles tuned for database workspaces.
    // cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
    pub fn dark() -> Self {
        Self {
            background: Background {
                canvas: dark::NEUTRAL_850,
                panel: dark::NEUTRAL_810,
                elevated: dark::NEUTRAL_780,
                floating: dark::NEUTRAL_780,
                editor: dark::NEUTRAL_850,
                hover: dark::NEUTRAL_750,
                // Lighter than hover so a selected row is not the same wash as a pointer hover.
                selected: dark::NEUTRAL_700,
                subtle: dark::NEUTRAL_775,
                // Sunken field fill: one step below `elevated`, so an input on a card or
                // dialog already separates by 1.20:1 before its resting stroke.
                field: dark::NEUTRAL_850,
                overlay: dark::SCRIM,
            },
            foreground: Foreground {
                primary: dark::NEUTRAL_50,
                secondary: dark::NEUTRAL_200,
                tertiary: dark::NEUTRAL_350,
                muted: dark::NEUTRAL_450,
                disabled: dark::NEUTRAL_500,
                inverse: dark::NEUTRAL_900,
                on_accent: dark::NEUTRAL_900,
            },
            border: Border {
                subtle: dark::NEUTRAL_740,
                default: dark::NEUTRAL_700,
                strong: dark::NEUTRAL_650,
                focus: dark::NEUTRAL_50,
                separator: dark::NEUTRAL_740,
            },
            accent: Accent {
                subtle: dark::NEUTRAL_775,
                solid: dark::NEUTRAL_50,
                solid_hover: dark::NEUTRAL_0,
                foreground: dark::NEUTRAL_900,
            },
            status: Status {
                success: StatusRole::from_solid(dark::GREEN_500, true),
                warning: StatusRole::from_solid(dark::AMBER_400, true),
                danger: StatusRole {
                    fill_strong: dark::RED_700,
                    ..StatusRole::from_solid(dark::RED_400, true)
                },
                info: StatusRole::from_solid(dark::BLUE_450, true),
            },
            syntax: Syntax {
                keyword: dark::PURPLE_400,
                string: dark::GREEN_500,
                number: dark::AMBER_400,
                comment: dark::NEUTRAL_450,
                type_: dark::AMBER_400,
                function: dark::CYAN_300,
                operator: dark::NEUTRAL_200,
                punctuation: dark::NEUTRAL_450,
                variable: dark::NEUTRAL_50,
            },
            chart: Chart {
                series: [
                    dark::CHART_BLUE_400,
                    dark::CHART_RED_400,
                    dark::CHART_YELLOW_400,
                    dark::CHART_GREEN_500,
                    dark::CHART_PURPLE_400,
                    dark::CHART_CYAN_500,
                ],
            },
        }
    }
}

#[cfg(test)]
#[path = "tests/semantic_tests.rs"]
mod tests;
