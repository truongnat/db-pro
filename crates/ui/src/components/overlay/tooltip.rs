// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use crate::components::animation::fade_alpha;
use crate::components::feedback::kbd_badge;
use crate::DbProTheme;
use egui::{Area, Margin, Order, Pos2, Response, RichText, Vec2};

use super::{config, ui::floating_surface};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TooltipPosition {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

pub struct Tooltip<'a> {
    text: &'a str,
    position: TooltipPosition,
    shortcut: Option<&'a str>,
    theme: DbProTheme,
}

impl<'a> Tooltip<'a> {
    pub fn new(text: &'a str, theme: DbProTheme) -> Self {
        Self {
            text,
            position: TooltipPosition::Top,
            shortcut: None,
            theme,
        }
    }

    pub fn position(mut self, position: TooltipPosition) -> Self {
        self.position = position;
        self
    }

    pub fn shortcut(mut self, shortcut: &'a str) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn show(self, response: &Response) -> Response {
        let is_hovered = response.hovered();
        let id = response.id.with("snappy_tooltip");
        let progress = response
            .ctx
            .animate_bool_with_time(id.with("motion"), is_hovered, 0.120);

        if progress > 0.0 && progress < 1.0 {
            response.ctx.request_repaint();
        }

        if progress <= 0.0 {
            return response.clone();
        }

        let ctx = &response.ctx;
        let screen = ctx.content_rect();
        let target_rect = response.rect;

        let font_id = egui::FontId::proportional(config::TOOLTIP_FONT_SIZE);
        let galley =
            ctx.fonts_mut(|fonts| fonts.layout(self.text.to_owned(), font_id.clone(), self.theme.text_primary, 260.0));

        let mut tooltip_w = galley.size().x + config::TOOLTIP_HORIZONTAL_PADDING;
        if self.shortcut.is_some() {
            tooltip_w += config::TOOLTIP_SHORTCUT_WIDTH;
        }
        let tooltip_h = (galley.size().y + config::TOOLTIP_VERTICAL_PADDING).max(config::TOOLTIP_MIN_HEIGHT);

        let motion_offset = (1.0 - progress) * 3.0;
        let effective_position = match self.position {
            TooltipPosition::Top if target_rect.top() - tooltip_h - 6.0 < screen.top() + 4.0 => TooltipPosition::Bottom,
            TooltipPosition::Bottom if target_rect.bottom() + tooltip_h + 6.0 > screen.bottom() - 4.0 => {
                TooltipPosition::Top
            }
            TooltipPosition::Left if target_rect.left() - tooltip_w - 6.0 < screen.left() + 4.0 => {
                TooltipPosition::Right
            }
            TooltipPosition::Right if target_rect.right() + tooltip_w + 6.0 > screen.right() - 4.0 => {
                TooltipPosition::Left
            }
            other => other,
        };
        let (raw_x, raw_y) = match effective_position {
            TooltipPosition::Top => (
                target_rect.center().x - tooltip_w * 0.5,
                target_rect.top() - tooltip_h - 6.0 + motion_offset,
            ),
            TooltipPosition::Bottom => (
                target_rect.center().x - tooltip_w * 0.5,
                target_rect.bottom() + 6.0 - motion_offset,
            ),
            TooltipPosition::Left => (
                target_rect.left() - tooltip_w - 6.0 + motion_offset,
                target_rect.center().y - tooltip_h * 0.5,
            ),
            TooltipPosition::Right => (
                target_rect.right() + 6.0 - motion_offset,
                target_rect.center().y - tooltip_h * 0.5,
            ),
        };

        let x = raw_x.clamp(
            screen.left() + 4.0,
            (screen.right() - tooltip_w - 4.0).max(screen.left() + 4.0),
        );
        let y = raw_y.clamp(
            screen.top() + 4.0,
            (screen.bottom() - tooltip_h - 4.0).max(screen.top() + 4.0),
        );
        let pos = Pos2::new(x, y);

        let theme = self.theme;
        let text = self.text.to_owned();
        let shortcut = self.shortcut;

        Area::new(id)
            .order(Order::Tooltip)
            .fixed_pos(pos)
            .interactable(false)
            .show(ctx, |ui| {
                ui.set_opacity(fade_alpha(progress));
                floating_surface(theme, 6.0, Margin::symmetric(8.0 as i8, 5.0 as i8)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);
                        ui.label(
                            RichText::new(text)
                                .size(config::TOOLTIP_FONT_SIZE)
                                .color(theme.text_primary),
                        );
                        if let Some(sc) = shortcut {
                            kbd_badge(ui, sc, theme);
                        }
                    });
                });
            });

        response.clone()
    }
}
