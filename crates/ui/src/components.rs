use crate::DbProTheme;
use egui::{
    text::{LayoutJob, TextFormat},
    Align2, Button, Color32, FontFamily, FontId, Frame, Margin, Response, RichText, Rounding, Sense, Stroke, TextEdit,
    Ui,
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
        inner_margin: Margin::symmetric(12.0, 8.0),
        stroke: Stroke::new(1.0, theme.border_subtle),
        ..Default::default()
    }
}

/// Shared Codex-light shell frame for navigation surfaces.
pub fn sidebar_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_panel,
        inner_margin: Margin::symmetric(12.0, 8.0),
        stroke: Stroke::new(1.0, theme.border_subtle),
        ..Default::default()
    }
}

pub fn activity_bar_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_panel,
        inner_margin: Margin::symmetric(8.0, 10.0),
        stroke: Stroke::new(1.0, theme.border_subtle),
        ..Default::default()
    }
}

pub fn toolbar_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_panel,
        inner_margin: Margin::symmetric(12.0, 6.0),
        stroke: Stroke::new(1.0, theme.border_subtle),
        rounding: Rounding::same(7.0),
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
        inner_margin: Margin::symmetric(10.0, 5.0),
        rounding: Rounding::same(6.0),
        stroke: Stroke::new(
            1.0,
            if active {
                theme.accent_soft
            } else {
                Color32::TRANSPARENT
            },
        ),
        ..Default::default()
    }
}

pub fn card_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_elevated,
        inner_margin: Margin::same(16.0),
        outer_margin: Margin::ZERO,
        rounding: Rounding::same(10.0),
        stroke: Stroke::new(1.0, theme.border_default),
        shadow: egui::epaint::Shadow {
            offset: egui::vec2(0.0, 2.0),
            blur: 10.0,
            spread: 0.0,
            color: Color32::from_black_alpha(16),
        },
    }
}

pub fn editor_frame(theme: DbProTheme) -> Frame {
    Frame {
        fill: theme.surface_editor,
        inner_margin: Margin::same(12.0),
        rounding: Rounding::same(8.0),
        stroke: Stroke::new(1.0, theme.border_default),
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
            .margin(Margin::symmetric(8.0, 5.0))
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
            .margin(Margin::symmetric(8.0, 5.0))
            .text_color(theme.text_primary),
    )
}

pub fn sidebar_item(ui: &mut Ui, icon: Icon, label: &str, active: bool, theme: DbProTheme) -> Response {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 30.0), Sense::click());
    let fill = if active {
        theme.accent_soft
    } else if response.hovered() {
        theme.surface_hover
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, Rounding::same(6.0), fill);
    ui.painter().text(
        egui::pos2(rect.left() + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        char::from(icon).to_string(),
        FontId::new(14.0, FontFamily::Name("lucide".into())),
        if active { theme.accent } else { theme.text_secondary },
    );
    ui.painter().text(
        egui::pos2(rect.left() + 34.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.0),
        if active {
            theme.text_primary
        } else {
            theme.text_secondary
        },
    );
    response
}

pub fn section_label(ui: &mut Ui, text: impl Into<String>, theme: DbProTheme) -> Response {
    ui.label(RichText::new(text.into()).size(11.0).strong().color(theme.text_muted))
}

pub fn primary_button(ui: &mut Ui, label: impl Into<RichText>, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(label.into().color(theme.accent_foreground).strong())
            .fill(theme.accent)
            .stroke(Stroke::new(1.0, theme.accent))
            .min_size(egui::vec2(0.0, 30.0))
            .rounding(Rounding::same(6.0)),
    )
}

pub fn primary_button_with_icon(ui: &mut Ui, icon: Icon, label: &str, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(icon_layout(icon, label, theme.accent_foreground))
            .fill(theme.accent)
            .stroke(Stroke::new(1.0, theme.accent))
            .min_size(egui::vec2(0.0, 30.0))
            .rounding(Rounding::same(6.0)),
    )
}

pub fn secondary_button(ui: &mut Ui, label: impl Into<RichText>, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(label.into().color(theme.text_primary))
            .fill(theme.surface_panel)
            .stroke(Stroke::new(1.0, theme.border_default))
            .min_size(egui::vec2(0.0, 30.0))
            .rounding(Rounding::same(6.0)),
    )
}

pub fn secondary_button_with_icon(ui: &mut Ui, icon: Icon, label: &str, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(icon_layout(icon, label, theme.text_primary))
            .fill(theme.surface_panel)
            .stroke(Stroke::new(1.0, theme.border_default))
            .min_size(egui::vec2(0.0, 30.0))
            .rounding(Rounding::same(6.0)),
    )
}

pub fn ghost_button(ui: &mut Ui, label: impl Into<RichText>, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(label.into().color(theme.text_secondary))
            .min_size(egui::vec2(0.0, 30.0))
            .rounding(Rounding::same(6.0)),
    )
}

pub fn ghost_button_with_icon(ui: &mut Ui, icon: Icon, label: &str, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(icon_layout(icon, label, theme.text_secondary))
            .min_size(egui::vec2(0.0, 30.0))
            .rounding(Rounding::same(6.0)),
    )
}

pub fn compact_button(ui: &mut Ui, label: impl Into<RichText>, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(label.into().size(12.0).color(theme.text_secondary))
            .min_size(egui::vec2(0.0, 24.0))
            .rounding(Rounding::same(5.0)),
    )
}

pub fn compact_button_with_icon(ui: &mut Ui, icon: Icon, label: &str, theme: DbProTheme) -> Response {
    ui.add(
        Button::new(icon_layout(icon, label, theme.text_secondary))
            .min_size(egui::vec2(0.0, 24.0))
            .rounding(Rounding::same(5.0)),
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
    let (rect, response) = ui.allocate_exact_size(egui::vec2(34.0, 34.0), Sense::click());
    let fill = if active {
        theme.accent_soft
    } else if response.hovered() {
        theme.surface_hover
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, Rounding::same(7.0), fill);
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        char::from(icon).to_string(),
        FontId::new(17.0, FontFamily::Name("lucide".into())),
        if active { theme.accent } else { theme.text_muted },
    );
    response
}

pub fn compact_icon_button(ui: &mut Ui, icon: Icon, theme: DbProTheme) -> Response {
    ui.add_sized(
        [24.0, 24.0],
        Button::new(
            RichText::new(char::from(icon).to_string())
                .font(FontId::new(14.0, FontFamily::Name("lucide".into())))
                .color(theme.text_muted),
        )
        .rounding(Rounding::same(5.0)),
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
        .rounding(Rounding::same(5.0)),
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
