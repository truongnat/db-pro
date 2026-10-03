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
#[derive(Debug, PartialEq, Eq)]
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
    // 2pt inset: the focused input stroke is centered on its rect edge, so a
    // 1pt inset would let it bleed 0.5pt over the cell border.
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(cell_rect.shrink(2.0)), |ui| {
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
                draw_temporal(ui, theme, context.value, context.error, *date_only)
            }
            CellEditorKind::Text => draw_text_input(ui, theme, context.value, ui.available_size()),
        }
    })
    .inner
}

fn draw_text_input(ui: &mut egui::Ui, theme: DbProTheme, value: &mut String, size: egui::Vec2) -> egui::Response {
    ui.add_sized(
        size,
        egui::TextEdit::singleline(value)
            // no frame: the editor should read as the cell's own text, not a
            // bordered widget floating on top of the grid
            .frame(false)
            .margin(egui::Margin::symmetric(6.0, 2.0))
            .text_color(theme.text_primary),
    )
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
    let selected = options
        .iter()
        .find(|(stored, _)| stored.eq_ignore_ascii_case(value.as_str()))
        .map(|(_, label)| label.clone())
        .unwrap_or_else(|| value.clone());
    egui::ComboBox::from_id_salt(id)
        .width(ui.available_width())
        .selected_text(RichText::new(selected).color(theme.text_primary).size(12.0))
        .show_ui(ui, |ui| {
            for (stored, label) in options {
                if ui
                    .selectable_label(stored.eq_ignore_ascii_case(value.as_str()), label)
                    .clicked()
                {
                    *value = stored.clone();
                    *error = None;
                }
            }
        })
        .response
}

fn draw_temporal(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    value: &mut String,
    error: &mut Option<String>,
    date_only: bool,
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
        let button = ui.add_sized(
            egui::vec2(button_width, ui.available_height()),
            egui::Button::new(icon_text(Icon::Calendar, "", theme.text_muted)).frame(false),
        );
        let popup_id = ui.id().with("date_picker_popup");
        if button.clicked() {
            ui.memory_mut(|memory| memory.toggle_popup(popup_id));
        }
        // DB_PRO_CAPTURE_PICKER keeps the calendar open so captures can
        // document the date editor. Read once; zero cost when unset.
        if picker_open_override() {
            ui.memory_mut(|memory| memory.open_popup(popup_id));
        }
        let editor_area = input.union(button);
        // Autofocus the text field, but never steal focus while the calendar
        // is open — that would fight the button and the day cells.
        if !input.has_focus() && !ui.memory(|memory| memory.is_popup_open(popup_id)) {
            input.request_focus();
        }
        // The anchor is the whole editor area so clicks into the text input do
        // not count as "clicked elsewhere" and collapse the calendar.
        let picked = egui::popup::popup_below_widget(
            ui,
            popup_id,
            &editor_area,
            egui::PopupCloseBehavior::CloseOnClickOutside,
            |ui| result_grid_date_picker_view::draw_calendar(ui, theme, date_only, value),
        );
        if picked == Some(true) {
            ui.memory_mut(|memory| memory.close_popup());
            *error = None;
        }
        editor_area
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
