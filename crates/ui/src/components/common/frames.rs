//! Canonical surface frames shared across shell, panel, and editor surfaces.
//!
//! These factories are the single recipe for `egui::Frame` chrome (fill, inset,
//! hairline stroke, radius) so view code never hand-builds frame literals.

use egui::{Color32, CornerRadius, Frame, Margin, Stroke};

use crate::{
    tokens::{RADIUS_SM, RADIUS_XS, SPACE_MD, SPACE_SM, SPACE_XS, SPACE_XXS, STROKE_THIN},
    DbProTheme,
};

/// Vertical inset shared by panel/sidebar strips, in egui points.
const SHELL_STRIP_PAD_Y: f32 = 7.0;
/// Horizontal inset of sidebar content, in egui points.
const SIDEBAR_PAD_X: f32 = 10.0;
/// Horizontal inset of the activity bar, in egui points.
const ACTIVITY_BAR_PAD_X: f32 = 5.0;
/// Horizontal inset of a content tab, in egui points.
const TAB_PAD_X: f32 = 10.0;
/// Vertical inset of a content tab, in egui points.
const TAB_PAD_Y: f32 = 5.0;
/// Vertical inset of a row inside a floating list, in egui points.
const LIST_ROW_PAD_Y: f32 = 3.0;
/// Horizontal inset of an agent message bubble, in egui points.
const AGENT_MESSAGE_PAD_X: f32 = 10.0;
/// Corner radius of a user message bubble, in egui points.
const AGENT_MESSAGE_RADIUS: f32 = 10.0;

pub fn panel_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_panel,
        inner_margin: Margin::symmetric(SPACE_MD as i8, SHELL_STRIP_PAD_Y as i8),
        stroke: Stroke::NONE,
        ..Default::default()
    }
}

/// Full-bleed application canvas. Root background for a whole workspace/settings
/// surface: `surface_app` fill with no inset, stroke or radius — the caller owns
/// the inner layout.
pub fn app_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_app,
        ..Default::default()
    }
}

/// Shared shell frame for navigation surfaces.
pub fn sidebar_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_panel,
        inner_margin: Margin::symmetric(SIDEBAR_PAD_X as i8, SHELL_STRIP_PAD_Y as i8),
        outer_margin: Margin::ZERO,
        stroke: Stroke::NONE,
        ..Default::default()
    }
}

pub fn activity_bar_frame(theme: DbProTheme) -> Frame {
    // Match the primary sidebar surface so the shell reads as one strip
    // instead of a white activity rail beside a gray navigator.
    Frame {
        fill: theme.surface_panel,
        inner_margin: Margin::symmetric(ACTIVITY_BAR_PAD_X as i8, SPACE_SM as i8),
        outer_margin: Margin::ZERO,
        stroke: Stroke::NONE,
        ..Default::default()
    }
}

pub fn toolbar_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_panel,
        // Horizontal inset is owned by CentralPanel (`SHELL_SPLIT_INSET`) so the
        // toolbar lines up with the sidebar content across the splitter.
        inner_margin: Margin {
            left: 0,
            right: 0,
            top: SPACE_XS as i8,
            bottom: SPACE_XS as i8,
        },
        stroke: Stroke::NONE,
        corner_radius: CornerRadius::ZERO,
        ..Default::default()
    }
}

pub fn tab_frame(theme: DbProTheme, active: bool) -> Frame {
    Frame {
        fill: if active {
            theme.surface_active
        } else {
            Color32::TRANSPARENT
        },
        inner_margin: Margin::symmetric(TAB_PAD_X as i8, TAB_PAD_Y as i8),
        corner_radius: CornerRadius {
            nw: RADIUS_SM as u8,
            ne: RADIUS_SM as u8,
            sw: 0,
            se: 0,
        },
        stroke: if active {
            Stroke::new(STROKE_THIN, theme.border_subtle)
        } else {
            Stroke::NONE
        },
        ..Default::default()
    }
}

/// Fill role for [`shell_surface_frame`]. Callers pick a role, never a raw colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellSurface {
    /// Application canvas — top bar and settings body.
    App,
    /// Panel strip — status bar, output dock, tab strip, sidebar.
    Panel,
}

/// Shell chrome surface frame: the fill comes from a [`ShellSurface`] role and the
/// hairline border from the theme, so shell surfaces stop hand-building
/// `egui::Frame` literals in view code. Layout owns the inset via `inner_margin`.
pub fn shell_surface_frame(theme: DbProTheme, surface: ShellSurface, inner_margin: Margin, bordered: bool) -> Frame {
    Frame {
        fill: match surface {
            ShellSurface::App => theme.surface_app,
            ShellSurface::Panel => theme.surface_panel,
        },
        inner_margin,
        stroke: if bordered {
            Stroke::new(STROKE_THIN, theme.border_subtle)
        } else {
            Stroke::NONE
        },
        ..Default::default()
    }
}

/// Compact chip frame for inline editor prefixes (icon + keyword tag), e.g. the
/// `WHERE` marker on the table-data condition editor.
pub fn chip_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_2,
        inner_margin: Margin::symmetric(SPACE_SM as i8, SPACE_XXS as i8),
        corner_radius: CornerRadius::same(RADIUS_XS as u8),
        ..Default::default()
    }
}

/// Content inset for a popup that already owns its surface (`egui::Popup`,
/// `FloatingPopup`): margin only, no fill or stroke, so rows and dividers sit
/// inside the rounded edge instead of bleeding into it.
///
/// Lives here rather than in a view because frame construction belongs to the
/// component layer (`RULES.md` §1: no hand-built `egui::Frame` in view code).
pub fn popup_content_frame() -> Frame {
    Frame::NONE.inner_margin(Margin::symmetric(SPACE_SM as i8, SPACE_SM as i8))
}

/// Row wash for a selectable row inside a floating list (suggestion/completion
/// menus): accent wash when selected, transparent otherwise.
pub fn list_row_frame(theme: DbProTheme, selected: bool) -> Frame {
    Frame {
        fill: if selected {
            theme.accent_soft
        } else {
            Color32::TRANSPARENT
        },
        inner_margin: Margin::symmetric(SPACE_SM as i8, LIST_ROW_PAD_Y as i8),
        corner_radius: CornerRadius::same(RADIUS_XS as u8),
        ..Default::default()
    }
}

pub fn agent_message_frame(theme: DbProTheme, user_message: bool) -> Frame {
    Frame {
        fill: if user_message {
            theme.surface_elevated
        } else {
            Color32::TRANSPARENT
        },
        inner_margin: Margin::symmetric(AGENT_MESSAGE_PAD_X as i8, SPACE_SM as i8),
        corner_radius: if user_message {
            CornerRadius::same(AGENT_MESSAGE_RADIUS as u8)
        } else {
            CornerRadius::ZERO
        },
        stroke: Stroke::NONE,
        ..Default::default()
    }
}

pub fn editor_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_editor,
        // Horizontal inset owned by the shell; keep a tight vertical breath only.
        inner_margin: Margin {
            left: 0,
            right: 0,
            top: SPACE_XS as i8,
            bottom: SPACE_XS as i8,
        },
        corner_radius: CornerRadius::ZERO,
        stroke: Stroke::NONE,
        ..Default::default()
    }
}

pub fn grid_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_editor,
        inner_margin: Margin {
            left: 0,
            right: 0,
            top: SPACE_XS as i8,
            bottom: SPACE_XS as i8,
        },
        corner_radius: CornerRadius::ZERO,
        stroke: Stroke::NONE,
        ..Default::default()
    }
}
