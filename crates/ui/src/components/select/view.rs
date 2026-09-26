use egui::{FontFamily, FontId, Frame, Id, Margin, Rect, Response, RichText, Rounding, Sense, Stroke, Ui};
use lucide_icons::Icon;
use std::borrow::Cow;

use crate::components::animation::{hover_t, lerp_color};
use crate::components::interact::{combo_box_info, paint_focus_ring};
use crate::components::overlay::{floating_surface, screen_rect};
use crate::DbProTheme;

use super::config::MENU_PAD;
use super::layout::calculate_menu_geometry;
use super::option::{paint_option, SelectOption};

pub struct Select<'a> {
    id_salt: &'a str,
    label: Option<Cow<'a, str>>,
    selected: &'a mut usize,
    options: &'a [String],
    width: Option<f32>,
    has_more: bool,
    load_more: Option<&'a mut bool>,
    theme: DbProTheme,
}

impl<'a> Select<'a> {
    pub fn new(id_salt: &'a str, selected: &'a mut usize, options: &'a [String]) -> Self {
        Self {
            id_salt,
            label: None,
            selected,
            options,
            width: None,
            has_more: false,
            load_more: None,
            theme: DbProTheme::default(),
        }
    }

    pub fn theme(mut self, theme: DbProTheme) -> Self {
        self.theme = theme;
        self
    }

    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn has_more(mut self, has_more: bool) -> Self {
        self.has_more = has_more;
        self
    }

    pub fn load_more(mut self, requested: &'a mut bool) -> Self {
        self.load_more = Some(requested);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let width = self.width.unwrap_or_else(|| ui.available_width());

        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.set_width(width);
            ui.set_max_width(width);
            if let Some(ref lbl) = self.label {
                ui.label(
                    RichText::new(lbl.as_ref())
                        .font(DbProTheme::ui_medium_font(12.0))
                        .color(self.theme.text_secondary),
                );
                ui.add_space(4.0);
            }

            let current_text = self
                .options
                .get(*self.selected)
                .map(|s| s.as_str())
                .unwrap_or("Select an option...");

            let popup_id = Id::new(self.id_salt);
            let is_open = ui.memory(|mem| mem.is_popup_open(popup_id));

            let trigger_btn = Frame {
                fill: self.theme.surface_editor,
                stroke: Stroke::new(1.0, self.theme.border_default),
                inner_margin: Margin::symmetric(10.0, 6.0),
                rounding: Rounding::same(6.0),
                ..Default::default()
            }
            .show(ui, |ui| {
                let content_width = (width - 20.0).min(ui.available_width()).max(80.0);
                ui.set_width(content_width);
                ui.horizontal(|ui| {
                    let icon = if is_open { Icon::ChevronUp } else { Icon::ChevronDown };
                    let text_width = (ui.available_width() - 22.0).max(32.0);
                    ui.add_sized(
                        [text_width, 18.0],
                        egui::Label::new(RichText::new(current_text).size(13.0).color(self.theme.text_primary))
                            .truncate(),
                    )
                    .on_hover_text(current_text);
                    ui.label(
                        RichText::new(char::from(icon).to_string())
                            .font(FontId::new(13.0, FontFamily::Name("lucide".into())))
                            .color(self.theme.text_muted),
                    );
                });
            })
            .response;

            let response = trigger_btn.interact(Sense::click());
            let info_label = self
                .label
                .as_deref()
                .map(|label| format!("{label}: {current_text}"))
                .unwrap_or_else(|| current_text.to_owned());
            response.widget_info(|| combo_box_info(true, &info_label));
            let keyboard_open = response.has_focus()
                && ui.input(|input| input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space));
            if response.clicked() || keyboard_open {
                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
            }

            let hover = hover_t(
                ui.ctx(),
                response.id.with("hover"),
                response.hovered() || response.has_focus() || is_open,
            );
            let border = if is_open || response.has_focus() {
                self.theme.accent
            } else {
                lerp_color(self.theme.border_default, self.theme.border_strong, hover)
            };
            ui.painter()
                .rect_stroke(response.rect, Rounding::same(6.0), Stroke::new(1.0, border));
            if response.has_focus() {
                paint_focus_ring(ui, response.rect, 6.0, self.theme);
            }

            if ui.memory(|mem| mem.is_popup_open(popup_id)) {
                self.show_menu(ui, popup_id, response.rect);
            }

            response
        })
        .inner
    }

    fn show_menu(self, ui: &mut Ui, popup_id: Id, parent_rect: Rect) {
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            ui.memory_mut(|mem| mem.close_popup());
        }
        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) && *self.selected + 1 < self.options.len() {
            *self.selected += 1;
        }
        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) && *self.selected > 0 {
            *self.selected -= 1;
        }
        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            ui.memory_mut(|mem| mem.close_popup());
        }

        let screen = screen_rect(ui);
        let geo = calculate_menu_geometry(screen, parent_rect, self.options.len(), self.has_more);

        let area_resp = egui::Area::new(popup_id)
            .fixed_pos(geo.menu_pos)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                floating_surface(self.theme, 10.0, Margin::symmetric(4.0, MENU_PAD)).show(ui, |ui| {
                    ui.set_min_width(geo.menu_width - 8.0);
                    ui.set_max_width(geo.menu_width);
                    egui::ScrollArea::vertical()
                        .id_salt(popup_id.with("scroll"))
                        .max_height(geo.max_height)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            for (idx, opt) in self.options.iter().enumerate() {
                                if paint_option(
                                    ui,
                                    SelectOption {
                                        label: opt,
                                        selected: idx == *self.selected,
                                        theme: self.theme,
                                    },
                                )
                                .clicked()
                                {
                                    *self.selected = idx;
                                    ui.memory_mut(|mem| mem.close_popup());
                                }
                            }
                            if self.has_more {
                                let load = paint_option(
                                    ui,
                                    SelectOption {
                                        label: "Load more…",
                                        selected: false,
                                        theme: self.theme,
                                    },
                                );
                                if load.clicked() {
                                    if let Some(flag) = self.load_more {
                                        *flag = true;
                                    }
                                }
                            }
                        });
                });
            });

        if ui.input(|i| i.pointer.any_click()) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                if !parent_rect.contains(pos) && !area_resp.response.rect.contains(pos) {
                    ui.memory_mut(|mem| mem.close_popup());
                }
            }
        }
    }
}
