//! Result-grid toolbar rendering and user intent collection.
use super::*;
use egui::{Align, Layout};
use lucide_icons::Icon;

pub(super) struct ResultGridToolbarContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) data: &'a mut TableDataState,
    pub(super) editing: &'a mut TableEditingState,
    pub(super) feedback: &'a FeedbackState,
    pub(super) editable: bool,
    pub(super) matching_rows: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ResultGridToolbarAction {
    CopySelectedCell,
    CopySelectedRow,
    CopyVisibleCsv,
    CopyVisibleJson,
    CopyVisibleMarkdown,
    CopyVisibleInsert,
    InspectSelectedCell { row_index: usize, column_index: usize },
}

pub(super) fn draw_toolbar(
    context: &mut ResultGridToolbarContext<'_>,
    ui: &mut egui::Ui,
) -> Option<ResultGridToolbarAction> {
    let mut action = None;
    toolbar_frame(context.theme).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            draw_filter_controls(context, ui);
            ui.separator();
            draw_copy_actions(context, ui, &mut action);
            draw_inspect_actions(context, ui, &mut action);
            if !context.feedback.copy_status.is_empty() {
                crate::components::badge::Badge::new(&context.feedback.copy_status, context.theme)
                    .variant(crate::components::badge::BadgeVariant::Success)
                    .compact(true)
                    .show(ui);
            }
        });
    });
    ui.add_space(4.0);
    action
}

fn draw_filter_controls(context: &mut ResultGridToolbarContext<'_>, ui: &mut egui::Ui) {
    input(
        ui,
        &mut context.data.grid_filter,
        "Filter visible rows…",
        160.0,
        context.theme,
    );
    if !context.data.grid_filter.is_empty()
        && Button::new(context.theme)
            .icon(Icon::X)
            .access_label("Clear filter")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .tooltip("Clear filter")
            .show(ui)
            .clicked()
    {
        context.data.grid_filter.clear();
    }
    crate::components::badge::Badge::new(format!("{} rows", context.matching_rows), context.theme)
        .variant(crate::components::badge::BadgeVariant::Secondary)
        .compact(true)
        .show(ui);
}

fn draw_copy_actions(
    context: &mut ResultGridToolbarContext<'_>,
    ui: &mut egui::Ui,
    action: &mut Option<ResultGridToolbarAction>,
) {
    let modifier = primary_modifier_label();
    icon_action(
        context,
        ui,
        Icon::Copy,
        &format!("Copy selected cell ({modifier}C)"),
        ResultGridToolbarAction::CopySelectedCell,
        action,
    );
    icon_action(
        context,
        ui,
        Icon::Table2,
        &format!("Copy selected row as TSV ({modifier}Shift+C)"),
        ResultGridToolbarAction::CopySelectedRow,
        action,
    );

    let menu_button = Button::new(context.theme)
        .text("Copy as")
        .icon(Icon::ChevronDown)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .access_label("Copy visible rows in another format")
        .tooltip("Copy visible rows as CSV, JSON, Markdown or SQL INSERT")
        .show(ui);
    egui::popup::popup_below_widget(
        ui,
        menu_button.id.with("copy_formats"),
        &menu_button,
        egui::PopupCloseBehavior::CloseOnClick,
        |ui| {
            ui.set_min_width(180.0);
            for (icon, label, next) in [
                (Icon::FileSpreadsheet, "CSV", ResultGridToolbarAction::CopyVisibleCsv),
                (Icon::Braces, "JSON", ResultGridToolbarAction::CopyVisibleJson),
                (Icon::FileText, "Markdown", ResultGridToolbarAction::CopyVisibleMarkdown),
                (Icon::Database, "SQL INSERT", ResultGridToolbarAction::CopyVisibleInsert),
            ] {
                if menu_button_with_icon(ui, icon, label, context.theme).clicked() {
                    *action = Some(next);
                }
            }
        },
    );
}

fn draw_inspect_actions(
    context: &mut ResultGridToolbarContext<'_>,
    ui: &mut egui::Ui,
    action: &mut Option<ResultGridToolbarAction>,
) {
    if Button::new(context.theme)
        .icon(Icon::PanelRight)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::IconSm)
        .access_label("Toggle record inspector")
        .tooltip("Toggle record / value inspector panel")
        .show(ui)
        .clicked()
    {
        context.editing.record_inspector_open = !context.editing.record_inspector_open;
    }
    if let Some((row_index, column_index)) = context.data.selected_cell {
        if Button::new(context.theme)
            .icon(Icon::ScanSearch)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .access_label("Inspect selected cell")
            .tooltip("Open advanced value inspector for the selected cell")
            .show(ui)
            .clicked()
        {
            *action = Some(ResultGridToolbarAction::InspectSelectedCell {
                row_index,
                column_index,
            });
        }
    }

    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        Button::new(context.theme)
            .icon(Icon::Info)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .access_label("Result grid help")
            .tooltip(if context.editable {
                "Double-click / Enter to edit\nRight-click for actions\nDrag divider to resize"
            } else {
                "Click cell to select\nRight-click for actions\nDrag divider to resize"
            })
            .show(ui);
    });
}

fn icon_action(
    context: &ResultGridToolbarContext<'_>,
    ui: &mut egui::Ui,
    icon: Icon,
    tooltip: &str,
    next: ResultGridToolbarAction,
    action: &mut Option<ResultGridToolbarAction>,
) {
    if Button::new(context.theme)
        .icon(icon)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::IconSm)
        .access_label(tooltip)
        .tooltip(tooltip)
        .show(ui)
        .clicked()
    {
        *action = Some(next);
    }
}

fn primary_modifier_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "⌘"
    } else {
        "Ctrl"
    }
}
