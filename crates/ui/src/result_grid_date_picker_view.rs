//! Adapts the shared calendar to the result grid's raw temporal cell value.
use super::super::*;
use chrono::{Datelike, Local};

/// Draws the shared calendar and remembers its viewed month for this cell.
pub(super) fn draw_calendar(ui: &mut egui::Ui, theme: DbProTheme, date_only: bool, value: &mut String) -> bool {
    let state_id = ui.id().with("viewed_month");
    let today = Local::now().date_naive();
    let selected = value.get(..10).and_then(SimpleDate::parse);
    let mut year = ui
        .ctx()
        .data_mut(|data| data.get_temp::<i32>(state_id.with("year")))
        .unwrap_or_else(|| selected.map(|date| date.year).unwrap_or(today.year()));
    let mut month = ui
        .ctx()
        .data_mut(|data| data.get_temp::<u32>(state_id.with("month")))
        .unwrap_or_else(|| selected.map(|date| date.month).unwrap_or(today.month()));

    let previous_value = value.clone();
    Calendar::for_temporal_value(value, date_only, &mut year, &mut month, theme).show(ui);

    ui.ctx().data_mut(|data| {
        data.insert_temp(state_id.with("year"), year);
        data.insert_temp(state_id.with("month"), month);
    });
    *value != previous_value
}
