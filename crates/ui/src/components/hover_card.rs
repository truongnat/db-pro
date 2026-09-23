use crate::DbProTheme;
use egui::{Area, Color32, Frame, Margin, Order, Pos2, Response, Rounding, Stroke, Ui};

pub struct HoverCard<'a> {
    id: &'a str,
    open_delay: f64,
    close_delay: f64,
    width: f32,
    theme: DbProTheme,
}

impl<'a> HoverCard<'a> {
    pub fn new(id: &'a str, theme: DbProTheme) -> Self {
        Self {
            id,
            open_delay: 0.20,
            close_delay: 0.15,
            width: 300.0,
            theme,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn open_delay(mut self, delay: f64) -> Self {
        self.open_delay = delay;
        self
    }

    pub fn close_delay(mut self, delay: f64) -> Self {
        self.close_delay = delay;
        self
    }

    /// Shows hover card trigger and content popup when hovered.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        trigger: impl FnOnce(&mut Ui) -> Response,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> (Response, Option<R>) {
        let trigger_resp = trigger(ui);
        let id = ui.id().with(("hover_card", self.id));

        let now = ui.input(|i| i.time);
        let mut hover_start = ui.data(|d| d.get_temp::<f64>(id.with("hover_start")));
        let mut leave_start = ui.data(|d| d.get_temp::<f64>(id.with("leave_start")));
        let was_open = ui.data(|d| d.get_temp::<bool>(id.with("is_open"))).unwrap_or(false);

        let card_hovered = ui
            .data(|d| d.get_temp::<bool>(id.with("card_hovered")))
            .unwrap_or(false);
        let is_any_hovered = trigger_resp.hovered() || card_hovered;

        if is_any_hovered {
            leave_start = None;
            if hover_start.is_none() {
                hover_start = Some(now);
            }
        } else {
            hover_start = None;
            if was_open && leave_start.is_none() {
                leave_start = Some(now);
            }
        }

        let is_open = if is_any_hovered {
            let elapsed = hover_start.map(|t| now - t).unwrap_or(0.0);
            elapsed >= self.open_delay || was_open
        } else if was_open {
            let elapsed = leave_start.map(|t| now - t).unwrap_or(0.0);
            elapsed < self.close_delay
        } else {
            false
        };

        ui.data_mut(|d| {
            d.insert_temp(id.with("hover_start"), hover_start);
            d.insert_temp(id.with("leave_start"), leave_start);
            d.insert_temp(id.with("is_open"), is_open);
        });

        let mut content_result = None;

        if is_open {
            ui.ctx().request_repaint();

            // Position below trigger rect with slight margin
            let trigger_rect = trigger_resp.rect;
            let screen_rect = ui.ctx().screen_rect();
            let mut pos = Pos2::new(trigger_rect.left(), trigger_rect.bottom() + 6.0);

            // Clamp inside screen bounds
            if pos.x + self.width > screen_rect.right() - 10.0 {
                pos.x = (screen_rect.right() - self.width - 10.0).max(10.0);
            }

            Area::new(id.with("area"))
                .order(Order::Tooltip)
                .fixed_pos(pos)
                .show(ui.ctx(), |ui| {
                    let frame = Frame {
                        fill: self.theme.surface_floating,
                        stroke: Stroke::new(1.0, self.theme.border_subtle),
                        inner_margin: Margin::same(14.0),
                        rounding: Rounding::same(8.0),
                        shadow: egui::epaint::Shadow {
                            offset: egui::vec2(0.0, 4.0),
                            blur: 16.0,
                            spread: 0.0,
                            color: Color32::from_black_alpha(40),
                        },
                        ..Default::default()
                    };

                    let response = frame.show(ui, |ui| {
                        ui.set_width(self.width);
                        content(ui)
                    });

                    let card_rect = response.response.rect;
                    let is_hovered = ui.rect_contains_pointer(card_rect);
                    ui.data_mut(|d| {
                        d.insert_temp(id.with("card_hovered"), is_hovered);
                    });

                    content_result = Some(response.inner);
                });
        } else {
            ui.data_mut(|d| {
                d.insert_temp(id.with("card_hovered"), false);
            });
        }

        (trigger_resp, content_result)
    }
}
