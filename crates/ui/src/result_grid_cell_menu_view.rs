//! Result-grid cell context menu and typed user actions.

use super::*;
use lucide_icons::Icon;

pub(super) struct GridCellMenuContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) editable: bool,
    pub(super) write_block_reason: Option<&'a str>,
    pub(super) has_staged_cell: bool,
    pub(super) has_staged_row: bool,
    pub(super) row_deleted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GridCellMenuAction {
    CopyCell,
    CopyRow,
    CopySelectedRows,
    CopySelectedRowsHeaders,
    CopySelectedRowsJson,
    CopySelectedRowsMarkdown,
    CopySelectedRowsInsert,
    CopyJson,
    CopyCsv,
    EditCell,
    SetNull,
    RevertCell,
    RevertRow,
    DuplicateRow,
    DeleteRow,
    FilterThisValue,
    SortAscending,
    SortDescending,
}

pub(super) fn draw_menu(
    context: &GridCellMenuContext<'_>,
    ui: &mut egui::Ui,
    cell_response: &egui::Response,
) -> Option<GridCellMenuAction> {
    let mut action = None;
    let modifier = primary_modifier_label();
    context_action_menu(ui, cell_response, context.theme, |ui, close_menu| {
        macro_rules! menu_item {
            ($icon:expr, $label:expr, $shortcut:expr, $value:expr $(,)?) => {
                if ctx_menu_item(
                    ui,
                    Some($icon),
                    $label,
                    $shortcut,
                    context.theme.text_primary,
                    context.theme,
                )
                .clicked()
                {
                    action = Some($value);
                    *close_menu = true;
                }
            };
        }
        menu_item!(
            Icon::Copy,
            "Copy Cell Value",
            Some(&format!("{modifier}C")),
            GridCellMenuAction::CopyCell,
        );
        menu_item!(
            Icon::Table2,
            "Copy Row (TSV)",
            Some(&format!("{modifier}Shift+C")),
            GridCellMenuAction::CopyRow,
        );
        menu_item!(
            Icon::Rows3,
            "Copy Selected Rows",
            None,
            GridCellMenuAction::CopySelectedRows,
        );
        menu_item!(
            Icon::Table2,
            "Copy Selected Rows with Headers",
            None,
            GridCellMenuAction::CopySelectedRowsHeaders,
        );
        menu_item!(
            Icon::Braces,
            "Copy Selected Rows as JSON",
            None,
            GridCellMenuAction::CopySelectedRowsJson,
        );
        menu_item!(
            Icon::FileText,
            "Copy Selected Rows as Markdown",
            None,
            GridCellMenuAction::CopySelectedRowsMarkdown,
        );
        menu_item!(
            Icon::Code,
            "Copy Selected Rows as INSERT SQL",
            None,
            GridCellMenuAction::CopySelectedRowsInsert,
        );
        menu_item!(Icon::Braces, "Copy Row as JSON", None, GridCellMenuAction::CopyJson);
        menu_item!(
            Icon::FileSpreadsheet,
            "Copy Row as CSV",
            None,
            GridCellMenuAction::CopyCsv,
        );

        if context.editable {
            ui.separator();
            if let Some(reason) = context.write_block_reason {
                ctx_menu_item(
                    ui,
                    Some(Icon::Lock),
                    "Read-only Column",
                    None,
                    context.theme.text_muted,
                    context.theme,
                )
                .on_hover_text(reason);
            } else {
                menu_item!(
                    Icon::Pencil,
                    "Edit Cell",
                    Some("Enter / F2"),
                    GridCellMenuAction::EditCell,
                );
                menu_item!(Icon::Eraser, "Set to NULL", None, GridCellMenuAction::SetNull);
            }
            if context.has_staged_cell {
                menu_item!(Icon::Undo2, "Revert Cell", None, GridCellMenuAction::RevertCell);
            }
            if context.has_staged_row {
                menu_item!(
                    Icon::Undo2,
                    if context.row_deleted {
                        "Undo Delete"
                    } else {
                        "Revert Row"
                    },
                    None,
                    GridCellMenuAction::RevertRow,
                );
            }
            menu_item!(Icon::CopyPlus, "Duplicate Row", None, GridCellMenuAction::DuplicateRow);
            menu_item!(
                Icon::Trash2,
                "Delete Row",
                Some("Delete / Backspace"),
                GridCellMenuAction::DeleteRow,
            );
        }

        ui.separator();
        menu_item!(
            Icon::Filter,
            "Filter by this value",
            None,
            GridCellMenuAction::FilterThisValue,
        );
        menu_item!(Icon::ArrowUp, "Sort Ascending", None, GridCellMenuAction::SortAscending);
        menu_item!(
            Icon::ArrowDown,
            "Sort Descending",
            None,
            GridCellMenuAction::SortDescending,
        );
    });
    action
}

fn primary_modifier_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "⌘"
    } else {
        "Ctrl"
    }
}
