use egui::{
    text::{LayoutJob, TextFormat},
    Align, Color32, CornerRadius, FontFamily, FontId, Frame, Margin, Rect, Response, RichText, Sense, Stroke, TextEdit,
    Ui,
};
use lucide_icons::Icon;

use crate::{
    components::card::CARD_FRAME_STROKE_WIDTH,
    tokens::{
        font_icon, CARD_INNER_PAD, FONT_SIZE_UI_LABEL, ICON_SM, RADIUS_SM, RADIUS_XS, SPACE_SM, SPACE_XS, SPACE_XXS,
        STROKE_THIN,
    },
    DbProTheme,
};

fn icon_layout(icon: Icon, label: &str, color: Color32) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.append(
        &char::from(icon).to_string(),
        0.0,
        TextFormat {
            font_id: font_icon(ICON_SM),
            color,
            ..Default::default()
        },
    );
    if !label.is_empty() {
        job.append(
            &format!("  {label}"),
            0.0,
            TextFormat {
                font_id: FontId::proportional(FONT_SIZE_UI_LABEL),
                color,
                ..Default::default()
            },
        );
    }
    job
}

pub fn icon_text(icon: Icon, label: &str, color: Color32) -> LayoutJob {
    icon_layout(icon, label, color)
}

pub fn panel_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_panel,
        inner_margin: Margin::symmetric(12.0 as i8, 7.0 as i8),
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
        inner_margin: Margin::symmetric(10.0 as i8, 7.0 as i8),
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
        inner_margin: Margin::symmetric(5.0 as i8, 8.0 as i8),
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
            left: 0.0 as i8,
            right: 0.0 as i8,
            top: (SPACE_XS) as i8,
            bottom: (SPACE_XS) as i8,
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
        inner_margin: Margin::symmetric(10.0 as i8, 5.0 as i8),
        corner_radius: CornerRadius {
            nw: 6.0 as u8,
            ne: 6.0 as u8,
            sw: 0.0 as u8,
            se: 0.0 as u8,
        },
        stroke: if active {
            Stroke::new(STROKE_THIN, theme.border_subtle)
        } else {
            Stroke::NONE
        },
        ..Default::default()
    }
}

/// Content tab: active and hover share the `surface_hover` wash and primary text.
// cc-scan:allow TOO_MANY_PARAMS — context params passed through
pub fn tab_button(ui: &mut Ui, theme: DbProTheme, icon: Option<Icon>, label: &str, selected: bool) -> Response {
    fn job(icon: Option<Icon>, label: &str, color: Color32) -> LayoutJob {
        match icon {
            Some(i) => icon_layout(i, label, color),
            None => {
                let mut j = LayoutJob::default();
                j.append(
                    label,
                    0.0,
                    TextFormat {
                        font_id: FontId::proportional(13.0),
                        color,
                        ..Default::default()
                    },
                );
                j
            }
        }
    }
    let pad = egui::vec2(11.0, 7.0);
    let probe = ui.fonts_mut(|f| f.layout_job(job(icon, label, theme.text_secondary)));
    let (rect, resp) = ui.allocate_exact_size(probe.size() + pad * 2.0, Sense::click());
    resp.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, ui.is_enabled(), selected, label)
    });
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);

    let color = if selected || resp.hovered() {
        theme.text_primary
    } else {
        theme.text_secondary
    };
    if selected || resp.hovered() {
        ui.painter().rect_filled(rect.shrink(1.0), 6.0, theme.surface_hover);
    }
    let galley = ui.fonts_mut(|f| f.layout_job(job(icon, label, color)));
    ui.painter().galley(rect.min + pad, galley, color);
    resp
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

const LEGACY_CARD_FRAME_RADIUS: f32 = 8.0;
/// Vertical inset of a row inside a floating list, in egui points.
const LIST_ROW_PAD_Y: f32 = 3.0;

pub fn card_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_elevated,
        inner_margin: Margin::same(CARD_INNER_PAD as i8),
        outer_margin: Margin::ZERO,
        corner_radius: CornerRadius::same(LEGACY_CARD_FRAME_RADIUS as u8),
        stroke: Stroke::new(CARD_FRAME_STROKE_WIDTH, theme.border_subtle),
        ..Default::default()
    }
}

/// Card container for a selectable/active row (tools hub, launcher grids):
/// `surface_active` + accent border when active, quiet panel otherwise.
pub fn selectable_card_frame(theme: DbProTheme, active: bool) -> Frame {
    Frame {
        fill: if active {
            theme.surface_active
        } else {
            theme.surface_panel
        },
        inner_margin: Margin::symmetric(SPACE_SM as i8, SPACE_XS as i8),
        stroke: Stroke::new(STROKE_THIN, if active { theme.accent } else { theme.border_subtle }),
        corner_radius: CornerRadius::same(RADIUS_SM as u8),
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
        inner_margin: Margin::symmetric(10.0 as i8, 8.0 as i8),
        corner_radius: if user_message {
            CornerRadius::same(10.0 as u8)
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
            left: 0.0 as i8,
            right: 0.0 as i8,
            top: (SPACE_XS) as i8,
            bottom: (SPACE_XS) as i8,
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
            left: 0.0 as i8,
            right: 0.0 as i8,
            top: (SPACE_XS) as i8,
            bottom: (SPACE_XS) as i8,
        },
        corner_radius: CornerRadius::ZERO,
        stroke: Stroke::NONE,
        ..Default::default()
    }
}

/// Compatibility single-line field. New surfaces use `components::input::Input`,
/// which owns the disabled, error, focus, and hover chrome.
// cc-scan:allow TOO_MANY_PARAMS — context params passed through
pub fn input(ui: &mut Ui, value: &mut String, hint: &str, width: f32, theme: DbProTheme) -> Response {
    let response = ui.add(
        TextEdit::singleline(value)
            .hint_text(RichText::new(hint).color(theme.text_muted))
            .desired_width(width)
            .min_size(egui::vec2(width, 32.0))
            .margin(Margin::symmetric(8.0 as i8, 5.0 as i8))
            .vertical_align(Align::Center)
            .text_color(theme.text_primary),
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, ui.is_enabled(), hint));
    response
}

// cc-scan:allow TOO_MANY_PARAMS — context params passed through
pub fn sidebar_item(ui: &mut Ui, icon: Icon, label: &str, active: bool, theme: DbProTheme) -> Response {
    let button = crate::components::button::Button::new(theme)
        .icon(icon)
        .text(label)
        .variant(if active {
            crate::components::button::ButtonVariant::Secondary
        } else {
            crate::components::button::ButtonVariant::Ghost
        })
        .size(crate::components::button::ButtonSize::Sm)
        .full_width(true)
        .left_aligned();
    let response = button.show(ui);
    if active {
        let pill_rect = Rect::from_min_max(
            egui::pos2(response.rect.left() + 1.0, response.rect.top() + 4.0),
            egui::pos2(response.rect.left() + 3.0, response.rect.bottom() - 4.0),
        );
        ui.painter()
            .rect_filled(pill_rect, CornerRadius::same(1.0 as u8), theme.accent);
    }
    response
}

/// Font size of the muted uppercase section labels above field groups, in egui
/// points (10.5 sits between caption 12 and kbd 9).
const SECTION_LABEL_FONT_SIZE: f32 = 10.5;

pub fn section_label(ui: &mut Ui, text: impl Into<String>, theme: DbProTheme) -> Response {
    ui.label(
        RichText::new(text.into())
            .font(FontId::new(
                SECTION_LABEL_FONT_SIZE,
                FontFamily::Name("ui_medium".into()),
            ))
            .strong()
            .color(theme.text_muted),
    )
}

/// Build a tinted version of `color` with the given alpha (0..=255).
/// Used by glow / pulse effects that need a translucent fill.
fn tinted(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

/// Allocated square size of the status dot hit area, in egui points.
const STATUS_DOT_SIZE: f32 = 12.0;
/// Radius of the solid core circle, in egui points.
const STATUS_DOT_CORE_RADIUS: f32 = 4.0;
/// Inner radius where the pulse ring starts, in egui points.
const STATUS_DOT_PULSE_MIN_RADIUS: f32 = 3.0;
/// Outer radius the pulse ring expands to over one cycle, in egui points.
const STATUS_DOT_PULSE_MAX_RADIUS: f32 = 6.0;
/// Pulse ring alpha at the start of the cycle (fades to 0).
const STATUS_DOT_PULSE_MAX_ALPHA: f32 = 120.0;
/// Duration of one pulse cycle, in seconds.
const STATUS_DOT_PULSE_SECS: f32 = 1.6;

// ── Status dot ─────────────────────────────────────────────────────────────

/// Small status dot with an optional pulsing outer ring. Used for connection
/// health, agent provider readiness, etc.
///
/// The dot takes its colour from the caller-supplied `color`; `theme` is
/// accepted for signature symmetry with the other status helpers.
pub fn status_dot(
    ui: &mut Ui,
    color: Color32,
    pulsing: bool,
    reduce_motion: bool,
    _theme: DbProTheme,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(STATUS_DOT_SIZE, STATUS_DOT_SIZE), egui::Sense::hover());
    let center = rect.center();

    if pulsing {
        let phase = if reduce_motion {
            0.0
        } else {
            ui.ctx()
                .animate_value_with_time(egui::Id::new("dbpro.status_dot.phase"), 1.0, STATUS_DOT_PULSE_SECS)
        };
        let pulse_r = STATUS_DOT_PULSE_MIN_RADIUS + (STATUS_DOT_PULSE_MAX_RADIUS - STATUS_DOT_PULSE_MIN_RADIUS) * phase;
        let alpha = ((1.0 - phase) * STATUS_DOT_PULSE_MAX_ALPHA) as u8;
        ui.painter().circle_filled(center, pulse_r, tinted(color, alpha));
        if !reduce_motion {
            ui.ctx().request_repaint();
        }
    }
    ui.painter().circle_filled(center, STATUS_DOT_CORE_RADIUS, color);
    response
}
