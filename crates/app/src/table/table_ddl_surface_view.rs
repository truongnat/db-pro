//! Pure DDL inspection surfaces and typed script intent.

use super::*;
use crate::components::badge::{Badge, BadgeVariant};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use egui::FontId;
use egui::{Align, Layout};
use lucide_icons::Icon;

/// DDL execution from the table editor is gated behind the confirmation card —
/// Apply raises `ddl_execute_confirmation`, never dispatches directly (#361).
const DDL_APPLY_ENABLED: bool = true;

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
    panel_frame(context.theme).show(ui, |ui| {
        let narrow = ui.available_width() < 720.0;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = SPACE_SM;
            ui.label(
                RichText::new(char::from(Icon::Code2).to_string())
                    .font(font_icon(ICON_DEFAULT))
                    .color(context.theme.accent),
            );
            ui.label(
                RichText::new("DDL SCRIPT")
                    .font(font_caption())
                    .strong()
                    .color(context.theme.accent),
            );
            Badge::new("CREATE TABLE", context.theme)
                .variant(BadgeVariant::Default)
                .compact(true)
                .show(ui);
            if !narrow {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    action = draw_toolbar_actions(context, ui);
                });
            }
        });
        if narrow {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(SPACE_SM, SPACE_XS);
                action = draw_toolbar_actions(context, ui);
            });
        }
    });
    action
}

fn draw_toolbar_actions(context: &DdlToolbarContext, ui: &mut egui::Ui) -> Option<DdlToolbarAction> {
    let mut action = None;
    for (icon, label, variant, tooltip, intent) in [
        (
            Icon::RotateCcw,
            t!("table.refresh_ddl"),
            ButtonVariant::Ghost,
            "Re-generate DDL from latest database schema",
            DdlToolbarAction::Refresh,
        ),
        (
            Icon::Play,
            t!("table.open_in_query"),
            ButtonVariant::Secondary,
            "Open DDL in SQL query console",
            DdlToolbarAction::OpenInQuery,
        ),
        (
            Icon::Copy,
            t!("table.copy_ddl"),
            ButtonVariant::Ghost,
            "Copy DDL statement to clipboard",
            DdlToolbarAction::Copy,
        ),
    ] {
        if Button::new(context.theme)
            .icon(icon)
            .text(label)
            .variant(variant)
            .size(ButtonSize::Sm)
            .tooltip(tooltip)
            .show(ui)
            .clicked()
        {
            action = Some(intent);
        }
    }
    action
}

pub(super) struct DdlPlaceholderContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) table_name: &'a str,
    pub(super) error: Option<&'a str>,
}

pub(super) fn draw_placeholder(context: &DdlPlaceholderContext<'_>, ui: &mut egui::Ui) {
    panel_frame(context.theme).show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(28.0);
            let failed = context.error.is_some();
            let icon = if failed { Icon::TriangleAlert } else { Icon::Code2 };
            let color = if failed {
                context.theme.warning
            } else {
                context.theme.accent
            };
            ui.label(
                RichText::new(char::from(icon).to_string())
                    .font(font_icon(ICON_XL))
                    .color(color),
            );
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
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_SM;
        ui.label(
            RichText::new("CREATE SCRIPT")
                .font(font_caption())
                .strong()
                .color(context.theme.text_secondary),
        );
        ui.label(
            RichText::new(if context.writable {
                "Editable preview · Apply asks for confirmation"
            } else {
                "Read-only preview"
            })
            .small()
            .color(context.theme.text_muted),
        );
        if context.writable
            && !context.executing
            && Button::new(context.theme)
                .icon(Icon::Play)
                .text(t!("table.apply_ddl"))
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .enabled(DDL_APPLY_ENABLED)
                .tooltip("Review and confirm before the DDL executes")
                .show(ui)
                .clicked()
        {
            action = Some(DdlScriptAction::Apply);
        }
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
        let response = crate::components::Textarea::new(context.ddl, "DDL script…", context.theme)
            .font(FontId::monospace(13.0))
            .min_rows(18)
            .read_only(!context.writable)
            .show(ui);
        changed = response.changed();
        // The editor has no visible label widget — name the accesskit node
        // directly so AT announces the control (fixes semantic.missing_name).
        ui.ctx().accesskit_node_builder(response.id, |builder| {
            builder.set_label("DDL script editor");
        });
    });
    changed.then_some(DdlScriptAction::Changed)
}

#[cfg(test)]
#[path = "tests/table_ddl_surface_view_tests.rs"]
mod layout_tests;
