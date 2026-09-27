use egui::{Area, Frame, Id, LayerId, Margin, Order, Pos2, Rect, RichText, Rounding, Stroke, Ui};
use std::hash::Hash;

use crate::components::animation::overlay_t;
use crate::components::overlay::screen_rect;
use crate::DbProTheme;

use super::config::{SHEET_TRANSLATE_PX, SHEET_WIDTH};
use super::handler::{modal_dismissal, ModalDismissal};
use super::layout::{overlay_widget_id, paint_dim, OverlayPaint};
use super::modal_guard;
use super::ui::close_icon_button;

pub struct Sheet<'a> {
    open: &'a mut bool,
    title: &'a str,
    width: f32,
    id_salt: Option<Id>,
    theme: DbProTheme,
}

impl<'a> Sheet<'a> {
    pub fn new(open: &'a mut bool, title: &'a str, theme: DbProTheme) -> Self {
        Self {
            open,
            title,
            width: SHEET_WIDTH,
            id_salt: None,
            theme,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn id_salt(mut self, salt: impl Hash) -> Self {
        self.id_salt = Some(Id::new(salt));
        self
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> Option<R> {
        let ctx = ui.ctx().clone();
        let id = overlay_widget_id(ui, self.id_salt, "sheet");
        let progress = overlay_t(&ctx, id.with("motion"), *self.open);
        if progress <= 0.0 {
            return None;
        }

        let is_topmost = modal_guard::register(&ctx, id);
        let sheet_id = id.with("sheet");
        let dim_id = id.with("dim");
        let sheet_layer = LayerId::new(Order::Foreground, sheet_id);
        let anchor_id = id.with("focus_anchor");

        if is_topmost {
            let escape_pressed = ctx.input(|input| input.key_pressed(egui::Key::Escape));
            if modal_dismissal(true, escape_pressed, false, None, None) == Some(ModalDismissal::Escape) {
                *self.open = false;
            }
            // Match modal dialogs: the anchor catches focus before sheet content may request a field.
            modal_guard::trap_focus(&ctx, sheet_layer, anchor_id);
        }

        let screen = screen_rect(ui);
        let theme = self.theme;
        let title = self.title;
        let open = self.open;
        let (width, x) = crate::components::common_utils::calculate_sheet_layout(
            screen,
            self.width,
            progress,
            SHEET_TRANSLATE_PX,
            16.0,
        );
        let mut inner = None;
        let dim_layer = LayerId::new(Order::Foreground, dim_id);

        Area::new(dim_id)
            .order(Order::Foreground)
            .fixed_pos(screen.min)
            .interactable(true)
            .show(&ctx, |ui| {
                ui.set_min_size(screen.size());
                paint_dim(
                    ui,
                    OverlayPaint {
                        screen,
                        overlay: theme.overlay,
                        progress,
                    },
                );
                // The sheet intentionally dismisses through Escape or its close control, not backdrop clicks.
                ui.allocate_response(screen.size(), egui::Sense::click());
            });

        Area::new(sheet_id)
            .order(Order::Foreground)
            .fixed_pos(Pos2::new(x, screen.top()))
            .interactable(true)
            .show(&ctx, |ui| {
                ui.set_width(width);
                ui.set_min_height(screen.height());
                Frame {
                    fill: theme.surface_floating,
                    stroke: Stroke::new(1.0, theme.border_subtle),
                    inner_margin: Margin::same(16.0),
                    rounding: Rounding {
                        nw: 12.0,
                        ne: 0.0,
                        sw: 12.0,
                        se: 0.0,
                    },
                    shadow: theme.floating_shadow(),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    let inner_width = (width - 32.0).max(80.0);
                    ui.set_width(inner_width);
                    ui.set_max_width(inner_width);
                    ui.set_min_height((screen.height() - 32.0).max(80.0));
                    ui.interact(
                        Rect::from_min_size(ui.cursor().min, egui::Vec2::ZERO),
                        anchor_id,
                        egui::Sense::focusable_noninteractive(),
                    );
                    ui.horizontal(|ui| {
                        let close_width = 28.0;
                        let title_width = (inner_width - close_width - 8.0).max(40.0);
                        ui.allocate_ui_with_layout(
                            egui::vec2(title_width, 0.0),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                ui.set_width(title_width);
                                ui.set_max_width(title_width);
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(title).size(16.0).strong().color(theme.text_primary),
                                    )
                                    .truncate(),
                                )
                                .on_hover_text(title);
                            },
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if close_icon_button(ui, theme).clicked() {
                                *open = false;
                            }
                        });
                    });
                    ui.add_space(12.0);
                    inner = Some(add_contents(ui));
                });
            });

        ctx.set_sublayer(dim_layer, sheet_layer);
        ctx.memory_mut(|memory| {
            memory.areas_mut().move_to_top(dim_layer);
            memory.areas_mut().move_to_top(sheet_layer);
        });
        inner
    }
}
