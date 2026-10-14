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

pub(super) fn result_grid_focus_id() -> egui::Id {
    egui::Id::new("db-pro-result-grid")
}

/// The grid handles keys when nothing else is focused, and when the focus command
/// has moved focus onto the grid itself. A text field still wins.
pub(super) fn grid_accepts_keyboard(ctx: &egui::Context) -> bool {
    match ctx.memory(|memory| memory.focused()) {
        None => true,
        Some(id) => id == result_grid_focus_id(),
    }
}

pub(super) fn read_keyboard_intent(ui: &egui::Ui, context: GridKeyboardInputContext) -> GridKeyboardIntent {
    if grid_accepts_keyboard(ui.ctx())
        && ui.input(|input| input.key_pressed(egui::Key::A) && primary_modifier_pressed(input))
    {
        return GridKeyboardIntent::SelectAll;
    }

    if grid_accepts_keyboard(ui.ctx()) && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
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
    commands.navigate = grid_accepts_keyboard(ui.ctx());
    GridKeyboardIntent::Commands(commands)
}

// cc-scan:allow HARD_COMPLEXITY — key→grid-command dispatch ladder
fn read_grid_commands(ui: &egui::Ui, context: GridKeyboardInputContext) -> GridKeyboardCommands {
    let modifier = ui.input(primary_modifier_pressed);
    let shift = ui.input(|input| input.modifiers.shift);
    GridKeyboardCommands {
        copy_selected_rows: grid_accepts_keyboard(ui.ctx())
            && !context.connection_dialog_open
            && ui.input(|input| input.key_pressed(egui::Key::C))
            && modifier
            && shift,
        copy_selected_cell: grid_accepts_keyboard(ui.ctx())
            && !context.connection_dialog_open
            && ui.input(|input| input.key_pressed(egui::Key::C))
            && modifier
            && !shift,
        apply_staged_changes: grid_accepts_keyboard(ui.ctx())
            && !context.connection_dialog_open
            && ui.input(|input| input.key_pressed(egui::Key::S) && primary_modifier_pressed(input)),
        discard_staged_changes: grid_accepts_keyboard(ui.ctx())
            && !context.connection_dialog_open
            && ui.input(|input| input.key_pressed(egui::Key::Z) && primary_modifier_pressed(input)),
        delete_selected_rows: context.editable
            && !context.editing_cell
            && grid_accepts_keyboard(ui.ctx())
            && !context.connection_dialog_open
            && ui.input(|input| input.key_pressed(egui::Key::Delete) || input.key_pressed(egui::Key::Backspace)),
        ..Default::default()
    }
}

fn primary_modifier_pressed(input: &egui::InputState) -> bool {
    input.modifiers.command || input.modifiers.ctrl || input.modifiers.mac_cmd
}
