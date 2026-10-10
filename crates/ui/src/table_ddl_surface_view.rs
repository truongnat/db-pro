//! Pure DDL inspection surfaces and typed script intent.

use super::*;
use crate::components::badge::{Badge, BadgeVariant};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use egui::FontId;
use egui::{Align, Layout};
use lucide_icons::Icon;

/// DDL execution from the table editor stays disabled until the feature is explicitly
/// qualified; the supported path remains opening the script in the query editor.
const DDL_APPLY_ENABLED: bool = false;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DdlScriptAction {
    Apply,
    Changed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DdlToolbarAction {
    Refresh,
    OpenInQuery,
    Copy,
}

pub(super) struct DdlToolbarContext {
    pub(super) theme: DbProTheme,
}

pub(super) fn draw_toolbar(context: &DdlToolbarContext, ui: &mut egui::Ui) -> Option<DdlToolbarAction> {
    let mut action = None;
    egui::Frame {
        fill: context.theme.surface_panel,
        inner_margin: egui::Margin::symmetric(SPACE_SM as i8, SPACE_XS as i8),
        stroke: egui::Stroke::new(STROKE_THIN, context.theme.border_subtle),
        corner_radius: egui::CornerRadius::same(RADIUS_SM as u8),
        ..Default::default()
    }
    .show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SPACE_SM;
            ui.label(
                RichText::new(char::from(Icon::Code2).to_string())
                    .font(font_icon(ICON_DEFAULT))
                    .color(context.theme.accent),
            );
            ui.label(RichText::new("DDL SCRIPT").font(font_caption()).strong().color(context.theme.accent));
            Badge::new("CREATE TABLE", context.theme)
                .variant(BadgeVariant::Default)
                .compact(true)
                .show(ui);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if Button::new(context.theme)
                    .icon(Icon::RotateCcw)
                    .text("Refresh DDL")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .tooltip("Re-generate DDL from latest database schema")
                    .show(ui)
                    .clicked()
                {
                    action = Some(DdlToolbarAction::Refresh);
                }
                if Button::new(context.theme)
                    .icon(Icon::Play)
                    .text("Open in Query")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .tooltip("Open DDL in SQL query console")
                    .show(ui)
                    .clicked()
                {
                    action = Some(DdlToolbarAction::OpenInQuery);
                }
                if Button::new(context.theme)
                    .icon(Icon::Copy)
                    .text("Copy DDL")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .tooltip("Copy DDL statement to clipboard")
                    .show(ui)
                    .clicked()
                {
                    action = Some(DdlToolbarAction::Copy);
                }
            });
        });
    });
    action
}

pub(super) struct DdlPlaceholderContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) table_name: &'a str,
    pub(super) error: Option<&'a str>,
}

pub(super) fn draw_placeholder(context: &DdlPlaceholderContext<'_>, ui: &mut egui::Ui) {
    egui::Frame {
        fill: context.theme.surface_panel,
        inner_margin: egui::Margin::symmetric(SPACE_MD as i8, SPACE_SM as i8),
        stroke: egui::Stroke::new(STROKE_THIN, context.theme.border_subtle),
        corner_radius: egui::CornerRadius::same(RADIUS_SM as u8),
        ..Default::default()
    }
    .show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(28.0);
            let failed = context.error.is_some();
            let icon = if failed { Icon::TriangleAlert } else { Icon::Code2 };
            let color = if failed { context.theme.warning } else { context.theme.accent };
            ui.label(RichText::new(char::from(icon).to_string()).font(font_icon(ICON_XL)).color(color));
            ui.add_space(8.0);
            ui.label(
                RichText::new(if failed {
                    format!("DDL for {} could not be loaded", context.table_name)
                } else {
                    format!("Loading DDL for {}…", context.table_name)
                })
                .strong()
                .color(context.theme.text_primary),
            );
            if let Some(error) = context.error {
                ui.label(RichText::new(error).small().color(context.theme.text_secondary));
            }
            ui.add_space(28.0);
        });
    });
}

pub(super) struct DdlScriptContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) writable: bool,
    pub(super) executing: bool,
    pub(super) error: Option<&'a str>,
    pub(super) ddl: &'a mut String,
}

pub(super) fn draw_script_card(context: &mut DdlScriptContext<'_>, ui: &mut egui::Ui) -> Option<DdlScriptAction> {
    let mut action = draw_script_header(context, ui);
    Card::new(context.theme).show(ui, |ui| {
        ui.add_space(8.0);
        draw_script_error(context, ui);
        if let Some(editor_action) = draw_script_editor(context, ui) {
            action = Some(editor_action);
        }
    });
    action
}

fn draw_script_header(context: &DdlScriptContext<'_>, ui: &mut egui::Ui) -> Option<DdlScriptAction> {
    let mut action = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_SM;
        ui.label(RichText::new("CREATE SCRIPT").font(font_caption()).strong().color(context.theme.text_secondary));
        ui.label(
            RichText::new(if context.writable {
                "Editable preview · execute DDL in the query editor (Open in Query)"
            } else {
                "Read-only preview"
            })
            .small()
            .color(context.theme.text_muted),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if context.writable
                && !context.executing
                && Button::new(context.theme)
                    .icon(Icon::Play)
                    .text("Apply DDL")
                    .variant(ButtonVariant::Default)
                    .size(ButtonSize::Sm)
                    .enabled(DDL_APPLY_ENABLED)
                    .tooltip(if DDL_APPLY_ENABLED {
                        "Execute the DDL script"
                    } else {
                        "Not enabled in v0.1 — run DDL with Open in Query"
                    })
                    .show(ui)
                    .clicked()
            {
                action = Some(DdlScriptAction::Apply);
            }
        });
    });
    action
}

fn draw_script_error(context: &DdlScriptContext<'_>, ui: &mut egui::Ui) {
    if let Some(error) = context.error {
        ui.label(
            RichText::new(format!("DDL execution failed · {error}"))
                .small()
                .color(context.theme.danger),
        );
        ui.add_space(6.0);
    }
}

fn draw_script_editor(context: &mut DdlScriptContext<'_>, ui: &mut egui::Ui) -> Option<DdlScriptAction> {
    let mut changed = false;
    editor_frame(context.theme).show(ui, |ui| {
        let response = ui.add(
            TextEdit::multiline(&mut *context.ddl)
                .font(FontId::monospace(13.0))
                .desired_width(ui.available_width())
                .desired_rows(18)
                .interactive(context.writable),
        );
        changed = response.changed();
        // The editor has no visible label widget — name the accesskit node
        // directly so AT announces the control (fixes semantic.missing_name).
        ui.ctx().accesskit_node_builder(response.id, |builder| {
            builder.set_label("DDL script editor");
        });
    });
    changed.then_some(DdlScriptAction::Changed)
}
