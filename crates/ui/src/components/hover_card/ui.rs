use super::config::{
    DEFAULT_CLOSE_DELAY, DEFAULT_OPEN_DELAY, DEFAULT_WIDTH, FRAME_INNER_MARGIN, FRAME_ROUNDING, SHADOW_ALPHA,
    SHADOW_BLUR, SHADOW_OFFSET_Y,
};
use super::handler::{
    calculate_card_position, clamp_card_width, compute_open_state, escape_dismissed, sanitize_delay, timer_duration,
};
use crate::DbProTheme;
use egui::{Area, Color32, CornerRadius, Frame, Margin, Order, Response, ScrollArea, Stroke, Ui};

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
                        shadow: egui::epaint::Shadow {
                            offset: [0, SHADOW_OFFSET_Y as i8],
                            blur: SHADOW_BLUR as u8,
                            spread: 0,
                            color: Color32::from_black_alpha(SHADOW_ALPHA),
                        },
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
mod tests {
    use super::*;
    use egui::{Button, CentralPanel, Context, Event, Key, Pos2, RawInput, Rect, Vec2};

    const SCREEN_SIZE: Vec2 = Vec2::new(800.0, 600.0);

    fn trigger_rect() -> Rect {
        Rect::from_min_size(Pos2::new(100.0, 120.0), Vec2::new(120.0, 32.0))
    }

    fn run_frame(ctx: &Context, time: f64, events: Vec<Event>, content_height: f32) -> bool {
        let mut content_was_rendered = false;
        let _ = crate::test_frame::frame(
            &ctx,
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN_SIZE)),
                time: Some(time),
                events,
                ..Default::default()
            },
            |ctx| {
                CentralPanel::default().show(ctx, |ui| {
                    let (_, content_result) = HoverCard::new("escape_test", DbProTheme::light())
                        .open_delay(0.0)
                        .close_delay(0.0)
                        .show(
                            ui,
                            |ui| ui.put(trigger_rect(), Button::new("Trigger")),
                            |ui| ui.add_space(content_height),
                        );
                    content_was_rendered = content_result.is_some();
                });
            },
        );
        content_was_rendered
    }

    #[test]
    fn escape_closes_immediately_and_stays_closed_while_trigger_is_hovered() {
        let ctx = Context::default();
        DbProTheme::install_fonts(&ctx);
        let pointer = egui::Event::PointerMoved(trigger_rect().center());

        assert!(!run_frame(&ctx, -0.1, Vec::new(), 24.0));
        assert!(run_frame(&ctx, 0.0, vec![pointer], 24.0));
        assert!(!run_frame(
            &ctx,
            0.1,
            vec![
                egui::Event::PointerMoved(trigger_rect().center()),
                Event::Key {
                    key: Key::Escape,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::default(),
                },
            ],
            24.0,
        ));
        assert!(!run_frame(
            &ctx,
            0.2,
            vec![egui::Event::PointerMoved(trigger_rect().center())],
            24.0,
        ));
    }

    #[test]
    fn area_keeps_the_calculated_position_after_its_size_is_known() {
        let ctx = Context::default();
        DbProTheme::install_fonts(&ctx);
        let trigger_rect = Rect::from_min_size(Pos2::new(100.0, 300.0), Vec2::new(120.0, 32.0));
        let mut area_id = None;

        let run_sized_frame = |ctx: &Context, time: f64, content_height: f32, area_id: &mut Option<egui::Id>| {
            let mut card_area_rect = None;
            let _ = crate::test_frame::frame(
                &ctx,
                RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN_SIZE)),
                    time: Some(time),
                    events: vec![Event::PointerMoved(trigger_rect.center())],
                    ..Default::default()
                },
                |ctx| {
                    CentralPanel::default().show(ctx, |ui| {
                        *area_id = Some(ui.id().with(("hover_card", "height_test")).with("area"));
                        let _ = HoverCard::new("height_test", DbProTheme::light()).open_delay(0.0).show(
                            ui,
                            |ui| ui.put(trigger_rect, Button::new("Trigger")),
                            |ui| {
                                ui.allocate_space(Vec2::new(ui.available_width(), content_height));
                            },
                        );
                    });
                },
            );
            if let Some(id) = *area_id {
                card_area_rect = ctx.memory(|memory| memory.area_rect(id));
            }
            card_area_rect
        };

        let _ = run_sized_frame(&ctx, 0.0, 20.0, &mut area_id);
        let card_rect = run_sized_frame(&ctx, 0.1, 20.0, &mut area_id).expect("card area");
        let expected_y = trigger_rect.bottom() + super::super::config::TRIGGER_GAP;

        assert!(
            (card_rect.top() - expected_y).abs() <= 1.0,
            "card {card_rect:?} should stay at calculated y={expected_y} below trigger {trigger_rect:?}"
        );
    }
}
