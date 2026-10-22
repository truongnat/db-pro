use egui::{
    text::{LayoutJob, TextFormat},
    Align, Button, Color32, CornerRadius, FontFamily, FontId, Frame, Margin, Rect, Response, RichText, Sense, Stroke,
    TextEdit, Ui,
};
use lucide_icons::Icon;

use crate::{
    components::card::CARD_FRAME_STROKE_WIDTH,
    tokens::{font_icon, CARD_INNER_PAD, RADIUS_SM, RADIUS_XS, SPACE_SM, SPACE_XS, SPACE_XXS, STROKE_THIN},
    DbProTheme,
};

/// `"1 table"` / `"2 tables"` style labels for dense status chips.
pub fn plural_count(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("1 {singular}")
    } else {
        format!("{count} {plural}")
    }
}

fn icon_layout(icon: Icon, label: &str, color: Color32) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.append(
        &char::from(icon).to_string(),
        0.0,
        TextFormat {
            font_id: FontId::new(14.0, FontFamily::Name("lucide".into())),
            color,
            ..Default::default()
        },
    );
    if !label.is_empty() {
        job.append(
            &format!("  {label}"),
            0.0,
            TextFormat {
                font_id: FontId::proportional(13.0),
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

/// Quiet, centered empty content for workspace surfaces that have no objects yet.
/// Keeping the icon/title/description stack here prevents metadata screens from
/// falling back to a tiny, top-left label that reads like unfinished egui output.
// cc-scan:allow TOO_MANY_PARAMS — context params passed through
pub fn empty_state(ui: &mut Ui, icon: Icon, title: &str, description: &str, theme: DbProTheme) {
    ui.vertical_centered(|ui| {
        ui.add_space(8.0);
        ui.label(
            RichText::new(char::from(icon).to_string())
                .font(font_icon(18.0))
                .color(theme.text_muted),
        );
        ui.label(RichText::new(title).strong().color(theme.text_secondary));
        if !description.is_empty() {
            ui.label(RichText::new(description).small().color(theme.text_muted));
        }
        ui.add_space(8.0);
    });
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

pub fn section_label(ui: &mut Ui, text: impl Into<String>, theme: DbProTheme) -> Response {
    ui.label(
        RichText::new(text.into())
            .font(FontId::new(10.5, FontFamily::Name("ui_medium".into())))
            .strong()
            .color(theme.text_muted),
    )
}

pub fn primary_button(ui: &mut Ui, label: impl Into<RichText>, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(label.into().color(theme.accent_foreground).strong())
            .fill(theme.accent)
            .stroke(Stroke::new(STROKE_THIN, theme.accent))
            .min_size(egui::vec2(0.0, 28.0))
            .corner_radius(CornerRadius::same(crate::tokens::RADIUS_BUTTON as u8)),
    )
}

pub fn primary_button_with_icon(ui: &mut Ui, icon: Icon, label: &str, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(icon_layout(icon, label, theme.accent_foreground))
            .fill(theme.accent)
            .stroke(Stroke::new(STROKE_THIN, theme.accent))
            .min_size(egui::vec2(0.0, 28.0))
            .corner_radius(CornerRadius::same(crate::tokens::RADIUS_BUTTON as u8)),
    )
}

pub fn secondary_button(ui: &mut Ui, label: impl Into<RichText>, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(label.into().color(theme.text_primary))
            .fill(theme.surface_hover)
            .stroke(Stroke::NONE)
            .min_size(egui::vec2(0.0, 28.0))
            .corner_radius(CornerRadius::same(crate::tokens::RADIUS_BUTTON as u8)),
    )
}

pub fn secondary_button_with_icon(ui: &mut Ui, icon: Icon, label: &str, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(icon_layout(icon, label, theme.text_primary))
            .fill(theme.surface_hover)
            .stroke(Stroke::NONE)
            .min_size(egui::vec2(0.0, 28.0))
            .corner_radius(CornerRadius::same(crate::tokens::RADIUS_BUTTON as u8)),
    )
}

pub fn ghost_button(ui: &mut Ui, label: impl Into<RichText>, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(label.into().color(theme.text_secondary))
            .min_size(egui::vec2(0.0, 26.0))
            .corner_radius(CornerRadius::same(crate::tokens::RADIUS_BUTTON as u8))
            .stroke(Stroke::NONE),
    )
}

pub fn ghost_button_with_icon(ui: &mut Ui, icon: Icon, label: &str, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(icon_layout(icon, label, theme.text_secondary))
            .min_size(egui::vec2(0.0, 26.0))
            .corner_radius(CornerRadius::same(crate::tokens::RADIUS_BUTTON as u8))
            .stroke(Stroke::NONE),
    )
}

/// Full-width action row for compact overflow menus.
pub fn menu_button_with_icon(ui: &mut Ui, icon: Icon, label: &str, theme: DbProTheme) -> Response {
    ui.add_sized(
        [ui.available_width(), 28.0],
        Button::new(icon_layout(icon, label, theme.text_primary))
            .corner_radius(CornerRadius::same(crate::tokens::RADIUS_SM as u8))
            .stroke(Stroke::NONE),
    )
}

pub fn danger_button(ui: &mut Ui, label: impl Into<RichText>, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(label.into().color(theme.text_inverse).strong())
            .fill(theme.danger)
            .stroke(Stroke::new(STROKE_THIN, theme.danger))
            .min_size(egui::vec2(0.0, 30.0))
            .corner_radius(CornerRadius::same(8.0 as u8)),
    )
}

pub fn badge(ui: &mut Ui, text: &str, fill: Color32, foreground: Color32) {
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), egui::FontId::proportional(10.0), foreground);
    let size = galley.size() + egui::vec2(12.0, 4.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::same(7.0 as u8), fill);
    ui.painter().galley(rect.min + egui::vec2(6.0, 2.0), galley, foreground);
}

/// Build a tinted version of `color` with the given alpha (0..=255).
/// Used by glow / pulse effects that need a translucent fill.
fn tinted(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

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
    let size = 12.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let center = rect.center();

    if pulsing {
        let phase = if reduce_motion {
            0.0
        } else {
            ui.ctx()
                .animate_value_with_time(egui::Id::new("dbpro.status_dot.phase"), 1.0, 1.6)
        };
        let max_r = 6.0;
        let pulse_r = 3.0 + (max_r - 3.0) * phase;
        let alpha = ((1.0 - phase) * 120.0) as u8;
        ui.painter().circle_filled(center, pulse_r, tinted(color, alpha));
        if !reduce_motion {
            ui.ctx().request_repaint();
        }
    }
    ui.painter().circle_filled(center, 4.0, color);
    response
}
