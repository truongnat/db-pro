//! Calendar popup for temporal result-cell editors.
//!
//! Pure presentation: draws a small month grid into a `popup_below_widget`
//! and writes the chosen day back into the editor's raw string, keeping any
//! existing time component for datetime columns.
use super::super::*;
use chrono::{Datelike, Local, NaiveDate};
use egui::{FontId, Layout, RichText, Rounding, Sense, Stroke};

const WEEKDAYS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
const CELL: f32 = 24.0;

/// Draws the month grid. Returns `true` when the user picked a day so the
/// caller can close the popup.
pub(super) fn draw_calendar(ui: &mut egui::Ui, theme: DbProTheme, date_only: bool, value: &mut String) -> bool {
    let state_id = ui.id().with("viewed_month");
    let today = Local::now().date_naive();
    let mut viewed = ui
        .ctx()
        .data_mut(|data| data.get_temp::<NaiveDate>(state_id))
        .unwrap_or_else(|| parse_date_prefix(value).unwrap_or(today).with_day(1).unwrap_or(today));

    ui.set_min_width(CELL * 7.0 + 16.0);
    let mut picked = false;
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);
        viewed = draw_month_header(ui, theme, viewed);
        draw_weekday_header(ui, theme);
        picked = draw_day_grid(ui, theme, viewed, today, date_only, value);
        draw_footer(ui, theme, today, date_only, value, &mut picked);
    });
    ui.ctx().data_mut(|data| data.insert_temp(state_id, viewed));
    picked
}

fn draw_month_header(ui: &mut egui::Ui, theme: DbProTheme, mut viewed: NaiveDate) -> NaiveDate {
    ui.horizontal(|ui| {
        if ui
            .add(egui::Button::new(RichText::new("‹").size(14.0).color(theme.text_secondary)).frame(false))
            .clicked()
        {
            viewed = shift_month(viewed, -1);
        }
        ui.with_layout(Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
            ui.label(
                RichText::new(viewed.format("%B %Y").to_string())
                    .size(12.0)
                    .color(theme.text_primary),
            );
        });
        if ui
            .add(egui::Button::new(RichText::new("›").size(14.0).color(theme.text_secondary)).frame(false))
            .clicked()
        {
            viewed = shift_month(viewed, 1);
        }
    });
    viewed
}

fn draw_weekday_header(ui: &mut egui::Ui, theme: DbProTheme) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
        for day in WEEKDAYS {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(CELL, 18.0), Sense::hover());
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                day,
                FontId::proportional(9.5),
                theme.text_muted,
            );
        }
    });
}

fn draw_day_grid(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    viewed: NaiveDate,
    today: NaiveDate,
    date_only: bool,
    value: &mut String,
) -> bool {
    let selected = parse_date_prefix(value);
    let days_in_month = days_in(viewed.year(), viewed.month());
    let offset = viewed.weekday().num_days_from_monday();
    let mut picked = false;
    for week in 0..6 {
        let first = week * 7;
        if first >= offset + days_in_month {
            break;
        }
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
            for column in 0..7u32 {
                let index = first + column;
                let (rect, response) = ui.allocate_exact_size(egui::vec2(CELL, CELL - 2.0), Sense::click());
                if index < offset || index >= offset + days_in_month {
                    continue;
                }
                let day = index - offset + 1;
                let Some(date) = viewed.with_day(day) else { continue };
                paint_day(ui, theme, rect, &response, date, today, selected);
                if response.clicked() {
                    *value = merge_date(date, value, date_only);
                    picked = true;
                }
            }
        });
    }
    picked
}

fn paint_day(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    rect: egui::Rect,
    response: &egui::Response,
    date: NaiveDate,
    today: NaiveDate,
    selected: Option<NaiveDate>,
) {
    if selected == Some(date) {
        ui.painter()
            .rect_filled(rect.shrink(1.0), Rounding::same(4.0), theme.accent_soft);
    } else if response.hovered() {
        ui.painter()
            .rect_filled(rect.shrink(1.0), Rounding::same(4.0), theme.surface_hover);
    }
    if date == today {
        ui.painter().rect_stroke(
            rect.shrink(1.0),
            Rounding::same(4.0),
            Stroke::new(1.0, theme.accent.linear_multiply(0.6)),
        );
    }
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        date.day().to_string(),
        FontId::monospace(11.0),
        if selected == Some(date) {
            theme.accent
        } else {
            theme.text_primary
        },
    );
}

fn draw_footer(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    today: NaiveDate,
    date_only: bool,
    value: &mut String,
    picked: &mut bool,
) {
    ui.horizontal(|ui| {
        if ui
            .add(egui::Button::new(RichText::new("Today").size(11.0).color(theme.accent)).frame(false))
            .clicked()
        {
            *value = merge_date(today, value, date_only);
            *picked = true;
        }
    });
}

/// Parses the leading `YYYY-MM-DD` portion of the editor value.
fn parse_date_prefix(value: &str) -> Option<NaiveDate> {
    let prefix = value.get(..10)?;
    NaiveDate::parse_from_str(prefix, "%Y-%m-%d").ok()
}

/// Keeps the existing time portion when the column is a datetime.
fn merge_date(date: NaiveDate, value: &str, date_only: bool) -> String {
    if date_only {
        return date.format("%Y-%m-%d").to_string();
    }
    let time = value
        .get(10..)
        .map(|rest| rest.trim_start_matches(['T', ' ']))
        .filter(|rest| rest.contains(':'))
        .unwrap_or("00:00:00");
    format!("{} {}", date.format("%Y-%m-%d"), time)
}

fn days_in(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .and_then(|first| first.pred_opt())
        .map(|last| last.day())
        .unwrap_or(30)
}

fn shift_month(viewed: NaiveDate, delta: i32) -> NaiveDate {
    let month_index = viewed.year() * 12 + viewed.month() as i32 - 1 + delta;
    let year = month_index.div_euclid(12);
    let month = month_index.rem_euclid(12) as u32 + 1;
    NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(viewed)
}
