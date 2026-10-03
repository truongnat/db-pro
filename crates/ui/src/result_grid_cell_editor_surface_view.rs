//! Inline result-cell editor presentation and typed keyboard actions.
use super::super::*;
use super::result_grid_date_picker_view;

/// Upper bound for the date-picker popup body so its desired size can never
/// feed back into the persisted `Area` size.
const POPUP_MAX_WIDTH: f32 = 200.0;

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
        // Autofocus the text field, but never steal focus while the calendar
        // is open — that would fight the button and the day cells.
        if !input.has_focus() && !ui.memory(|memory| memory.is_popup_open(popup_id)) {
            input.request_focus();
        }
        // Anchor on the whole cell: `popup_below_widget` positions the area at
        // `widget.rect.left_bottom`, and its `clicked_elsewhere` guard reads
        // `interact_rect`/`hovered` — copying the input's would count a click
        // on the calendar button as "outside" and close the popup instantly.
        let pointer_over_cell = ui
            .ctx()
            .pointer_interact_pos()
            .is_some_and(|pos| cell_rect.contains(pos));
        let anchor = egui::Response {
            rect: cell_rect,
            interact_rect: cell_rect,
            hovered: pointer_over_cell,
            contains_pointer: pointer_over_cell,
            ..input.clone()
        };
        let picked = egui::popup::popup_below_widget(
            ui,
            popup_id,
            &anchor,
            egui::PopupCloseBehavior::CloseOnClickOutside,
            |ui| {
                // The justified popup layout can feed back into the Area size
                // (`state.size = min_size`), growing the popup a few px per
                // frame until it hits the screen edge. A hard max breaks the
                // loop; the calendar only needs ~190 px.
                ui.set_max_width(POPUP_MAX_WIDTH);
                result_grid_date_picker_view::draw_calendar(ui, theme, date_only, value)
            },
        );
        if picked == Some(true) {
            ui.memory_mut(|memory| memory.close_popup());
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
mod tests {
    use super::*;
    use egui::{Pos2, RawInput, Sense, Vec2};

    /// The editor draws inside a row's horizontal layout; its allocated
    /// footprint must stay within the 28pt cell or the whole row resizes.
    fn row_height_for(kind: CellEditorKind) -> f32 {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let mut value = "2026-09-30 14:22:11".to_owned();
        let mut error = None;
        let mut height = 0.0;
        let _ = ctx.run(
            RawInput {
                screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 300.0))),
                ..Default::default()
            },
            |ctx| {
                height = egui::CentralPanel::default()
                    .show(ctx, |ui| {
                        let row = ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                            ui.allocate_exact_size(Vec2::new(40.0, 28.0), Sense::hover());
                            let (cell_rect, _) = ui.allocate_exact_size(Vec2::new(220.0, 28.0), Sense::hover());
                            let mut context = CellEditorContext {
                                theme: DbProTheme::dark(),
                                kind: kind.clone(),
                                value: &mut value,
                                error: &mut error,
                            };
                            draw(&mut context, ui, cell_rect);
                            ui.allocate_exact_size(Vec2::new(120.0, 28.0), Sense::hover());
                        });
                        row.response.rect.height()
                    })
                    .inner;
            },
        );
        height
    }

    #[test]
    fn temporal_editor_keeps_the_row_height() {
        let height = row_height_for(CellEditorKind::Temporal { date_only: false });
        assert!(height <= 28.5, "temporal editor grew the row to {height}");
    }

    #[test]
    fn select_editor_keeps_the_row_height() {
        let height = row_height_for(CellEditorKind::Select(vec!["pending".to_owned()]));
        assert!(height <= 28.5, "select editor grew the row to {height}");
    }

    #[test]
    fn text_editor_keeps_the_row_height() {
        let height = row_height_for(CellEditorKind::Text);
        assert!(height <= 28.5, "text editor grew the row to {height}");
    }

    /// The calendar button must actually open the picker: a click inside the
    // cell must not be treated as a "click outside" that closes the popup in
    /// the same frame it was toggled open.
    #[test]
    fn calendar_button_opens_the_picker() {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let mut value = "2026-09-30 14:22:11".to_owned();
        let mut error = None;
        let screen = egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 300.0));
        let cell = egui::Rect::from_min_size(Pos2::new(60.0, 0.0), Vec2::new(220.0, 28.0));
        // The calendar button occupies the trailing 24px of the editor.
        let click = Pos2::new(cell.max.x - 12.0, 14.0);
        // A warm-up frame registers the widget rects; pointer containment is
        // resolved from last frame's rects, so the first frame cannot click.
        for event in [None, Some(true), Some(false)] {
            let events = match event {
                None => vec![],
                Some(pressed) => vec![
                    egui::Event::PointerMoved(click),
                    egui::Event::PointerButton {
                        pos: click,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::default(),
                    },
                ],
            };
            let _ = ctx.run(
                RawInput {
                    screen_rect: Some(screen),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let mut context = CellEditorContext {
                            theme: DbProTheme::dark(),
                            kind: CellEditorKind::Temporal { date_only: false },
                            value: &mut value,
                            error: &mut error,
                        };
                        draw(&mut context, ui, cell);
                    });
                },
            );
        }
        assert!(
            ctx.memory(|memory| memory.any_popup_open()),
            "clicking the calendar button did not open the picker"
        );
    }
}
