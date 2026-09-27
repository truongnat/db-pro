use super::config::{
    DEFAULT_CLOSE_DELAY, DEFAULT_OPEN_DELAY, DEFAULT_WIDTH, FRAME_INNER_MARGIN, FRAME_ROUNDING, SHADOW_ALPHA,
    SHADOW_BLUR, SHADOW_OFFSET_Y,
};
use super::handler::{calculate_card_position, clamp_card_width, compute_open_state};
use crate::DbProTheme;
use egui::{Area, Color32, Frame, Margin, Order, Response, Rounding, ScrollArea, Stroke, Ui};

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
        self.open_delay = delay;
        self
    }

    pub fn close_delay(mut self, delay: f64) -> Self {
        self.close_delay = delay;
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
        // Accessibility: support both pointer hover and keyboard focus
        let is_any_active = trigger_resp.hovered() || trigger_resp.has_focus() || card_hovered;

        let (is_open, next_hover_start, next_leave_start) = compute_open_state(
            is_any_active,
            was_open,
            hover_start,
            leave_start,
            now,
            self.open_delay,
            self.close_delay,
        );

        ui.data_mut(|d| {
            d.insert_temp(id.with("hover_start"), next_hover_start);
            d.insert_temp(id.with("leave_start"), next_leave_start);
            d.insert_temp(id.with("is_open"), is_open);
        });

        let mut content_result = None;

        if is_open {
            ui.ctx().request_repaint();

            let screen_rect = ui.ctx().screen_rect();
            let effective_width = clamp_card_width(self.width, screen_rect.width());
            let max_card_height = (screen_rect.height() - 2.0 * super::config::SCREEN_EDGE_INSET).max(0.0);
            let pos = calculate_card_position(trigger_resp.rect, effective_width, max_card_height, screen_rect);

            Area::new(id.with("area"))
                .order(Order::Tooltip)
                .fixed_pos(pos)
                .show(ui.ctx(), |ui| {
                    let frame = Frame {
                        fill: self.theme.surface_floating,
                        stroke: Stroke::new(1.0, self.theme.border_subtle),
                        inner_margin: Margin::same(FRAME_INNER_MARGIN),
                        rounding: Rounding::same(FRAME_ROUNDING),
                        shadow: egui::epaint::Shadow {
                            offset: egui::vec2(0.0, SHADOW_OFFSET_Y),
                            blur: SHADOW_BLUR,
                            spread: 0.0,
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
