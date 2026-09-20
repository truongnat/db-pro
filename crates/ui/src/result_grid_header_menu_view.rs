//! Column-header context menu and typed actions.

use super::*;
use lucide_icons::Icon;

pub(super) struct GridHeaderMenuContext {
    pub(super) theme: DbProTheme,
    pub(super) visual_index: usize,
    pub(super) column_count: usize,
    pub(super) column_index: usize,
    pub(super) is_sorted: bool,
    pub(super) table_data_active: bool,
    pub(super) has_hidden_columns: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GridHeaderMenuAction {
    Sort(Option<bool>),
    AddFilter,
    MoveLeft(usize),
    MoveRight(usize),
    ResetOrder,
    ResetWidths,
    HideColumn(usize),
    ShowColumns,
    ResetLayout,
    AutoSize(usize),
}

pub(super) fn draw_menu(
    context: &GridHeaderMenuContext,
    ui: &mut egui::Ui,
    response: &egui::Response,
) -> Option<GridHeaderMenuAction> {
    let mut action = None;
    context_action_menu(ui, response, context.theme, |ui, close_menu| {
        if ctx_menu_item(
            ui,
            Some(Icon::ArrowUp),
            "Sort Ascending (A → Z)",
            None,
            context.theme.text_primary,
            context.theme,
        )
        .clicked()
        {
            action = Some(GridHeaderMenuAction::Sort(Some(false)));
            *close_menu = true;
        }
        if ctx_menu_item(
            ui,
            Some(Icon::ArrowDown),
            "Sort Descending (Z → A)",
            None,
            context.theme.text_primary,
            context.theme,
        )
        .clicked()
        {
            action = Some(GridHeaderMenuAction::Sort(Some(true)));
            *close_menu = true;
        }
        if context.is_sorted
            && ctx_menu_item(
                ui,
                Some(Icon::X),
                "Clear Sort",
                None,
                context.theme.text_secondary,
                context.theme,
            )
            .clicked()
        {
            action = Some(GridHeaderMenuAction::Sort(None));
            *close_menu = true;
        }
        if context.table_data_active
            && ctx_menu_item(
                ui,
                Some(Icon::Filter),
                "Add Filter",
                None,
                context.theme.text_primary,
                context.theme,
            )
            .clicked()
        {
            action = Some(GridHeaderMenuAction::AddFilter);
            *close_menu = true;
        }
        ui.separator();
        if context.visual_index > 0
            && ctx_menu_item(
                ui,
                Some(Icon::ArrowLeft),
                "Move Column Left",
                None,
                context.theme.text_primary,
                context.theme,
            )
            .clicked()
        {
            action = Some(GridHeaderMenuAction::MoveLeft(context.visual_index));
            *close_menu = true;
        }
        if context.visual_index + 1 < context.column_count
            && ctx_menu_item(
                ui,
                Some(Icon::ArrowRight),
                "Move Column Right",
                None,
                context.theme.text_primary,
                context.theme,
            )
            .clicked()
        {
            action = Some(GridHeaderMenuAction::MoveRight(context.visual_index));
            *close_menu = true;
        }
        ui.separator();
        if ctx_menu_item(
            ui,
            Some(Icon::RotateCcw),
            "Reset Column Order",
            None,
            context.theme.text_secondary,
            context.theme,
        )
        .clicked()
        {
            action = Some(GridHeaderMenuAction::ResetOrder);
            *close_menu = true;
        }
        if ctx_menu_item(
            ui,
            Some(Icon::Maximize2),
            "Reset Column Widths",
            None,
            context.theme.text_secondary,
            context.theme,
        )
        .clicked()
        {
            action = Some(GridHeaderMenuAction::ResetWidths);
            *close_menu = true;
        }
        if ctx_menu_item(
            ui,
            Some(Icon::EyeOff),
            "Hide Column",
            None,
            context.theme.text_secondary,
            context.theme,
        )
        .clicked()
        {
            action = Some(GridHeaderMenuAction::HideColumn(context.column_index));
            *close_menu = true;
        }
        if context.has_hidden_columns
            && ctx_menu_item(
                ui,
                Some(Icon::Eye),
                "Show Columns",
                None,
                context.theme.text_secondary,
                context.theme,
            )
            .clicked()
        {
            action = Some(GridHeaderMenuAction::ShowColumns);
            *close_menu = true;
        }
        if ctx_menu_item(
            ui,
            Some(Icon::RotateCcw),
            "Reset Layout",
            None,
            context.theme.text_secondary,
            context.theme,
        )
        .clicked()
        {
            action = Some(GridHeaderMenuAction::ResetLayout);
            *close_menu = true;
        }
        if ctx_menu_item(
            ui,
            Some(Icon::Ruler),
            "Auto Size",
            None,
            context.theme.text_secondary,
            context.theme,
        )
        .clicked()
        {
            action = Some(GridHeaderMenuAction::AutoSize(context.column_index));
            *close_menu = true;
        }
    });
    action
}
