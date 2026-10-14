use egui::{Area, CornerRadius, Frame, Margin, Order, Response, ScrollArea, Stroke, Ui};

use super::{
    config::{DEFAULT_CLOSE_DELAY, DEFAULT_OPEN_DELAY, DEFAULT_WIDTH, FRAME_INNER_MARGIN, FRAME_ROUNDING},
    handler::{
        calculate_card_position, clamp_card_width, compute_open_state, escape_dismissed, sanitize_delay, timer_duration,
    },
};
use crate::DbProTheme;

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
            open_delay: DEFAULT_OPEN_DELAY,
            close_delay: DEFAULT_CLOSE_DELAY,
            width: DEFAULT_WIDTH,
            theme,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn open_delay(mut self, delay: f64) -> Self {
        self.open_delay = sanitize_delay(delay);
        self
    }

    pub fn close_delay(mut self, delay: f64) -> Self {
        self.close_delay = sanitize_delay(delay);
        self
    }

    /// Shows hover card trigger and floating content popup when hovered or focused.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        trigger: impl FnOnce(&mut Ui) -> Response,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> (Response, Option<R>) {
        let trigger_resp = trigger(ui);
        let id = ui.id().with(("hover_card", self.id));

        let now = ui.input(|i| i.time);
        let hover_start = ui.data(|d| d.get_temp::<f64>(id.with("hover_start")));
        let leave_start = ui.data(|d| d.get_temp::<f64>(id.with("leave_start")));
        let was_open = ui.data(|d| d.get_temp::<bool>(id.with("is_open"))).unwrap_or(false);

        let card_hovered = ui
            .data(|d| d.get_temp::<bool>(id.with("card_hovered")))
            .unwrap_or(false);
        let mut dismissed = ui.data(|d| d.get_temp::<bool>(id.with("dismissed"))).unwrap_or(false);
        let escape_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));
        let escaped_open_card = escape_dismissed(escape_pressed, was_open);
        if escaped_open_card {
            // Apply dismissal to the current frame, rather than waiting for the
            // next frame to observe the value written to temporary data.
            dismissed = true;
            ui.data_mut(|d| {
                d.insert_temp(id.with("dismissed"), true);
                d.insert_temp(id.with("hover_start"), None::<f64>);
                d.insert_temp(id.with("leave_start"), None::<f64>);
            });
            ui.memory_mut(|memory| {
                memory.surrender_focus(trigger_resp.id);
                memory.surrender_focus(id.with("area"));
            });
        }

        let trigger_active = trigger_resp.hovered() || trigger_resp.has_focus();
        let is_active = trigger_active || card_hovered;
        if dismissed && !is_active {
            dismissed = false;
            ui.data_mut(|d| d.insert_temp(id.with("dismissed"), false));
        }
        let is_suppressed = dismissed && is_active;
        // Accessibility: support both pointer hover and keyboard focus.
        let is_any_active = is_active && !is_suppressed;

        let (is_open, next_hover_start, next_leave_start) = if escaped_open_card {
            // Escape closes immediately and clears both pending timers. This
            // also prevents compute_open_state from reopening a still-active trigger.
            (false, None, None)
        } else {
            compute_open_state(
                is_any_active,
                was_open,
                hover_start,
                leave_start,
                now,
                self.open_delay,
                self.close_delay,
            )
        };
        ui.data_mut(|d| {
            d.insert_temp(id.with("hover_start"), next_hover_start);
            d.insert_temp(id.with("leave_start"), next_leave_start);
            d.insert_temp(id.with("is_open"), is_open);
        });

        if let Some(start) = if is_open { next_leave_start } else { next_hover_start } {
            let delay = if is_open { self.close_delay } else { self.open_delay };
            let remaining = (delay - (now - start)).max(0.0);
            ui.ctx().request_repaint_after(timer_duration(remaining));
        }

        let mut content_result = None;

        if is_open {
            let screen_rect = ui.ctx().content_rect();
            let effective_width = clamp_card_width(self.width, screen_rect.width());
            let max_card_height = (screen_rect.height() - 2.0 * super::config::SCREEN_EDGE_INSET).max(0.0);
            let measured_height = ui
                .data(|d| d.get_temp::<f32>(id.with("card_height")))
                .unwrap_or(super::config::MIN_VISIBLE_HEIGHT);
            let pos = calculate_card_position(trigger_resp.rect, effective_width, measured_height, screen_rect);

            Area::new(id.with("area"))
                .order(Order::Tooltip)
                // `calculate_card_position` already accounts for the measured card
                // height and viewport edges. Letting Area constrain again uses its
                // previous frame's size and can move a newly-sized card to an
                // unrelated position before the measurement catches up.
                .constrain(false)
                .fixed_pos(pos)
                .show(ui.ctx(), |ui| {
                    let frame = Frame {
                        fill: self.theme.surface_floating,
                        stroke: Stroke::new(1.0, self.theme.border_subtle),
                        inner_margin: Margin::same(FRAME_INNER_MARGIN as i8),
                        corner_radius: CornerRadius::same(FRAME_ROUNDING as u8),
                        shadow: self.theme.floating_shadow(),
                        ..Default::default()
                    };

                    let response = frame.show(ui, |ui| {
                        let content_width = (effective_width - 2.0 * FRAME_INNER_MARGIN - 2.0).max(0.0);
                        ui.set_width(content_width);
                        ScrollArea::vertical()
                            .id_salt(id.with("scroll"))
                            .max_height(max_card_height)
                            .show(ui, |ui| content(ui))
                            .inner
                    });

                    let card_rect = response.response.rect;
                    let new_height = card_rect.height();
                    let height_changed = ui
                        .data(|d| d.get_temp::<f32>(id.with("card_height")))
                        .map(|height| (height - new_height).abs() > f32::EPSILON)
                        .unwrap_or(true);
                    ui.data_mut(|d| d.insert_temp(id.with("card_height"), new_height));
                    if height_changed {
                        ui.ctx().request_repaint();
                    }
                    let is_hovered = ui.rect_contains_pointer(card_rect);
                    let has_focus = ui.memory(|memory| memory.has_focus(ui.id()));
                    ui.data_mut(|d| {
                        d.insert_temp(id.with("card_hovered"), is_hovered || has_focus);
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

#[cfg(test)]
#[path = "tests/ui_tests.rs"]
mod tests;
