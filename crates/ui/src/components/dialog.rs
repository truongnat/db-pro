use crate::components::animation::{faded_overlay, overlay_t, small_translate};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::overlay::screen_rect;
use crate::DbProTheme;
use egui::{Area, Frame, Id, Margin, Order, Pos2, Rect, Response, RichText, Rounding, Stroke, Ui};
use lucide_icons::Icon;
use std::hash::Hash;

const DIALOG_RADIUS: f32 = 16.0;
const DIALOG_WIDTH: f32 = 420.0;
const SHEET_WIDTH: f32 = 360.0;
const DIALOG_TRANSLATE_PX: f32 = 8.0;
const SHEET_TRANSLATE_PX: f32 = 16.0;

pub struct Dialog<'a> {
    open: &'a mut bool,
    title: &'a str,
    description: Option<&'a str>,
    width: f32,
    id_salt: Option<Id>,
    theme: DbProTheme,
}

impl<'a> Dialog<'a> {
    pub fn new(open: &'a mut bool, title: &'a str, theme: DbProTheme) -> Self {
        Self {
            open,
            title,
            description: None,
            width: DIALOG_WIDTH,
            id_salt: None,
            theme,
        }
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
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
        let id = overlay_widget_id(ui, self.id_salt, "dialog");
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
        let width = self.width;
        let title = self.title;
        let description = self.description;
        let open = self.open;

        let origin = Pos2::new(
            screen.center().x - width * 0.5,
            (screen.center().y - 120.0).max(screen.top() + 40.0) + small_translate(progress, DIALOG_TRANSLATE_PX),
        );

        let mut card_rect = None;
        Area::new(id.with("card"))
            .order(Order::Tooltip)
            .fixed_pos(origin)
            .show(ui.ctx(), |ui| {
                ui.set_width(width);
                let res = paint_dialog_card(ui, open, title, description, width, theme, add_contents);
                card_rect = Some(ui.min_rect());
                inner = Some(res);
            });

        let dim_resp = Area::new(id.with("dim"))
            .order(Order::Foreground)
            .fixed_pos(screen.min)
            .interactable(true)
            .show(ui.ctx(), |ui| {
                ui.set_min_size(screen.size());
                let resp = ui.allocate_response(screen.size(), egui::Sense::click());
                paint_dim(ui, screen, theme.overlay, progress);
                resp
            });

        if dim_resp.inner.clicked() {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                if !card_rect.is_some_and(|r| r.contains(pos)) {
                    *open = false;
                }
            } else {
                *open = false;
            }
        }

        inner
    }
}

fn paint_dialog_card<R>(
    ui: &mut Ui,
    open: &mut bool,
    title: &str,
    description: Option<&str>,
    width: f32,
    theme: DbProTheme,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    Frame {
        fill: theme.surface_floating,
        stroke: Stroke::new(1.0, theme.border_subtle),
        inner_margin: Margin::same(20.0),
        rounding: Rounding::same(DIALOG_RADIUS),
        shadow: theme.floating_shadow(),
        ..Default::default()
    }
    .show(ui, |ui| {
        ui.set_width((width - 40.0).max(80.0));
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(title).size(16.0).strong().color(theme.text_primary));
                if let Some(description) = description {
                    ui.add_space(4.0);
                    ui.add(egui::Label::new(RichText::new(description).size(13.0).color(theme.text_secondary)).wrap());
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if close_icon_button(ui, theme).clicked() {
                    *open = false;
                }
            });
        });
        ui.add_space(16.0);
        add_contents(ui)
    })
    .inner
}

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
        let width = self.width;
        let title = self.title;
        let open = self.open;
        let x = screen.right() - width + small_translate(progress, SHEET_TRANSLATE_PX);

        Area::new(id)
            .order(Order::Foreground)
            .fixed_pos(screen.min)
            .interactable(true)
            .show(ui.ctx(), |ui| {
                ui.set_min_size(screen.size());
                paint_dim(ui, screen, theme.overlay, progress);
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
                        ui.set_width((width - 32.0).max(80.0));
                        ui.set_min_height((screen.height() - 32.0).max(80.0));
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(title).size(16.0).strong().color(theme.text_primary));
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

fn overlay_widget_id(ui: &mut Ui, salt: Option<Id>, kind: &'static str) -> Id {
    if let Some(salt) = salt {
        return salt;
    }
    let id = ui.auto_id_with(kind);
    ui.skip_ahead_auto_ids(1);
    id
}

fn paint_dim(ui: &mut Ui, screen: Rect, overlay: egui::Color32, progress: f32) {
    ui.painter()
        .rect_filled(screen, Rounding::ZERO, faded_overlay(overlay, progress));
}

fn close_icon_button(ui: &mut Ui, theme: DbProTheme) -> Response {
    Button::new(theme)
        .icon(Icon::X)
        .size(ButtonSize::IconSm)
        .variant(ButtonVariant::Ghost)
        .show(ui)
}

pub fn dialog_actions(ui: &mut Ui, theme: DbProTheme, secondary: &str, primary: &str) -> (bool, bool) {
    let mut secondary_clicked = false;
    let mut primary_clicked = false;
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        primary_clicked = Button::new(theme).text(primary).show(ui).clicked();
        ui.add_space(8.0);
        secondary_clicked = Button::new(theme)
            .text(secondary)
            .variant(ButtonVariant::Ghost)
            .show(ui)
            .clicked();
    });
    (secondary_clicked, primary_clicked)
}
