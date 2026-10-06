use super::config;
use super::handler::{
    date_picker_popup_open, day_of_week, days_in_month, next_month, previous_month, should_activate_focused_control,
};
use crate::components::animation::hover_t;
use crate::components::clamp_popup_to_screen;
use crate::DbProTheme;
use chrono::{Datelike, Local};
use egui::{
    Align2, FontFamily, FontId, Frame, Layout, Margin, Order, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke,
    Ui, Vec2, WidgetInfo, WidgetType,
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

const SUNDAY_FIRST_WEEKDAYS: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
const MONDAY_FIRST_WEEKDAYS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

pub struct Calendar<'a> {
    selected: CalendarSelection<'a>,
    view_year: &'a mut i32,
    view_month: &'a mut u32,
    monday_first: bool,
    show_today: bool,
    theme: DbProTheme,
}

enum CalendarSelection<'a> {
    Date(&'a mut Option<SimpleDate>),
    TemporalValue { value: &'a mut String, date_only: bool },
}

impl CalendarSelection<'_> {
    fn selected_date(&self) -> Option<SimpleDate> {
        match self {
            Self::Date(selected) => **selected,
            Self::TemporalValue { value, .. } => value.get(..10).and_then(SimpleDate::parse),
        }
    }

    fn select(&mut self, date: SimpleDate) {
        match self {
            Self::Date(selected) => **selected = Some(date),
            Self::TemporalValue { value, date_only } => {
                **value = merge_temporal_date(date, value, *date_only);
            }
        }
    }
}

fn merge_temporal_date(date: SimpleDate, value: &str, date_only: bool) -> String {
    let iso_date = date.to_iso_string();
    if date_only {
        return iso_date;
    }

    let time = value
        .get(10..)
        .map(|suffix| suffix.trim_start_matches(['T', ' ']))
        .filter(|suffix| suffix.contains(':'))
        .unwrap_or("00:00:00");
    format!("{iso_date} {time}")
}

impl<'a> Calendar<'a> {
    pub(crate) fn popup_position(anchor: Rect, screen: Rect, show_today: bool) -> Pos2 {
        let desired_position = Pos2::new(anchor.left(), anchor.bottom() + config::POPOVER_GAP);
        clamp_popup_to_screen(
            desired_position,
            calendar_frame_size(show_today),
            screen,
            config::POPOVER_SCREEN_MARGIN,
        )
    }

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
            selected: CalendarSelection::Date(selected),
            view_year,
            view_month,
            monday_first: false,
            show_today: false,
            theme,
        }
    }

    /// Creates a calendar for an ISO date or datetime string.
    /// Date selection preserves the existing time suffix for datetime values.
    pub fn for_temporal_value(
        value: &'a mut String,
        date_only: bool,
        view_year: &'a mut i32,
        view_month: &'a mut u32,
        theme: DbProTheme,
    ) -> Self {
        if *view_month < 1 || *view_month > 12 {
            *view_month = 1;
        }
        Self {
            selected: CalendarSelection::TemporalValue { value, date_only },
            view_year,
            view_month,
            monday_first: true,
            show_today: true,
            theme,
        }
    }

    pub fn show(mut self, ui: &mut Ui) -> Response {
        let layout = calendar_layout(ui.available_width());
        let cell_size = layout.cell_size;
        let pad = config::CELL_GAP;
        let content_width = layout.content_width;
        let frame_width = layout.frame_width;

        let frame = Frame {
            fill: self.theme.surface_floating,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            inner_margin: Margin::same(config::CALENDAR_INNER_MARGIN),
            rounding: Rounding::same(config::SURFACE_RADIUS),
            shadow: self.theme.floating_shadow(),
            ..Default::default()
        };

        ui.allocate_ui_with_layout(Vec2::new(frame_width, 0.0), Layout::top_down(egui::Align::Min), |ui| {
            frame
                .show(ui, |ui| {
                    ui.set_width(content_width);
                    ui.vertical(|ui| {
                        // Header: Month & Year with navigation chevrons. Keep the
                        // three regions fixed to the grid width so the title
                        // cannot consume space that belongs to the next button.
                        ui.allocate_ui_with_layout(
                            Vec2::new(content_width, config::HEADER_BUTTON_SIZE),
                            Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.spacing_mut().item_spacing = Vec2::ZERO;
                                let prev_resp =
                                    ui.allocate_exact_size(Vec2::splat(config::HEADER_BUTTON_SIZE), Sense::click());
                                prev_resp
                                    .1
                                    .widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, "Previous month"));
                                let prev_keyboard_activation = calendar_keyboard_activation(ui, &prev_resp.1);
                                if prev_resp.1.has_focus() {
                                    crate::components::interact::paint_focus_ring(
                                        ui,
                                        prev_resp.0,
                                        config::CONTROL_RADIUS,
                                        self.theme,
                                    );
                                }
                                let p_hover = if self.theme.reduce_motion {
                                    if prev_resp.1.hovered() {
                                        1.0
                                    } else {
                                        0.0
                                    }
                                } else {
                                    hover_t(ui.ctx(), prev_resp.1.id.with("prev_hover"), prev_resp.1.hovered())
                                };
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
                                if prev_resp.1.clicked() || prev_keyboard_activation {
                                    prev_resp.1.request_focus();
                                    (*self.view_year, *self.view_month) =
                                        previous_month(*self.view_year, *self.view_month);
                                }

                                let title_width = (content_width - (config::HEADER_BUTTON_SIZE * 2.0)).max(0.0);
                                ui.allocate_ui_with_layout(
                                    Vec2::new(title_width, config::HEADER_BUTTON_SIZE),
                                    Layout::centered_and_justified(egui::Direction::LeftToRight),
                                    |ui| {
                                        let m_idx = (*self.view_month as usize).saturating_sub(1).min(11);
                                        let title = format!("{} {}", MONTH_NAMES[m_idx], self.view_year);
                                        ui.label(
                                            RichText::new(title)
                                                .font(DbProTheme::ui_medium_font(13.0))
                                                .color(self.theme.text_primary),
                                        );
                                    },
                                );

                                let next_resp =
                                    ui.allocate_exact_size(Vec2::splat(config::HEADER_BUTTON_SIZE), Sense::click());
                                next_resp
                                    .1
                                    .widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, "Next month"));
                                let next_keyboard_activation = calendar_keyboard_activation(ui, &next_resp.1);
                                if next_resp.1.has_focus() {
                                    crate::components::interact::paint_focus_ring(
                                        ui,
                                        next_resp.0,
                                        config::CONTROL_RADIUS,
                                        self.theme,
                                    );
                                }
                                let n_hover = if self.theme.reduce_motion {
                                    if next_resp.1.hovered() {
                                        1.0
                                    } else {
                                        0.0
                                    }
                                } else {
                                    hover_t(ui.ctx(), next_resp.1.id.with("next_hover"), next_resp.1.hovered())
                                };
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
                                if next_resp.1.clicked() || next_keyboard_activation {
                                    next_resp.1.request_focus();
                                    (*self.view_year, *self.view_month) = next_month(*self.view_year, *self.view_month);
                                }
                            },
                        );

                        ui.add_space(config::SECTION_GAP);

                        // Day of week headers
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(pad, 0.0);
                            let weekday_names = if self.monday_first {
                                MONDAY_FIRST_WEEKDAYS
                            } else {
                                SUNDAY_FIRST_WEEKDAYS
                            };
                            for day_name in weekday_names {
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
                        let sunday_first_dow = day_of_week(*self.view_year, *self.view_month, 1);
                        let first_dow = if self.monday_first {
                            (sunday_first_dow + 6) % 7
                        } else {
                            sunday_first_dow
                        };
                        let days_this_month = days_in_month(*self.view_year, *self.view_month);
                        let mut selected_date = self.selected.selected_date();

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
                                    let (cell_rect, mut resp) =
                                        ui.allocate_exact_size(Vec2::splat(cell_size), Sense::click());
                                    let keyboard_activation = calendar_keyboard_activation(ui, &resp);

                                    if idx < first_dow {
                                        // Day from previous month
                                        let d = days_prev_month - (first_dow - idx - 1);
                                        let date = SimpleDate::new(prev_year, prev_month, d);
                                        let activated = resp.clicked() || keyboard_activation;
                                        if activated {
                                            resp.request_focus();
                                            self.selected.select(date);
                                            selected_date = Some(date);
                                            *self.view_month = prev_month;
                                            *self.view_year = prev_year;
                                            resp.mark_changed();
                                        }
                                        resp.widget_info(|| {
                                            WidgetInfo::selected(
                                                WidgetType::Button,
                                                true,
                                                selected_date == Some(date),
                                                date.to_iso_string(),
                                            )
                                        });
                                        if resp.has_focus() {
                                            crate::components::interact::paint_focus_ring(
                                                ui,
                                                cell_rect,
                                                config::CONTROL_RADIUS,
                                                self.theme,
                                            );
                                        }
                                        ui.painter().text(
                                            cell_rect.center(),
                                            Align2::CENTER_CENTER,
                                            d.to_string(),
                                            FontId::proportional(config::DAY_FONT_SIZE),
                                            self.theme.text_disabled,
                                        );
                                    } else if idx < first_dow + days_this_month {
                                        // Day in current month
                                        let day_num = idx - first_dow + 1;
                                        let this_date = SimpleDate::new(*self.view_year, *self.view_month, day_num);
                                        let activated = resp.clicked() || keyboard_activation;
                                        if activated {
                                            resp.request_focus();
                                            self.selected.select(this_date);
                                            selected_date = Some(this_date);
                                            resp.mark_changed();
                                        }
                                        let is_selected = selected_date == Some(this_date);
                                        resp.widget_info(|| {
                                            WidgetInfo::selected(
                                                WidgetType::Button,
                                                true,
                                                is_selected,
                                                this_date.to_iso_string(),
                                            )
                                        });
                                        if resp.has_focus() {
                                            crate::components::interact::paint_focus_ring(
                                                ui,
                                                cell_rect,
                                                config::CONTROL_RADIUS,
                                                self.theme,
                                            );
                                        }

                                        let is_hovered = resp.hovered() && !is_selected;
                                        let hover = if self.theme.reduce_motion {
                                            if is_hovered {
                                                1.0
                                            } else {
                                                0.0
                                            }
                                        } else {
                                            hover_t(ui.ctx(), resp.id.with("day_hover"), is_hovered)
                                        };

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
                                    } else {
                                        // Day in next month
                                        let d = idx - (first_dow + days_this_month) + 1;
                                        let (next_year, next_month) = next_month(*self.view_year, *self.view_month);
                                        let date = SimpleDate::new(next_year, next_month, d);
                                        let activated = resp.clicked() || keyboard_activation;
                                        if activated {
                                            resp.request_focus();
                                            self.selected.select(date);
                                            selected_date = Some(date);
                                            *self.view_month = next_month;
                                            *self.view_year = next_year;
                                            resp.mark_changed();
                                        }
                                        resp.widget_info(|| {
                                            WidgetInfo::selected(
                                                WidgetType::Button,
                                                true,
                                                selected_date == Some(date),
                                                date.to_iso_string(),
                                            )
                                        });
                                        if resp.has_focus() {
                                            crate::components::interact::paint_focus_ring(
                                                ui,
                                                cell_rect,
                                                config::CONTROL_RADIUS,
                                                self.theme,
                                            );
                                        }
                                        ui.painter().text(
                                            cell_rect.center(),
                                            Align2::CENTER_CENTER,
                                            d.to_string(),
                                            FontId::proportional(config::DAY_FONT_SIZE),
                                            self.theme.text_disabled,
                                        );
                                    }
                                }
                            });
                        }

                        if self.show_today {
                            ui.add_space(config::SECTION_GAP);
                            let today = Local::now().date_naive();
                            let today = SimpleDate::new(today.year(), today.month(), today.day());
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Today")
                                            .font(DbProTheme::ui_medium_font(13.0))
                                            .color(self.theme.accent),
                                    )
                                    .frame(false),
                                )
                                .clicked()
                            {
                                self.selected.select(today);
                            }
                        }
                    });
                })
                .response
        })
        .inner
    }
}

fn calendar_content_width() -> f32 {
    (config::CELL_SIZE * 7.0) + (config::CELL_GAP * 6.0)
}

fn calendar_frame_size(show_today: bool) -> Vec2 {
    Vec2::new(
        calendar_content_width() + (config::CALENDAR_INNER_MARGIN * 2.0),
        config::CALENDAR_INNER_MARGIN * 2.0
            + config::HEADER_BUTTON_SIZE
            + config::SECTION_GAP
            + config::DAY_ROW_HEIGHT
            + config::GRID_GAP
            + (config::CELL_SIZE * 6.0)
            + (config::CELL_GAP * 5.0)
            + if show_today {
                config::SECTION_GAP + config::DAY_ROW_HEIGHT
            } else {
                0.0
            },
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CalendarLayout {
    cell_size: f32,
    content_width: f32,
    frame_width: f32,
}

fn calendar_layout(available_width: f32) -> CalendarLayout {
    let intrinsic_frame_width = calendar_frame_size(false).x;
    let target_frame_width = available_width.max(0.0).min(intrinsic_frame_width);
    let content_width = (target_frame_width - (config::CALENDAR_INNER_MARGIN * 2.0)).max(0.0);
    let cell_size = ((content_width - (config::CELL_GAP * 6.0)) / 7.0).max(1.0);
    let content_width = (cell_size * 7.0) + (config::CELL_GAP * 6.0);

    CalendarLayout {
        cell_size,
        content_width,
        frame_width: content_width + (config::CALENDAR_INNER_MARGIN * 2.0),
    }
}

fn calendar_keyboard_activation(ui: &mut Ui, response: &Response) -> bool {
    // Custom-painted click responses need an explicit focused-key path for activation.
    let (enter_pressed, space_pressed) = if response.has_focus() {
        ui.input_mut(|input| {
            (
                input.consume_key(egui::Modifiers::NONE, egui::Key::Enter),
                input.consume_key(egui::Modifiers::NONE, egui::Key::Space),
            )
        })
    } else {
        (false, false)
    };

    should_activate_focused_control(response.has_focus(), enter_pressed, space_pressed)
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
        let previous_date = *self.date;

        let height = config::DATE_PICKER_HEIGHT;
        let width = config::DATE_PICKER_WIDTH;
        let (rect, mut response) = ui.allocate_exact_size(
            Vec2::new(width, height),
            if self.enabled { Sense::click() } else { Sense::hover() },
        );

        let was_open_before_input = is_open;
        let keyboard_activation = calendar_keyboard_activation(ui, &response);
        if self.enabled && (response.clicked() || keyboard_activation) {
            response.request_focus();
            is_open = !is_open;
        }

        let escape_pressed = ui.input(|input| input.key_pressed(egui::Key::Escape));
        let was_open_before_escape = is_open;
        is_open = date_picker_popup_open(self.enabled, is_open, escape_pressed);
        if self.enabled && was_open_before_escape && escape_pressed {
            ui.memory_mut(|memory| memory.request_focus(response.id));
        }
        if is_open != was_open_before_input {
            response.mark_changed();
        }
        // Disabled pickers must not leave a stale popup in egui's temp state.
        if !self.enabled {
            ui.data_mut(|data| data.insert_temp(id.with("is_open"), false));
        }

        let is_hovered = self.enabled && response.hovered();
        let hover = if self.theme.reduce_motion {
            if is_hovered {
                1.0
            } else {
                0.0
            }
        } else {
            hover_t(ui.ctx(), id.with("hover"), is_hovered)
        };
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

        if self.enabled && (response.clicked() || keyboard_activation) && is_open {
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
            let popover_pos = Calendar::popup_position(rect, ui.ctx().screen_rect(), false);
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
                ui.memory_mut(|memory| memory.request_focus(response.id));
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

        if is_open != was_open_before_input || *self.date != previous_date {
            response.mark_changed();
        }
        ui.data_mut(|d| d.insert_temp(id.with("is_open"), is_open));
        response.widget_info(|| {
            let label = self
                .date
                .map(|date| date.to_iso_string())
                .unwrap_or_else(|| self.placeholder.to_owned());
            WidgetInfo::selected(WidgetType::Button, self.enabled, is_open, &label)
        });
        if response.has_focus() {
            crate::components::interact::paint_focus_ring(ui, rect, config::CONTROL_RADIUS, self.theme);
        }

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
        assert_eq!(calendar_content_width(), 248.0);
        assert_eq!(calendar_frame_size(false), Vec2::new(272.0, 292.0));
    }

    #[test]
    fn calendar_layout_shrinks_cells_when_the_gallery_column_is_narrow() {
        let layout = calendar_layout(236.0);

        assert!(layout.cell_size < config::CELL_SIZE);
        assert!(layout.frame_width <= 236.0);
        assert_eq!(
            layout.content_width,
            (layout.cell_size * 7.0) + (config::CELL_GAP * 6.0)
        );
    }

    #[test]
    fn calendar_layout_keeps_intrinsic_cells_when_space_is_available() {
        assert_eq!(
            calendar_layout(400.0),
            CalendarLayout {
                cell_size: config::CELL_SIZE,
                content_width: calendar_content_width(),
                frame_width: calendar_frame_size(false).x,
            }
        );
    }
}
