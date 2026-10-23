// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
//! Inline result-cell editor presentation and typed keyboard actions.
use super::super::*;
use super::result_grid_date_picker_view;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CellEditorAction {
    Commit,
    Cancel,
}

/// Editor widget chosen from the column's data type: plain input for
/// text/number/…, a select for boolean and enum-like columns, and an input
/// plus calendar button for date/datetime columns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CellEditorKind {
    Text,
    Boolean { nullable: bool },
    Select(Vec<String>),
    Temporal { date_only: bool },
}

pub(super) struct CellEditorContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) kind: CellEditorKind,
    pub(super) value: &'a mut String,
    pub(super) error: &'a mut Option<String>,
}

pub(super) fn draw(
    context: &mut CellEditorContext<'_>,
    ui: &mut egui::Ui,
    cell_rect: egui::Rect,
) -> Option<CellEditorAction> {
    let response = draw_input(context, ui, cell_rect);
    // Autofocus applies to the plain input only; the temporal editor manages
    // focus itself so an open calendar is not focus-starved each frame.
    if !response.has_focus() && matches!(context.kind, CellEditorKind::Text) {
        response.request_focus();
    }
    if response.changed() {
        *context.error = None;
    }
    keyboard_action(ui)
}

fn draw_input(context: &mut CellEditorContext<'_>, ui: &mut egui::Ui, cell_rect: egui::Rect) -> egui::Response {
    // `new_child` (not `allocate_new_ui`): the cell's space is already
    // reserved, and nothing the editor contains may grow the row layout —
    // `allocate_new_ui` would feed the child's min_rect back into the
    // row's horizontal placer, which can push the row taller.
    let mut editor_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(cell_rect.shrink(2.0))
            .layout(egui::Layout::top_down(egui::Align::LEFT)),
    );
    editor_ui.set_clip_rect(editor_ui.clip_rect().intersect(cell_rect));
    let ui = &mut editor_ui;
    let theme = context.theme;
    match &context.kind {
        CellEditorKind::Boolean { nullable } => {
            let options = boolean_options(*nullable);
            draw_select(
                ui,
                theme,
                context.value,
                context.error,
                ui.id().with("bool_select"),
                &options,
            )
        }
        CellEditorKind::Select(values) => {
            let options: Vec<(String, String)> = values.iter().map(|v| (v.clone(), v.clone())).collect();
            draw_select(
                ui,
                theme,
                context.value,
                context.error,
                ui.id().with("enum_select"),
                &options,
            )
        }
        CellEditorKind::Temporal { date_only } => {
            draw_temporal(ui, theme, context.value, context.error, *date_only, cell_rect)
        }
        CellEditorKind::Text => draw_text_input(ui, theme, context.value, ui.available_size()),
    }
}

fn draw_text_input(ui: &mut egui::Ui, theme: DbProTheme, value: &mut String, size: egui::Vec2) -> egui::Response {
    let response = ui.add_sized(
        size,
        egui::TextEdit::singleline(value) // ui-rules: in-cell grid editor must stay chromeless (Frame::NONE, add_sized); Input owns label/field chrome
            .frame(egui::Frame::NONE)
            .margin(egui::Margin::symmetric(6.0 as i8, 2.0 as i8))
            .text_color(theme.text_primary),
    );
    // The active editor is the only text-entry surface in the grid, so it is
    // the only place that shows an I-beam.
    if ui.is_enabled() {
        response.on_hover_cursor(egui::CursorIcon::Text)
    } else {
        response
    }
}

/// `(stored_value, label)` pairs for the boolean select. `NULL` is only
/// offered when the column is nullable.
fn boolean_options(nullable: bool) -> Vec<(String, String)> {
    let mut options = vec![
        ("true".to_owned(), "TRUE".to_owned()),
        ("false".to_owned(), "FALSE".to_owned()),
    ];
    if nullable {
        options.push(("NULL".to_owned(), "NULL".to_owned()));
    }
    options
}

fn draw_select(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    value: &mut String,
    error: &mut Option<String>,
    id: egui::Id,
    options: &[(String, String)],
) -> egui::Response {
    let labels: Vec<String> = options.iter().map(|(_, label)| label.clone()).collect();
    let mut selected = options
        .iter()
        .position(|(stored, _)| stored.eq_ignore_ascii_case(value.as_str()))
        .unwrap_or(usize::MAX);
    // Show the matched label (or the raw stored value) on the trigger even
    // though `Select` bookkeeping is index-based.
    let selected_text = options
        .iter()
        .find(|(stored, _)| stored.eq_ignore_ascii_case(value.as_str()))
        .map(|(_, label)| label.clone())
        .unwrap_or_else(|| value.clone());
    let idx_before = selected;
    let response = crate::components::Select::new(id, &mut selected, &labels)
        .theme(theme)
        .size(crate::components::SelectSize::Sm)
        .width(ui.available_width())
        .selected_text(selected_text)
        .show(ui);
    if selected != idx_before && selected < options.len() {
        *value = options[selected].0.clone();
        *error = None;
    }
    response
}

fn draw_temporal(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    value: &mut String,
    error: &mut Option<String>,
    date_only: bool,
    cell_rect: egui::Rect,
) -> egui::Response {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
        let button_width = 24.0;
        let input = draw_text_input(
            ui,
            theme,
            value,
            egui::vec2((ui.available_width() - button_width).max(40.0), ui.available_height()),
        );
        let button = Button::new(theme)
            .icon(Icon::Calendar)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .access_label("Open date picker")
            .show(ui);
        let popup_id = ui.id().with("date_picker_popup");
        if button.clicked() {
            egui::Popup::toggle_id(ui.ctx(), popup_id);
        }
        // DB_PRO_CAPTURE_PICKER keeps the calendar open so captures can
        // document the date editor. Read once; zero cost when unset.
        if picker_open_override() {
            egui::Popup::open_id(ui.ctx(), popup_id);
        }
        // Autofocus the text field, but never steal focus while the calendar
        // is open — that would fight the button and the day cells.
        if !input.has_focus() && !egui::Popup::is_id_open(ui.ctx(), popup_id) {
            input.request_focus();
        }
        let mut picked = false;
        let mut close_popup = false;
        let mut escape_pressed = false;
        if egui::Popup::is_id_open(ui.ctx(), popup_id) {
            let popup_pos = Calendar::popup_position(cell_rect, ui.ctx().content_rect(), true);
            let popup = FloatingPopup::new(popup_id.with("area"), theme, popup_pos)
                .order(egui::Order::Foreground)
                .frameless()
                .show(ui.ctx(), |ui| {
                    result_grid_date_picker_view::draw_calendar(ui, theme, date_only, value)
                });
            picked = popup.inner;
            let pointer_clicked_outside = ui.input(|input| {
                input.pointer.any_click()
                    && input.pointer.interact_pos().is_some_and(|position| {
                        !cell_rect.contains(position) && !popup.response.rect.contains(position)
                    })
            });
            escape_pressed = ui.input(|input| input.key_pressed(egui::Key::Escape));
            close_popup = picked || pointer_clicked_outside || escape_pressed;
        }
        if close_popup {
            egui::Popup::close_all(ui.ctx());
        }
        if picked || escape_pressed {
            input.request_focus();
        }
        if picked {
            *error = None;
        }
        input.union(button)
    })
    .inner
}

fn picker_open_override() -> bool {
    static FLAG: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *FLAG.get_or_init(|| std::env::var_os("DB_PRO_CAPTURE_PICKER").is_some())
}

fn keyboard_action(ui: &egui::Ui) -> Option<CellEditorAction> {
    if ui.input(|input| input.key_pressed(egui::Key::Enter)) {
        Some(CellEditorAction::Commit)
    } else if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        Some(CellEditorAction::Cancel)
    } else {
        None
    }
}

#[cfg(test)]
#[path = "tests/result_grid_cell_editor_surface_view_tests.rs"]
mod tests;

