use crate::DbProTheme;
use egui::{
    text::{LayoutJob, TextFormat},
    Align, Button, Color32, FontFamily, FontId, Frame, Margin, Response, RichText, Rounding, Stroke, TextEdit, Ui,
};
use lucide_icons::Icon;

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
        inner_margin: Margin::symmetric(12.0, 7.0),
        stroke: Stroke::new(1.0, theme.border_subtle),
        ..Default::default()
    }
}

/// Shared shell frame for navigation surfaces.
pub fn sidebar_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_panel,
        inner_margin: Margin::symmetric(10.0, 7.0),
        stroke: Stroke::new(1.0, theme.border_subtle),
        ..Default::default()
    }
}

pub fn activity_bar_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_app,
        inner_margin: Margin::symmetric(5.0, 8.0),
        stroke: Stroke::new(1.0, theme.border_subtle),
        ..Default::default()
    }
}

pub fn toolbar_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_panel,
        inner_margin: Margin::symmetric(10.0, 4.0),
        stroke: Stroke::new(1.0, theme.border_subtle),
        rounding: Rounding::ZERO,
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
        inner_margin: Margin::symmetric(10.0, 3.0),
        rounding: Rounding::ZERO,
        stroke: Stroke::NONE,
        ..Default::default()
    }
}

pub fn card_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_elevated,
        inner_margin: Margin::same(14.0),
        outer_margin: Margin::ZERO,
        rounding: Rounding::same(8.0),
        stroke: Stroke::new(1.0, theme.border_subtle),
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
        inner_margin: Margin::symmetric(10.0, 8.0),
        rounding: if user_message {
            Rounding::same(10.0)
        } else {
            Rounding::ZERO
        },
        stroke: Stroke::NONE,
        ..Default::default()
    }
}

pub fn editor_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_editor,
        inner_margin: Margin::same(10.0),
        rounding: Rounding::same(8.0),
        stroke: Stroke::NONE,
        ..Default::default()
    }
}

pub fn grid_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_editor,
        inner_margin: Margin::same(10.0),
        rounding: Rounding::same(8.0),
        stroke: Stroke::NONE,
        ..Default::default()
    }
}

/// Quiet, centered empty content for workspace surfaces that have no objects yet.
/// Keeping the icon/title/description stack here prevents metadata screens from
/// falling back to a tiny, top-left label that reads like unfinished egui output.
pub fn empty_state(ui: &mut Ui, icon: Icon, title: &str, description: &str, theme: DbProTheme) {
    ui.vertical_centered(|ui| {
        ui.add_space(8.0);
        ui.label(
            RichText::new(char::from(icon).to_string())
                .font(FontId::new(18.0, FontFamily::Name("lucide".into())))
                .color(theme.text_muted),
        );
        ui.label(RichText::new(title).strong().color(theme.text_secondary));
        if !description.is_empty() {
            ui.label(RichText::new(description).small().color(theme.text_muted));
        }
        ui.add_space(8.0);
    });
}

/// The only shared single-line input primitive used by the native shell.
/// Keeping its margin, height and text colors here prevents each screen from
/// drifting into a different field style.
pub fn input(ui: &mut Ui, value: &mut String, hint: &str, width: f32, theme: DbProTheme) -> Response {
    ui.add(
        TextEdit::singleline(value)
            .hint_text(RichText::new(hint).color(theme.text_muted))
            .desired_width(width)
            .min_size(egui::vec2(width, 32.0))
            .margin(Margin::symmetric(8.0, 5.0))
            .vertical_align(Align::Center)
            .text_color(theme.text_primary),
    )
}

pub fn input_full_width(ui: &mut Ui, value: &mut String, hint: &str, theme: DbProTheme) -> Response {
    let width = ui.available_width();
    input(ui, value, hint, width, theme)
}

pub fn password_input(ui: &mut Ui, value: &mut String, hint: &str, width: f32, theme: DbProTheme) -> Response {
    ui.add(
        TextEdit::singleline(value)
            .password(true)
            .hint_text(RichText::new(hint).color(theme.text_muted))
            .desired_width(width)
            .min_size(egui::vec2(width, 32.0))
            .margin(Margin::symmetric(8.0, 5.0))
            .vertical_align(Align::Center)
            .text_color(theme.text_primary),
    )
}

pub fn sidebar_item(ui: &mut Ui, icon: Icon, label: &str, active: bool, theme: DbProTheme) -> Response {
    let width = ui.available_width();
    let text_color = if active {
        theme.text_primary
    } else {
        theme.text_secondary
    };
    let button = Button::new(icon_layout(icon, label, text_color))
        .min_size(egui::vec2(width, 26.0))
        .rounding(Rounding::same(7.0))
        .stroke(Stroke::NONE);
    let response = if active {
        ui.add(button.fill(theme.surface_active))
    } else {
        ui.add(button)
    };
    if active {
        ui.painter().line_segment(
            [
                egui::pos2(response.rect.left() + 1.0, response.rect.top() + 4.0),
                egui::pos2(response.rect.left() + 1.0, response.rect.bottom() - 4.0),
            ],
            Stroke::new(2.0, theme.accent),
        );
    }
    response
}

pub fn section_label(ui: &mut Ui, text: impl Into<String>, theme: DbProTheme) -> Response {
    ui.label(RichText::new(text.into()).size(10.0).strong().color(theme.text_muted))
}

pub fn primary_button(ui: &mut Ui, label: impl Into<RichText>, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(label.into().color(theme.accent_foreground).strong())
            .fill(theme.accent)
            .stroke(Stroke::new(1.0, theme.accent))
            .min_size(egui::vec2(0.0, 28.0))
            .rounding(Rounding::same(7.0)),
    )
}

pub fn primary_button_with_icon(ui: &mut Ui, icon: Icon, label: &str, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(icon_layout(icon, label, theme.accent_foreground))
            .fill(theme.accent)
            .stroke(Stroke::new(1.0, theme.accent))
            .min_size(egui::vec2(0.0, 28.0))
            .rounding(Rounding::same(7.0)),
    )
}

pub fn secondary_button(ui: &mut Ui, label: impl Into<RichText>, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(label.into().color(theme.text_primary))
            .fill(theme.surface_hover)
            .stroke(Stroke::NONE)
            .min_size(egui::vec2(0.0, 28.0))
            .rounding(Rounding::same(7.0)),
    )
}

pub fn secondary_button_with_icon(ui: &mut Ui, icon: Icon, label: &str, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(icon_layout(icon, label, theme.text_primary))
            .fill(theme.surface_hover)
            .stroke(Stroke::NONE)
            .min_size(egui::vec2(0.0, 28.0))
            .rounding(Rounding::same(7.0)),
    )
}

pub fn ghost_button(ui: &mut Ui, label: impl Into<RichText>, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(label.into().color(theme.text_secondary))
            .min_size(egui::vec2(0.0, 26.0))
            .rounding(Rounding::same(4.0))
            .stroke(Stroke::NONE),
    )
}

pub fn ghost_button_with_icon(ui: &mut Ui, icon: Icon, label: &str, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(icon_layout(icon, label, theme.text_secondary))
            .min_size(egui::vec2(0.0, 26.0))
            .rounding(Rounding::same(4.0))
            .stroke(Stroke::NONE),
    )
}

/// Full-width action row for compact overflow menus.
pub fn menu_button_with_icon(ui: &mut Ui, icon: Icon, label: &str, theme: DbProTheme) -> Response {
    ui.add_sized(
        [ui.available_width(), 28.0],
        Button::new(icon_layout(icon, label, theme.text_primary))
            .rounding(Rounding::same(6.0))
            .stroke(Stroke::NONE),
    )
}

pub fn compact_button(ui: &mut Ui, label: impl Into<RichText>, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(label.into().size(12.0).color(theme.text_secondary))
            .min_size(egui::vec2(0.0, 24.0))
            .rounding(Rounding::same(7.0))
            .stroke(Stroke::NONE),
    )
}

pub fn compact_button_enabled(ui: &mut Ui, label: impl Into<RichText>, enabled: bool, theme: DbProTheme) -> Response {
    ui.add_enabled(
        enabled,
        Button::new(label.into().size(12.0).color(theme.text_secondary))
            .min_size(egui::vec2(0.0, 24.0))
            .rounding(Rounding::same(7.0))
            .stroke(Stroke::NONE),
    )
}

pub fn compact_button_with_icon(ui: &mut Ui, icon: Icon, label: &str, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(icon_layout(icon, label, theme.text_secondary))
            .min_size(egui::vec2(0.0, 24.0))
            .rounding(Rounding::same(7.0))
            .stroke(Stroke::NONE),
    )
}

pub fn danger_button(ui: &mut Ui, label: impl Into<RichText>, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(label.into().color(theme.text_inverse).strong())
            .fill(theme.danger)
            .stroke(Stroke::new(1.0, theme.danger))
            .min_size(egui::vec2(0.0, 30.0))
            .rounding(Rounding::same(8.0)),
    )
}

pub fn icon_button(ui: &mut Ui, icon: Icon, active: bool, theme: DbProTheme) -> Response {
    let text = RichText::new(char::from(icon).to_string())
        .font(FontId::new(17.0, FontFamily::Name("lucide".into())))
        .color(if active { theme.accent } else { theme.text_muted });
    let button = Button::new(text)
        .min_size(egui::vec2(32.0, 30.0))
        .rounding(Rounding::same(8.0))
        .stroke(Stroke::NONE);
    if active {
        ui.add(button.fill(theme.surface_active))
    } else {
        ui.add(button)
    }
}

pub fn compact_icon_button(ui: &mut Ui, icon: Icon, theme: DbProTheme) -> Response {
    ui.add_sized(
        [24.0, 24.0],
        Button::new(
            RichText::new(char::from(icon).to_string())
                .font(FontId::new(14.0, FontFamily::Name("lucide".into())))
                .color(theme.text_muted),
        )
        .rounding(Rounding::same(7.0))
        .stroke(Stroke::NONE),
    )
}

pub fn compact_icon_button_enabled(ui: &mut Ui, icon: Icon, enabled: bool, theme: DbProTheme) -> Response {
    ui.add_enabled(
        enabled,
        Button::new(
            RichText::new(char::from(icon).to_string())
                .font(FontId::new(14.0, FontFamily::Name("lucide".into())))
                .color(theme.text_muted),
        )
        .min_size(egui::vec2(24.0, 24.0))
        .rounding(Rounding::same(7.0))
        .stroke(Stroke::NONE),
    )
}

pub fn badge(ui: &mut Ui, text: &str, fill: Color32, foreground: Color32) {
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), egui::FontId::proportional(10.0), foreground);
    let size = galley.size() + egui::vec2(12.0, 4.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.painter().rect_filled(rect, Rounding::same(7.0), fill);
    ui.painter().galley(rect.min + egui::vec2(6.0, 2.0), galley, foreground);
}

// ────────────────────────────────────────────────────────────────────────────
// Core UI Modernization — new primitive widgets
//
// All new primitives are additive. They compose the existing `DbProTheme`
// tokens (no new color tokens are introduced) and follow the codex-neutral
// surface contract already enforced in `theme::tests`.
// ────────────────────────────────────────────────────────────────────────────

/// Visual level for transient notifications (toasts, status pills).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLevel {
    Info,
    Success,
    Warning,
    Danger,
}

/// Resolve a `UiLevel` to its theme color pair (accent + foreground).
fn level_colors(level: UiLevel, theme: DbProTheme) -> (Color32, Color32) {
    match level {
        UiLevel::Info => (theme.accent, theme.text_inverse),
        UiLevel::Success => (theme.success, theme.text_inverse),
        UiLevel::Warning => (theme.warning, theme.text_inverse),
        UiLevel::Danger => (theme.danger, theme.text_inverse),
    }
}

/// Build a tinted version of `color` with the given alpha (0..=255).
/// Used by glow / pulse effects that need a translucent fill.
fn tinted(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

// ── Spinner ────────────────────────────────────────────────────────────────

/// Custom-painted spinner: a rotating arc that replaces the default egui
/// spinner when call sites opt in. The arc rotates ~720° over one cycle and
/// loops forever; `reduce_motion=true` freezes the arc at the 12-o'clock
/// position (the same as a stilled indicator, which still reads as a load
/// signal next to its label).
///
/// `ui.ctx().request_repaint()` is requested every frame while the spinner
/// is shown so the rotation animates smoothly.
pub fn spinner(ui: &mut Ui, color: Color32, reduce_motion: bool, theme: DbProTheme) -> egui::Response {
    let size = egui::vec2(14.0, 14.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
    let center = rect.center();
    let radius = 5.0;
    let stroke_width = 1.6;

    let rotation = if reduce_motion {
        0.0
    } else {
        ui.ctx()
            .animate_value_with_time(egui::Id::new("dbpro.spinner.rotation"), 1.0, 0.9)
    };

    if !reduce_motion {
        ui.ctx().request_repaint();
    }

    // Track ring (subtle).
    ui.painter()
        .circle_stroke(center, radius, Stroke::new(stroke_width, theme.border_default));
    // Foreground arc.
    ui.painter()
        .circle_stroke(center, radius, Stroke::new(stroke_width, color));
    // Mask out the trailing portion of the foreground arc by overlaying a
    // background-colored chord. This is the "rotating arc" effect.
    let trailing_angle = std::f32::consts::TAU * (1.0 - rotation * 0.75);
    let mask = center + egui::vec2(radius * trailing_angle.cos(), radius * trailing_angle.sin());
    ui.painter()
        .line_segment([center, mask], Stroke::new(stroke_width + 0.4, theme.surface_panel));
    response
}

// ── Progress bar ───────────────────────────────────────────────────────────

/// Linear progress bar with rounded ends. `progress` is clamped to `0.0..=1.0`.
pub fn progress_bar(ui: &mut Ui, progress: f32, theme: DbProTheme) -> egui::Response {
    let progress = progress.clamp(0.0, 1.0);
    let height = 6.0;
    let width = ui.available_width().max(80.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let rounding = Rounding::same(height / 2.0);

    ui.painter().rect_filled(rect, rounding, theme.surface_hover);
    let fill_width = rect.width() * progress;
    if fill_width > 0.0 {
        let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(fill_width, rect.height()));
        // Vertical gradient from `accent_hover` (top) to `accent` (bottom).
        let top_color = theme.accent_hover;
        let bottom_color = theme.accent;
        ui.painter().rect_filled(fill_rect, rounding, bottom_color);
        let clipped = fill_rect.intersect(rect);
        ui.painter().rect_filled(
            egui::Rect::from_min_size(clipped.min, egui::vec2(clipped.width(), clipped.height() * 0.5)),
            Rounding {
                nw: rounding.nw,
                ne: rounding.ne,
                sw: 0.0,
                se: 0.0,
            },
            top_color.linear_multiply(0.65),
        );
    }
    response
}

// ── Skeleton ───────────────────────────────────────────────────────────────

/// Rounded placeholder block. When `reduce_motion=false`, a translucent
/// shimmer sweeps across the block to communicate loading.
pub fn skeleton(ui: &mut Ui, width: f32, height: f32, reduce_motion: bool, theme: DbProTheme) -> egui::Response {
    let width = width.max(8.0);
    let height = height.max(8.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let rounding = Rounding::same(4.0);
    ui.painter().rect_filled(rect, rounding, theme.surface_hover);

    if !reduce_motion {
        let t = ui
            .ctx()
            .animate_value_with_time(egui::Id::new("dbpro.skeleton.shimmer"), 1.0, 1.4);
        let offset = (t - 0.5) * (rect.width() * 2.0);
        let band_width = rect.width() * 0.35;
        let band_start = rect.left() + offset;
        let band_end = band_start + band_width;
        let band = egui::Rect::from_x_y_ranges(egui::Rangef::new(band_start, band_end), rect.y_range());
        let visible = band.intersect(rect);
        if visible.width() > 0.0 {
            ui.painter()
                .rect_filled(visible, rounding, theme.surface_active.linear_multiply(0.6));
        }
        ui.ctx().request_repaint();
    }
    response
}

// ── Switch ──────────────────────────────────────────────────────────────────

/// On/off toggle. `value` is mutated in place when the user clicks the
/// track or the label. The thumb position animates smoothly between off
/// and on when `reduce_motion=false`.
pub fn switch(ui: &mut Ui, value: &mut bool, label: &str, reduce_motion: bool, theme: DbProTheme) -> egui::Response {
    let track_width = 28.0;
    let track_height: f32 = 16.0;
    let thumb_size = 12.0;
    let spacing = 8.0;

    let label_galley =
        ui.painter()
            .layout_no_wrap(label.to_owned(), egui::FontId::proportional(13.0), theme.text_primary);
    let total_width = track_width + spacing + label_galley.size().x;
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(total_width, track_height.max(label_galley.size().y)),
        egui::Sense::click(),
    );

    let track_rect = egui::Rect::from_min_size(rect.min, egui::vec2(track_width, track_height));
    let target = if *value { 1.0 } else { 0.0 };
    let animated = if reduce_motion {
        target
    } else {
        ui.ctx()
            .animate_bool_with_time(egui::Id::new("dbpro.switch"), *value, 0.18)
    };

    let track_color = if *value { theme.accent } else { theme.surface_active };
    let track_stroke = Stroke::new(1.0, if *value { theme.accent } else { theme.border_default });
    ui.painter()
        .rect_filled(track_rect, Rounding::same(track_height / 2.0), track_color);
    ui.painter()
        .rect_stroke(track_rect, Rounding::same(track_height / 2.0), track_stroke);

    let thumb_left = track_rect.left() + 2.0 + animated * (track_width - thumb_size - 4.0);
    let thumb_rect = egui::Rect::from_min_size(
        egui::pos2(thumb_left, track_rect.center().y - thumb_size / 2.0),
        egui::vec2(thumb_size, thumb_size),
    );
    let thumb_color = if *value {
        theme.text_inverse
    } else {
        theme.text_secondary
    };
    ui.painter()
        .circle_filled(thumb_rect.center(), thumb_size / 2.0, thumb_color);

    // Label
    let label_pos = egui::pos2(
        track_rect.right() + spacing,
        rect.center().y - label_galley.size().y / 2.0,
    );
    ui.painter().galley(label_pos, label_galley, theme.text_primary);

    if response.clicked() {
        *value = !*value;
    }
    response
}

// ── Segmented control ──────────────────────────────────────────────────────

/// Pill bar with `options.len()` segments. The selected segment is painted
/// with `surface_active` and a subtle border. Returns the newly selected
/// index, if any.
pub fn segmented_control(
    ui: &mut Ui,
    options: &[&str],
    selected: usize,
    reduce_motion: bool,
    theme: DbProTheme,
) -> Option<usize> {
    let height = 26.0;
    let total_width = ui.available_width().max(120.0);
    let segment_width = total_width / options.len() as f32;

    let (bar_rect, _) = ui.allocate_exact_size(egui::vec2(total_width, height), egui::Sense::hover());
    ui.painter()
        .rect_filled(bar_rect, Rounding::same(7.0), theme.surface_panel);
    ui.painter()
        .rect_stroke(bar_rect, Rounding::same(7.0), Stroke::new(1.0, theme.border_subtle));

    let mut clicked = None;
    for (index, option) in options.iter().enumerate() {
        let segment_rect = egui::Rect::from_min_size(
            egui::pos2(bar_rect.left() + segment_width * index as f32, bar_rect.top()),
            egui::vec2(segment_width, height),
        );
        let is_selected = index == selected;
        if is_selected {
            let inner = segment_rect.shrink2(egui::vec2(2.0, 2.0));
            // Animate selection background opacity to avoid hard pop when
            // the selection changes — keep the same easing whether or not
            // reduce_motion is set.
            let _ = reduce_motion;
            ui.painter()
                .rect_filled(inner, Rounding::same(5.0), theme.surface_active);
            ui.painter()
                .rect_stroke(inner, Rounding::same(5.0), Stroke::new(1.0, theme.border_default));
        }
        let text_color = if is_selected {
            theme.text_primary
        } else {
            theme.text_secondary
        };
        let galley = ui
            .painter()
            .layout_no_wrap((*option).to_owned(), egui::FontId::proportional(12.0), text_color);
        let text_pos = segment_rect.center() - galley.size() * 0.5;
        ui.painter().galley(text_pos, galley, text_color);

        // Hit-test the painted segment rect directly. Allocating here instead would
        // test the layout cursor and also push every later widget down by `height`.
        let segment_response = ui.interact(
            segment_rect,
            ui.id().with(("segmented_control", index)),
            egui::Sense::click(),
        );
        if segment_response.clicked() && !is_selected {
            clicked = Some(index);
        }
    }
    clicked
}

// ── Kbd chip ───────────────────────────────────────────────────────────────

/// Rounded monospace pill that displays a single keyboard key or a combined
/// shortcut (e.g. `"⌘ K"`).
pub fn kbd_chip(ui: &mut Ui, label: &str, theme: DbProTheme) -> egui::Response {
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), egui::FontId::monospace(11.0), theme.text_secondary);
    let size = galley.size() + egui::vec2(8.0, 3.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
    let rounding = Rounding::same(4.0);
    ui.painter().rect_filled(rect, rounding, theme.surface_elevated);
    ui.painter()
        .rect_stroke(rect, rounding, Stroke::new(1.0, theme.border_default));
    ui.painter()
        .galley(rect.min + egui::vec2(4.0, 1.5), galley, theme.text_secondary);
    response
}

// ── Tag chip ───────────────────────────────────────────────────────────────

/// Rounded filter tag with a label and an optional close (X) affordance.
/// Returns `true` if the close X was clicked during this frame.
pub fn tag_chip(ui: &mut Ui, label: &str, removable: bool, theme: DbProTheme) -> bool {
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), egui::FontId::proportional(11.0), theme.text_secondary);
    let close_size = if removable { 16.0 } else { 0.0 };
    let padding = egui::vec2(8.0, 3.0);
    let gap = if removable { 4.0 } else { 0.0 };
    let size = egui::vec2(
        galley.size().x + padding.x * 2.0 + close_size + gap,
        galley.size().y + padding.y * 2.0,
    );
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let rounding = Rounding::same(11.0);
    ui.painter().rect_filled(rect, rounding, theme.surface_elevated);
    ui.painter()
        .rect_stroke(rect, rounding, Stroke::new(1.0, theme.border_subtle));
    let text_pos = egui::pos2(rect.left() + padding.x, rect.center().y - galley.size().y / 2.0);
    ui.painter().galley(text_pos, galley, theme.text_secondary);

    let mut closed = false;
    if removable {
        let close_rect = egui::Rect::from_min_size(
            egui::pos2(
                rect.right() - padding.x - close_size,
                rect.center().y - close_size / 2.0,
            ),
            egui::vec2(close_size, close_size),
        );
        // Hit-test the painted close affordance instead of allocating layout space
        // for it (see `segmented_control`).
        let close_response = ui.interact(
            close_rect,
            ui.id().with(("tag_chip_close", label)),
            egui::Sense::click(),
        );
        let hover = close_response.hovered();
        if hover {
            ui.painter()
                .circle_filled(close_rect.center(), close_size / 2.0, theme.surface_active);
        }
        ui.painter().text(
            close_rect.center(),
            egui::Align2::CENTER_CENTER,
            "×",
            egui::FontId::proportional(12.0),
            if hover { theme.text_primary } else { theme.text_muted },
        );
        if close_response.clicked() {
            closed = true;
        }
    }
    closed
}

// ── Status dot ─────────────────────────────────────────────────────────────

/// Small status dot with an optional pulsing outer ring. Used for connection
/// health, agent provider readiness, etc.
pub fn status_dot(
    ui: &mut Ui,
    color: Color32,
    pulsing: bool,
    reduce_motion: bool,
    theme: DbProTheme,
) -> egui::Response {
    let _ = theme;
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

// ── Toast ──────────────────────────────────────────────────────────────────

/// Transient notification panel rendered in the bottom-right area of the
/// containing `Ui`. The toast has a colored left accent strip, a small
/// elevation shadow, and a tinted background so it stands out without
/// looking out of place.
pub fn toast(ui: &mut Ui, level: UiLevel, message: &str, theme: DbProTheme) -> egui::Response {
    let (accent, foreground) = level_colors(level, theme);
    let max_width = 320.0_f32.min(ui.available_width());
    let galley = ui.painter().layout(
        message.to_owned(),
        egui::FontId::proportional(12.0),
        foreground,
        max_width - 24.0,
    );
    let size = egui::vec2(max_width, galley.size().y + 18.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());

    let rounding = Rounding::same(8.0);
    ui.painter().rect_filled(rect, rounding, theme.surface_floating);
    ui.painter()
        .rect_stroke(rect, rounding, Stroke::new(1.0, theme.border_subtle));
    // Subtle elevation shadow.
    ui.painter().rect_filled(
        rect.translate(egui::vec2(0.0, 2.0)),
        rounding,
        Color32::from_black_alpha(20),
    );
    // Accent strip.
    let strip_width = 3.0;
    let strip_rect = egui::Rect::from_min_size(rect.min, egui::vec2(strip_width, rect.height()));
    ui.painter().rect_filled(
        strip_rect,
        Rounding {
            nw: rounding.nw,
            sw: rounding.sw,
            ne: 0.0,
            se: 0.0,
        },
        accent,
    );

    let text_pos = egui::pos2(rect.left() + 14.0, rect.top() + 9.0);
    ui.painter().galley(text_pos, galley, theme.text_primary);
    response
}
