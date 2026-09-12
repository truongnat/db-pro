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
        inner_margin: Margin::symmetric(10.0, 6.0),
        stroke: Stroke::NONE,
        ..Default::default()
    }
}

/// Shared shell frame for navigation surfaces.
pub fn sidebar_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_panel,
        inner_margin: Margin::symmetric(9.0, 6.0),
        stroke: Stroke::NONE,
        ..Default::default()
    }
}

pub fn activity_bar_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_panel,
        inner_margin: Margin::symmetric(6.0, 8.0),
        stroke: Stroke::NONE,
        ..Default::default()
    }
}

pub fn toolbar_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_panel,
        inner_margin: Margin::symmetric(8.0, 4.0),
        stroke: Stroke::NONE,
        rounding: Rounding::ZERO,
        ..Default::default()
    }
}

pub fn tab_frame(theme: DbProTheme, active: bool) -> Frame {
    Frame {
        fill: if active {
            theme.surface_elevated
        } else {
            Color32::TRANSPARENT
        },
        inner_margin: Margin::symmetric(10.0, 5.0),
        rounding: Rounding {
            nw: 6.0,
            ne: 6.0,
            sw: 0.0,
            se: 0.0,
        },
        stroke: if active {
            Stroke::new(1.0, theme.border_subtle)
        } else {
            Stroke::NONE
        },
        ..Default::default()
    }
}

pub fn card_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_elevated,
        inner_margin: Margin::same(12.0),
        outer_margin: Margin::ZERO,
        rounding: Rounding::same(8.0),
        stroke: Stroke::NONE,
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
            Rounding::same(8.0)
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
        rounding: Rounding::same(6.0),
        stroke: Stroke::NONE,
        ..Default::default()
    }
}

pub fn grid_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_editor,
        inner_margin: Margin::same(10.0),
        rounding: Rounding::same(6.0),
        stroke: Stroke::NONE,
        ..Default::default()
    }
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
        .min_size(egui::vec2(width, 28.0))
        .rounding(Rounding::same(4.0))
        .stroke(Stroke::NONE);
    if active {
        ui.add(button.fill(theme.accent_soft))
    } else {
        ui.add(button)
    }
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
            .rounding(Rounding::same(4.0)),
    )
}

pub fn primary_button_with_icon(ui: &mut Ui, icon: Icon, label: &str, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(icon_layout(icon, label, theme.accent_foreground))
            .fill(theme.accent)
            .stroke(Stroke::new(1.0, theme.accent))
            .min_size(egui::vec2(0.0, 28.0))
            .rounding(Rounding::same(4.0)),
    )
}

pub fn secondary_button(ui: &mut Ui, label: impl Into<RichText>, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(label.into().color(theme.text_primary))
            .fill(theme.surface_hover)
            .stroke(Stroke::NONE)
            .min_size(egui::vec2(0.0, 28.0))
            .rounding(Rounding::same(4.0)),
    )
}

pub fn secondary_button_with_icon(ui: &mut Ui, icon: Icon, label: &str, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(icon_layout(icon, label, theme.text_primary))
            .fill(theme.surface_hover)
            .stroke(Stroke::NONE)
            .min_size(egui::vec2(0.0, 28.0))
            .rounding(Rounding::same(4.0)),
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

pub fn compact_button(ui: &mut Ui, label: impl Into<RichText>, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(label.into().size(12.0).color(theme.text_secondary))
            .min_size(egui::vec2(0.0, 24.0))
            .rounding(Rounding::same(5.0))
            .stroke(Stroke::NONE),
    )
}

pub fn compact_button_enabled(ui: &mut Ui, label: impl Into<RichText>, enabled: bool, theme: DbProTheme) -> Response {
    ui.add_enabled(
        enabled,
        Button::new(label.into().size(12.0).color(theme.text_secondary))
            .min_size(egui::vec2(0.0, 24.0))
            .rounding(Rounding::same(5.0))
            .stroke(Stroke::NONE),
    )
}

pub fn compact_button_with_icon(ui: &mut Ui, icon: Icon, label: &str, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(icon_layout(icon, label, theme.text_secondary))
            .min_size(egui::vec2(0.0, 24.0))
            .rounding(Rounding::same(5.0))
            .stroke(Stroke::NONE),
    )
}

pub fn danger_button(ui: &mut Ui, label: impl Into<RichText>, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(label.into().color(theme.text_inverse).strong())
            .fill(theme.danger)
            .stroke(Stroke::new(1.0, theme.danger))
            .min_size(egui::vec2(0.0, 30.0))
            .rounding(Rounding::same(6.0)),
    )
}

pub fn icon_button(ui: &mut Ui, icon: Icon, active: bool, theme: DbProTheme) -> Response {
    let text = RichText::new(char::from(icon).to_string())
        .font(FontId::new(17.0, FontFamily::Name("lucide".into())))
        .color(if active { theme.accent } else { theme.text_muted });
    let button = Button::new(text)
        .min_size(egui::vec2(32.0, 32.0))
        .rounding(Rounding::same(5.0))
        .stroke(Stroke::NONE);
    if active {
        ui.add(button.fill(theme.accent_soft))
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
        .rounding(Rounding::same(5.0))
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
        .rounding(Rounding::same(5.0))
        .stroke(Stroke::NONE),
    )
}

pub fn badge(ui: &mut Ui, text: &str, fill: Color32, foreground: Color32) {
    Frame {
        fill,
        inner_margin: Margin::symmetric(6.0, 2.0),
        rounding: Rounding::same(4.0),
        ..Default::default()
    }
    .show(ui, |ui| {
        ui.label(RichText::new(text).size(10.0).strong().color(foreground));
    });
}
