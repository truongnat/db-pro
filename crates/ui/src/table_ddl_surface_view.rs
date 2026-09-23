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
    toolbar_frame(context.theme).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(icon_text(Icon::Code2, "DDL SCRIPT", context.theme.accent));
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
    grid_frame(context.theme).show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(28.0);
            let failed = context.error.is_some();
            ui.label(icon_text(
                if failed { Icon::TriangleAlert } else { Icon::Code2 },
                "",
                if failed {
                    context.theme.warning
                } else {
                    context.theme.accent
                },
            ));
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
    card_frame(context.theme).show(ui, |ui| {
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
        section_label(ui, "CREATE SCRIPT", context.theme);
        ui.label(
            RichText::new(if context.writable {
                "Editable preview · execute DDL in the query editor (Open in Query)"
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
        changed = ui
            .add(
                TextEdit::multiline(&mut *context.ddl)
                    .font(FontId::monospace(13.0))
                    .desired_width(ui.available_width())
                    .desired_rows(18)
                    .interactive(context.writable),
            )
            .changed();
    });
    changed.then_some(DdlScriptAction::Changed)
}
