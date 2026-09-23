use crate::components::animation::hover_t;
use crate::DbProTheme;
use egui::{
    Align2, Color32, FontFamily, FontId, Frame, Margin, Order, Pos2, Response, RichText, Rounding, Sense, Stroke,
    Ui, Vec2,
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
            day: day.clamp(1, days_in_month(year, month)),
        }
    }

    /// Formats as ISO-8601 "YYYY-MM-DD".
    pub fn to_iso_string(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// Attempts to parse "YYYY-MM-DD".
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.trim().split('-').collect();
        if parts.len() == 3 {
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

pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 => 31,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        3 => 31,
        4 => 30,
        5 => 31,
        6 => 30,
        7 => 31,
        8 => 31,
        9 => 30,
        10 => 31,
        11 => 30,
        12 => 31,
        _ => 30,
    }
}

/// Computes the day of the week for a given date (0 = Sunday, 1 = Monday, ..., 6 = Saturday) using Zeller/Sakamoto formula.
pub fn day_of_week(year: i32, month: u32, day: u32) -> u32 {
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut y = year;
    if month < 3 {
        y -= 1;
    }
    let m = month as usize;
    let offset = if m > 0 && m <= 12 { t[m - 1] } else { 0 };
    let dow = (y + y / 4 - y / 100 + y / 400 + offset + day as i32) % 7;
    if dow < 0 {
        (dow + 7) as u32
    } else {
        dow as u32
    }
}

const MONTH_NAMES: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
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
        let cell_size = 32.0;
        let pad = 4.0;
        let total_width = (cell_size * 7.0) + (pad * 6.0) + 16.0;

        let frame = Frame {
            fill: self.theme.surface_floating,
            stroke: Stroke::new(1.0, self.theme.border_default),
            inner_margin: Margin::same(12.0),
            rounding: Rounding::same(8.0),
            shadow: egui::epaint::Shadow {
                offset: egui::vec2(0.0, 2.0),
                blur: 8.0,
                spread: 0.0,
                color: Color32::from_black_alpha(20),
            },
            ..Default::default()
        };

        frame
            .show(ui, |ui| {
                ui.set_width(total_width);
                ui.vertical(|ui| {
                    // Header: Month & Year with navigation chevrons
                    ui.horizontal(|ui| {
                        let prev_resp = ui.allocate_exact_size(Vec2::splat(24.0), Sense::click());
                        let p_hover = hover_t(ui.ctx(), prev_resp.1.id.with("prev_hover"), prev_resp.1.hovered());
                        if p_hover > 0.001 {
                            ui.painter().rect_filled(
                                prev_resp.0,
                                Rounding::same(4.0),
                                self.theme.surface_hover.linear_multiply(p_hover),
                            );
                        }
                        ui.painter().text(
                            prev_resp.0.center(),
                            Align2::CENTER_CENTER,
                            char::from(Icon::ChevronLeft).to_string(),
                            FontId::new(14.0, FontFamily::Name("lucide".into())),
                            self.theme.text_secondary,
                        );
                        if prev_resp.1.clicked() {
                            if *self.view_month == 1 {
                                *self.view_month = 12;
                                *self.view_year -= 1;
                            } else {
                                *self.view_month -= 1;
                            }
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

                        let next_resp = ui.allocate_exact_size(Vec2::splat(24.0), Sense::click());
                        let n_hover = hover_t(ui.ctx(), next_resp.1.id.with("next_hover"), next_resp.1.hovered());
                        if n_hover > 0.001 {
                            ui.painter().rect_filled(
                                next_resp.0,
                                Rounding::same(4.0),
                                self.theme.surface_hover.linear_multiply(n_hover),
                            );
                        }
                        ui.painter().text(
                            next_resp.0.center(),
                            Align2::CENTER_CENTER,
                            char::from(Icon::ChevronRight).to_string(),
                            FontId::new(14.0, FontFamily::Name("lucide".into())),
                            self.theme.text_secondary,
                        );
                        if next_resp.1.clicked() {
                            if *self.view_month == 12 {
                                *self.view_month = 1;
                                *self.view_year += 1;
                            } else {
                                *self.view_month += 1;
                            }
                        }
                    });

                    ui.add_space(8.0);

                    // Day of week headers
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(pad, 0.0);
                        for day_name in WEEKDAY_NAMES {
                            let (rect, _) = ui.allocate_exact_size(Vec2::new(cell_size, 20.0), Sense::hover());
                            ui.painter().text(
                                rect.center(),
                                Align2::CENTER_CENTER,
                                day_name,
                                FontId::proportional(11.5),
                                self.theme.text_muted,
                            );
                        }
                    });

                    ui.add_space(4.0);

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
                                        FontId::proportional(12.0),
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
                                            Rounding::same(6.0),
                                            self.theme.accent,
                                        );
                                    } else if hover > 0.001 {
                                        ui.painter().rect_filled(
                                            cell_rect,
                                            Rounding::same(6.0),
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
                                        FontId::proportional(12.5),
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
                                        FontId::proportional(12.0),
                                        self.theme.text_disabled,
                                    );
                                    if resp.clicked() {
                                        let (next_year, next_month) = if *self.view_month == 12 {
                                            (*self.view_year + 1, 1)
                                        } else {
                                            (*self.view_year, *self.view_month + 1)
                                        };
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
    }
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

        let height = 32.0;
        let width = 160.0;
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(width, height),
            if self.enabled { Sense::click() } else { Sense::hover() },
        );

        if self.enabled && response.clicked() {
            is_open = !is_open;
        }

        let hover = hover_t(ui.ctx(), id.with("hover"), self.enabled && response.hovered());
        let fill = if is_open {
            self.theme.surface_hover
        } else if hover > 0.001 {
            self.theme.surface_hover.linear_multiply(hover * 0.7)
        } else {
            self.theme.surface_panel
        };

        ui.painter().rect_filled(rect, Rounding::same(6.0), fill);
        ui.painter().rect_stroke(
            rect,
            Rounding::same(6.0),
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
        let icon_pos = Pos2::new(rect.left() + 10.0, rect.center().y);
        ui.painter().text(
            icon_pos,
            Align2::LEFT_CENTER,
            char::from(Icon::Calendar).to_string(),
            FontId::new(14.0, FontFamily::Name("lucide".into())),
            if self.date.is_some() { self.theme.accent } else { self.theme.text_secondary },
        );

        // Date text
        let text_pos = Pos2::new(rect.left() + 32.0, rect.center().y);
        let (text, color) = if let Some(d) = self.date {
            (d.to_iso_string(), self.theme.text_primary)
        } else {
            (self.placeholder.to_string(), self.theme.text_muted)
        };
        ui.painter().text(
            text_pos,
            Align2::LEFT_CENTER,
            text,
            DbProTheme::ui_medium_font(12.5),
            color,
        );

        if is_open {
            let mut view_year = self.date.map(|d| d.year).unwrap_or(2026);
            let mut view_month = self.date.map(|d| d.month).unwrap_or(9);

            let popover_pos = Pos2::new(rect.left(), rect.bottom() + 4.0);
            let popup = egui::Area::new(id.with("popover"))
                .order(Order::Foreground)
                .fixed_pos(popover_pos)
                .show(ui.ctx(), |ui| {
                    Calendar::new(self.date, &mut view_year, &mut view_month, self.theme).show(ui)
                });

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
