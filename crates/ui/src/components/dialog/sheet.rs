use egui::{Area, Frame, Id, Margin, Order, Pos2, Rect, RichText, Rounding, Stroke, Ui};
use std::hash::Hash;

use crate::components::animation::overlay_t;
use crate::components::overlay::screen_rect;
use crate::DbProTheme;

use super::config::{SHEET_TRANSLATE_PX, SHEET_WIDTH};
use super::layout::{overlay_widget_id, paint_dim, OverlayPaint};
use super::modal::close_icon_button;

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
        let id = overlay_widget_id(ui, self.id_salt, "sheet");
        let progress = overlay_t(ui.ctx(), id.with("motion"), *self.open);
        if progress <= 0.0 {
            return None;
        }

        if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            *self.open = false;
        }

        let screen = screen_rect(ui);
        let mut inner = None;
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

        Area::new(id)
            .order(Order::Foreground)
            .fixed_pos(screen.min)
            .interactable(true)
            .show(ui.ctx(), |ui| {
                ui.set_min_size(screen.size());
                paint_dim(
                    ui,
                    OverlayPaint {
                        screen,
                        overlay: theme.overlay,
                        progress,
                    },
                );
                let sheet_rect = Rect::from_min_max(Pos2::new(x, screen.top()), screen.max);
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(sheet_rect), |ui| {
                    ui.set_width(width);
                    ui.set_min_height(screen.height());
                    let rounding = Rounding {
                        nw: 12.0,
                        ne: 0.0,
                        sw: 12.0,
                        se: 0.0,
                    };
                    Frame {
                        fill: theme.surface_floating,
                        stroke: Stroke::new(1.0, theme.border_subtle),
                        inner_margin: Margin::same(16.0),
                        rounding,
                        shadow: theme.floating_shadow(),
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        let inner_w = (width - 32.0).max(80.0);
                        ui.set_width(inner_w);
                        ui.set_max_width(inner_w);
                        ui.set_min_height((screen.height() - 32.0).max(80.0));
                        ui.horizontal(|ui| {
                            let close_w = 28.0;
                            let title_w = (inner_w - close_w - 8.0).max(40.0);
                            ui.allocate_ui_with_layout(
                                egui::vec2(title_w, 0.0),
                                egui::Layout::top_down(egui::Align::LEFT),
                                |ui| {
                                    ui.set_width(title_w);
                                    ui.set_max_width(title_w);
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
            });
        inner
    }
}
