use super::config;
use super::handler::{date_picker_popup_open, day_of_week, days_in_month, next_month, previous_month};
use crate::components::animation::hover_t;
use crate::components::clamp_popup_to_screen;
use crate::DbProTheme;
use chrono::Datelike;
use egui::{
    Align2, Color32, FontFamily, FontId, Frame, Layout, Margin, Order, Pos2, Response, RichText, Rounding, Sense,
    Stroke, Ui, Vec2,
};
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimpleDate {
    pub year: i32,
    pub month: u32, // 1 to 12
    pub day: u32,   // 1 to 31
}

impl SimpleDate {
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self {
            year,
            month: month.clamp(1, 12),
            day: day.clamp(1, days_in_month(year, month.clamp(1, 12))),
        }
    }

    /// Formats as ISO-8601 "YYYY-MM-DD".
    pub fn to_iso_string(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// Attempts to parse "YYYY-MM-DD".
    pub fn parse(s: &str) -> Option<Self> {
        if s.len() != 10 || s.as_bytes().get(4) != Some(&b'-') || s.as_bytes().get(7) != Some(&b'-') {
            return None;
        }
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() == 3 && parts[0].len() == 4 && parts[1].len() == 2 && parts[2].len() == 2 {
            let year = parts[0].parse::<i32>().ok()?;
            let month = parts[1].parse::<u32>().ok()?;
            let day = parts[2].parse::<u32>().ok()?;
            if (1..=12).contains(&month) && day >= 1 && day <= days_in_month(year, month) {
                return Some(Self::new(year, month, day));
            }
        }
        None
    }
}

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const WEEKDAY_NAMES: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

pub struct Calendar<'a> {
    selected: &'a mut Option<SimpleDate>,
    view_year: &'a mut i32,
    view_month: &'a mut u32,
    theme: DbProTheme,
}

impl<'a> Calendar<'a> {
    pub fn new(
        selected: &'a mut Option<SimpleDate>,
        view_year: &'a mut i32,
        view_month: &'a mut u32,
        theme: DbProTheme,
    ) -> Self {
        if *view_month < 1 || *view_month > 12 {
            *view_month = 1;
        }
        Self {
            selected,
            view_year,
            view_month,
            theme,
        }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let cell_size = config::CELL_SIZE;
        let pad = config::CELL_GAP;
        let content_width = calendar_content_width();
        let frame_width = calendar_frame_size().x;

        let frame = Frame {
            fill: self.theme.surface_floating,
            stroke: Stroke::new(1.0, self.theme.border_default),
            inner_margin: Margin::same(config::CALENDAR_INNER_MARGIN),
            rounding: Rounding::same(config::SURFACE_RADIUS),
            shadow: egui::epaint::Shadow {
                offset: egui::vec2(0.0, 2.0),
                blur: config::SHADOW_BLUR,
                spread: 0.0,
                color: Color32::from_black_alpha(20),
            },
            ..Default::default()
        };

        ui.allocate_ui_with_layout(Vec2::new(frame_width, 0.0), Layout::top_down(egui::Align::Min), |ui| {
            frame
                .show(ui, |ui| {
                    ui.set_width(content_width);
                    ui.vertical(|ui| {
                        // Header: Month & Year with navigation chevrons
                        ui.horizontal(|ui| {
                            let prev_resp =
                                ui.allocate_exact_size(Vec2::splat(config::HEADER_BUTTON_SIZE), Sense::click());
                            let p_hover = hover_t(ui.ctx(), prev_resp.1.id.with("prev_hover"), prev_resp.1.hovered());
                            if p_hover > 0.001 {
                                ui.painter().rect_filled(
                                    prev_resp.0,
                                    Rounding::same(config::CONTROL_RADIUS),
                                    self.theme.surface_hover.linear_multiply(p_hover),
                                );
                            }
                            ui.painter().text(
                                prev_resp.0.center(),
                                Align2::CENTER_CENTER,
                                char::from(Icon::ChevronLeft).to_string(),
                                FontId::new(config::CALENDAR_ICON_SIZE, FontFamily::Name("lucide".into())),
                                self.theme.text_secondary,
                            );
                            if prev_resp.1.clicked() {
                                (*self.view_year, *self.view_month) = previous_month(*self.view_year, *self.view_month);
                            }

                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                let m_idx = (*self.view_month as usize).saturating_sub(1).min(11);
                                let title = format!("{} {}", MONTH_NAMES[m_idx], self.view_year);
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        RichText::new(title)
                                            .font(DbProTheme::ui_medium_font(13.0))
                                            .color(self.theme.text_primary),
                                    );
                                });
                            });

                            let next_resp =
                                ui.allocate_exact_size(Vec2::splat(config::HEADER_BUTTON_SIZE), Sense::click());
                            let n_hover = hover_t(ui.ctx(), next_resp.1.id.with("next_hover"), next_resp.1.hovered());
                            if n_hover > 0.001 {
                                ui.painter().rect_filled(
                                    next_resp.0,
                                    Rounding::same(config::NAVIGATION_RADIUS),
                                    self.theme.surface_hover.linear_multiply(n_hover),
                                );
                            }
                            ui.painter().text(
                                next_resp.0.center(),
                                Align2::CENTER_CENTER,
                                char::from(Icon::ChevronRight).to_string(),
                                FontId::new(config::CALENDAR_ICON_SIZE, FontFamily::Name("lucide".into())),
                                self.theme.text_secondary,
                            );
                            if next_resp.1.clicked() {
                                (*self.view_year, *self.view_month) = next_month(*self.view_year, *self.view_month);
                            }
                        });

                        ui.add_space(config::SECTION_GAP);

                        // Day of week headers
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(pad, 0.0);
                            for day_name in WEEKDAY_NAMES {
                                let (rect, _) = ui
                                    .allocate_exact_size(Vec2::new(cell_size, config::DAY_ROW_HEIGHT), Sense::hover());
                                ui.painter().text(
                                    rect.center(),
                                    Align2::CENTER_CENTER,
                                    day_name,
                                    FontId::proportional(11.5),
                                    self.theme.text_muted,
                                );
                            }
                        });

                        ui.add_space(config::GRID_GAP);

                        // Day grid
                        let first_dow = day_of_week(*self.view_year, *self.view_month, 1);
                        let days_this_month = days_in_month(*self.view_year, *self.view_month);

                        let (prev_year, prev_month) = if *self.view_month == 1 {
                            (*self.view_year - 1, 12)
                        } else {
                            (*self.view_year, *self.view_month - 1)
                        };
                        let days_prev_month = days_in_month(prev_year, prev_month);

                        for row in 0..6 {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = Vec2::new(pad, pad);
                                for col in 0..7 {
                                    let idx = row * 7 + col;
                                    let (cell_rect, resp) =
                                        ui.allocate_exact_size(Vec2::splat(cell_size), Sense::click());

                                    if idx < first_dow {
                                        // Day from previous month
                                        let d = days_prev_month - (first_dow - idx - 1);
                                        ui.painter().text(
                                            cell_rect.center(),
                                            Align2::CENTER_CENTER,
                                            d.to_string(),
                                            FontId::proportional(config::DAY_FONT_SIZE),
                                            self.theme.text_disabled,
                                        );
                                        if resp.clicked() {
                                            *self.selected = Some(SimpleDate::new(prev_year, prev_month, d));
                                            *self.view_month = prev_month;
                                            *self.view_year = prev_year;
                                        }
                                    } else if idx < first_dow + days_this_month {
                                        // Day in current month
                                        let day_num = idx - first_dow + 1;
                                        let this_date = SimpleDate::new(*self.view_year, *self.view_month, day_num);
                                        let is_selected = *self.selected == Some(this_date);

                                        let hover = hover_t(
                                            ui.ctx(),
                                            resp.id.with("day_hover"),
                                            resp.hovered() && !is_selected,
                                        );

                                        if is_selected {
                                            ui.painter().rect_filled(
                                                cell_rect,
                                                Rounding::same(config::CONTROL_RADIUS),
                                                self.theme.accent,
                                            );
                                        } else if hover > 0.001 {
                                            ui.painter().rect_filled(
                                                cell_rect,
                                                Rounding::same(config::CONTROL_RADIUS),
                                                self.theme.surface_hover.linear_multiply(hover),
                                            );
                                        }

                                        let text_color = if is_selected {
                                            self.theme.accent_foreground
                                        } else if resp.hovered() {
                                            self.theme.text_primary
                                        } else {
                                            self.theme.text_secondary
                                        };

                                        ui.painter().text(
                                            cell_rect.center(),
                                            Align2::CENTER_CENTER,
                                            day_num.to_string(),
                                            FontId::proportional(config::SELECTED_DAY_FONT_SIZE),
                                            text_color,
                                        );

                                        if resp.clicked() {
                                            *self.selected = Some(this_date);
                                        }
                                    } else {
                                        // Day in next month
                                        let d = idx - (first_dow + days_this_month) + 1;
                                        ui.painter().text(
                                            cell_rect.center(),
                                            Align2::CENTER_CENTER,
                                            d.to_string(),
                                            FontId::proportional(config::DAY_FONT_SIZE),
                                            self.theme.text_disabled,
                                        );
                                        if resp.clicked() {
                                            let (next_year, next_month) = next_month(*self.view_year, *self.view_month);
                                            *self.selected = Some(SimpleDate::new(next_year, next_month, d));
                                            *self.view_month = next_month;
                                            *self.view_year = next_year;
                                        }
                                    }
                                }
                            });
                        }
                    });
                })
                .response
        })
        .inner
    }
}

fn calendar_content_width() -> f32 {
    (config::CELL_SIZE * 7.0) + (config::CELL_GAP * 6.0) + config::GRID_OUTER_PADDING
}

fn calendar_frame_size() -> Vec2 {
    Vec2::new(
        calendar_content_width() + (config::CALENDAR_INNER_MARGIN * 2.0),
        config::CALENDAR_INNER_MARGIN * 2.0
            + config::HEADER_BUTTON_SIZE
            + config::SECTION_GAP
            + config::DAY_ROW_HEIGHT
            + config::GRID_GAP
            + (config::CELL_SIZE * 6.0)
            + (config::CELL_GAP * 5.0),
    )
}

pub struct DatePicker<'a> {
    id: &'a str,
    date: &'a mut Option<SimpleDate>,
    placeholder: &'a str,
    enabled: bool,
    theme: DbProTheme,
}

impl<'a> DatePicker<'a> {
    pub fn new(id: &'a str, date: &'a mut Option<SimpleDate>, theme: DbProTheme) -> Self {
        Self {
            id,
            date,
            placeholder: "Pick a date",
            enabled: true,
            theme,
        }
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let id = ui.id().with(("date_picker", self.id));
        let mut is_open = ui.data(|d| d.get_temp::<bool>(id.with("is_open"))).unwrap_or(false);

        let height = config::DATE_PICKER_HEIGHT;
        let width = config::DATE_PICKER_WIDTH;
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(width, height),
            if self.enabled { Sense::click() } else { Sense::hover() },
        );

        if self.enabled && response.clicked() {
            is_open = !is_open;
        }

        let escape_pressed = ui.input(|input| input.key_pressed(egui::Key::Escape));
        is_open = date_picker_popup_open(self.enabled, is_open, escape_pressed);
        // Disabled pickers must not leave a stale popup in egui's temp state.
        if !self.enabled {
            ui.data_mut(|data| data.insert_temp(id.with("is_open"), false));
        }

        let hover = hover_t(ui.ctx(), id.with("hover"), self.enabled && response.hovered());
        let fill = if is_open {
            self.theme.surface_hover
        } else if hover > 0.001 {
            self.theme.surface_hover.linear_multiply(hover * 0.7)
        } else {
            self.theme.surface_panel
        };

        ui.painter()
            .rect_filled(rect, Rounding::same(config::CONTROL_RADIUS), fill);
        ui.painter().rect_stroke(
            rect,
            Rounding::same(config::CONTROL_RADIUS),
            Stroke::new(
                1.0,
                if is_open {
                    self.theme.accent
                } else if hover > 0.001 {
                    self.theme.border_strong
                } else {
                    self.theme.border_default
                },
            ),
        );

        // Icon Calendar on left
        let icon_pos = Pos2::new(rect.left() + config::ICON_INSET, rect.center().y);
        ui.painter().text(
            icon_pos,
            Align2::LEFT_CENTER,
            char::from(Icon::Calendar).to_string(),
            FontId::new(config::CALENDAR_ICON_SIZE, FontFamily::Name("lucide".into())),
            if self.date.is_some() {
                self.theme.accent
            } else {
                self.theme.text_secondary
            },
        );

        // Date text
        let text_pos = Pos2::new(rect.left() + config::TEXT_INSET, rect.center().y);
        let (text, color) = if let Some(d) = self.date {
            (d.to_iso_string(), self.theme.text_primary)
        } else {
            (self.placeholder.to_string(), self.theme.text_muted)
        };
        ui.painter().text(
            text_pos,
            Align2::LEFT_CENTER,
            text,
            DbProTheme::ui_medium_font(config::SELECTED_DAY_FONT_SIZE),
            color,
        );

        if self.enabled && response.clicked() && is_open {
            let today = chrono::Local::now().date_naive();
            let (year, month) = self
                .date
                .map(|d| (d.year, d.month))
                .unwrap_or((today.year(), today.month()));
            ui.data_mut(|d| {
                d.insert_temp(id.with("view_year"), year);
                d.insert_temp(id.with("view_month"), month);
            });
        }

        if is_open {
            let today = chrono::Local::now().date_naive();
            let mut view_year = ui
                .data(|d| d.get_temp::<i32>(id.with("view_year")))
                .unwrap_or(self.date.map(|d| d.year).unwrap_or(today.year()));
            let mut view_month = ui
                .data(|d| d.get_temp::<u32>(id.with("view_month")))
                .unwrap_or(self.date.map(|d| d.month).unwrap_or(today.month()));
            let previous_date = *self.date;

            let desired_popover_pos = Pos2::new(rect.left(), rect.bottom() + config::POPOVER_GAP);
            let popover_pos = clamp_popup_to_screen(
                desired_popover_pos,
                calendar_frame_size(),
                ui.ctx().screen_rect(),
                config::POPOVER_SCREEN_MARGIN,
            );
            let popup = egui::Area::new(id.with("popover"))
                .order(Order::Foreground)
                .fixed_pos(popover_pos)
                .show(ui.ctx(), |ui| {
                    Calendar::new(self.date, &mut view_year, &mut view_month, self.theme).show(ui)
                });
            // Navigation mutates local values during this frame; persist them under this
            // picker's stable ID so Previous/Next survives the next egui frame.
            ui.data_mut(|data| {
                data.insert_temp(id.with("view_year"), view_year);
                data.insert_temp(id.with("view_month"), view_month);
            });

            // Selection changes are produced inside the popup; close after observing them.
            if *self.date != previous_date {
                is_open = false;
            }

            // Close on click outside
            if ui.input(|i| i.pointer.any_click()) {
                if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                    if !rect.contains(pos) && !popup.response.rect.contains(pos) {
                        is_open = false;
                    }
                }
            }
        }

        ui.data_mut(|d| d.insert_temp(id.with("is_open"), is_open));

        if self.enabled {
            response.on_hover_cursor(egui::CursorIcon::PointingHand)
        } else {
            response
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_frame_size_is_intrinsic_and_includes_symmetric_margin() {
        assert_eq!(calendar_content_width(), 264.0);
        assert_eq!(calendar_frame_size(), Vec2::new(288.0, 292.0));
    }
}
