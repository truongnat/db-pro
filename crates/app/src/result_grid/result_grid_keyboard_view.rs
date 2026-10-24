use super::*;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct GridKeyboardCommands {
    pub(super) copy_selected_rows: bool,
    pub(super) copy_selected_cell: bool,
    pub(super) apply_staged_changes: bool,
    pub(super) discard_staged_changes: bool,
    pub(super) delete_selected_rows: bool,
    pub(super) pasted_text: Option<String>,
    pub(super) commit_edit_and_navigate: bool,
    pub(super) navigate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum GridKeyboardIntent {
    SelectAll,
    ClearSelection,
    Commands(GridKeyboardCommands),
}

#[derive(Clone, Copy)]
pub(super) struct GridKeyboardInputContext {
    pub(super) editable: bool,
    pub(super) editing_cell: bool,
    pub(super) connection_dialog_open: bool,
}

/// Focus id for the grid drawn inside `ui`. Scoped to the grid's own `Ui` via
/// `unique_id` (stable across frames, distinct per ui subtree) so the table
/// workspace grid and the query output-dock grid can coexist in one frame
/// without an egui "Second use of widget ID" collision (#UAT-2).
pub(super) fn result_grid_focus_id(ui: &egui::Ui) -> egui::Id {
    ui.unique_id().with("db-pro-result-grid")
}

/// The grid handles keys when nothing else is focused, and when the focus command
/// has moved focus onto the grid itself. A text field still wins.
pub(super) fn grid_accepts_keyboard(ui: &egui::Ui) -> bool {
    match ui.ctx().memory(|memory| memory.focused()) {
        None => true,
        Some(id) => id == result_grid_focus_id(ui),
    }
}

/// Context temp-data key holding the output-dock grid's focus id, refreshed
/// each frame the docked query grid draws. Lets `handle_shortcuts` request
/// focus without access to that grid's `Ui`.
fn query_result_grid_focus_key() -> egui::Id {
    egui::Id::new("db-pro-query-result-grid-focus")
}

pub(super) fn record_query_result_grid_focus_id(ui: &egui::Ui) {
    let id = result_grid_focus_id(ui);
    ui.ctx()
        .data_mut(|data| data.insert_temp(query_result_grid_focus_key(), id));
}

/// Focus id of the output-dock grid from the last frame it drew.
pub(super) fn query_result_grid_focus_id(ctx: &egui::Context) -> Option<egui::Id> {
    ctx.data(|data| data.get_temp::<egui::Id>(query_result_grid_focus_key()))
}

pub(super) fn read_keyboard_intent(ui: &egui::Ui, context: GridKeyboardInputContext) -> GridKeyboardIntent {
    if grid_accepts_keyboard(ui) && ui.input(|input| input.key_pressed(egui::Key::A) && primary_modifier_pressed(input))
    {
        return GridKeyboardIntent::SelectAll;
    }

    if grid_accepts_keyboard(ui) && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        return GridKeyboardIntent::ClearSelection;
    }

    let mut commands = read_grid_commands(ui, context);
    commands.pasted_text = ui.input(|input| {
        input.events.iter().find_map(|event| match event {
            egui::Event::Paste(text) => Some(text.clone()),
            _ => None,
        })
    });

    if context.editable && context.editing_cell && ui.input(|input| input.key_pressed(egui::Key::Tab)) {
        commands.commit_edit_and_navigate = true;
        return GridKeyboardIntent::Commands(commands);
    }
    commands.navigate = grid_accepts_keyboard(ui);
    GridKeyboardIntent::Commands(commands)
}

// cc-scan:allow HARD_COMPLEXITY — key→grid-command dispatch ladder
fn read_grid_commands(ui: &egui::Ui, context: GridKeyboardInputContext) -> GridKeyboardCommands {
    let modifier = ui.input(primary_modifier_pressed);
    let shift = ui.input(|input| input.modifiers.shift);
    GridKeyboardCommands {
        copy_selected_rows: grid_accepts_keyboard(ui)
            && !context.connection_dialog_open
            && ui.input(|input| input.key_pressed(egui::Key::C))
            && modifier
            && shift,
        copy_selected_cell: grid_accepts_keyboard(ui)
            && !context.connection_dialog_open
            && ui.input(|input| input.key_pressed(egui::Key::C))
            && modifier
            && !shift,
        apply_staged_changes: grid_accepts_keyboard(ui)
            && !context.connection_dialog_open
            && ui.input(|input| input.key_pressed(egui::Key::S) && primary_modifier_pressed(input)),
        discard_staged_changes: grid_accepts_keyboard(ui)
            && !context.connection_dialog_open
            && ui.input(|input| input.key_pressed(egui::Key::Z) && primary_modifier_pressed(input)),
        delete_selected_rows: context.editable
            && !context.editing_cell
            && grid_accepts_keyboard(ui)
            && !context.connection_dialog_open
            && ui.input(|input| input.key_pressed(egui::Key::Delete) || input.key_pressed(egui::Key::Backspace)),
        ..Default::default()
    }
}

fn primary_modifier_pressed(input: &egui::InputState) -> bool {
    input.modifiers.command || input.modifiers.ctrl || input.modifiers.mac_cmd
}
