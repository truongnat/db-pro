use crate::DbProTheme;
use egui::{FontFamily, FontId, Frame, Id, Margin, Pos2, Response, RichText, Rounding, Stroke, Ui, Vec2};
use lucide_icons::Icon;

pub struct Select<'a> {
    id_salt: &'a str,
    label: Option<&'a str>,
    selected: &'a mut usize,
    options: &'a [String],
    width: Option<f32>,
    theme: DbProTheme,
}

pub type ShadcnSelect<'a> = Select<'a>;

impl<'a> Select<'a> {
    pub fn new(id_salt: &'a str, selected: &'a mut usize, options: &'a [String], theme: DbProTheme) -> Self {
        Self {
            id_salt,
            label: None,
            selected,
            options,
            width: None,
            theme,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let width = self.width.unwrap_or_else(|| ui.available_width());

        ui.vertical(|ui| {
            if let Some(lbl) = self.label {
                ui.label(RichText::new(lbl).size(12.0).strong().color(self.theme.text_secondary));
                ui.add_space(3.0);
            }

            let current_text = self
                .options
                .get(*self.selected)
                .map(|s| s.as_str())
                .unwrap_or("Select an option...");

            let popup_id = Id::new(self.id_salt);
            let is_open = ui.memory(|mem| mem.is_popup_open(popup_id));

            let border_stroke = if is_open {
                Stroke::new(1.0, self.theme.accent)
            } else {
                Stroke::new(1.0, self.theme.border_default)
            };

            let trigger_btn = Frame {
                fill: self.theme.surface_editor,
                stroke: border_stroke,
                inner_margin: Margin::symmetric(10.0, 6.0),
                rounding: Rounding::same(6.0),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.set_min_width(width - 20.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(current_text).size(13.0).color(self.theme.text_primary));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let icon = if is_open { Icon::ChevronUp } else { Icon::ChevronDown };
                        ui.label(
                            RichText::new(char::from(icon).to_string())
                                .font(FontId::new(13.0, FontFamily::Name("lucide".into())))
                                .color(self.theme.text_muted),
                        );
                    });
                })
            })
            .response;

            let response = trigger_btn.interact(egui::Sense::click());
            if response.clicked() {
                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
            }

            // Keyboard navigation on trigger button
            if response.has_focus() {
                ui.painter()
                    .rect_stroke(response.rect, Rounding::same(6.0), Stroke::new(1.5, self.theme.accent));
                if ui.input(|i| i.key_pressed(egui::Key::Space) || i.key_pressed(egui::Key::Enter)) {
                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                }
            }

            if ui.memory(|mem| mem.is_popup_open(popup_id)) {
                let parent_rect = response.rect;
                let menu_pos = parent_rect.left_bottom() + egui::vec2(0.0, 4.0);

                // Keyboard cycling when popup is open
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

                let area_resp = egui::Area::new(popup_id)
                    .fixed_pos(menu_pos)
                    .order(egui::Order::Foreground)
                    .show(ui.ctx(), |ui| {
                        Frame {
                            fill: self.theme.surface_floating,
                            stroke: Stroke::new(1.0, self.theme.border_default),
                            inner_margin: Margin::symmetric(4.0, 4.0),
                            rounding: Rounding::same(8.0),
                            shadow: egui::epaint::Shadow {
                                offset: egui::vec2(0.0, 4.0),
                                blur: 16.0,
                                spread: 0.0,
                                color: egui::Color32::from_black_alpha(20),
                            },
                            ..Default::default()
                        }
                        .show(ui, |ui| {
                            ui.set_min_width(parent_rect.width() - 8.0);
                            for (idx, opt) in self.options.iter().enumerate() {
                                let is_selected = idx == *self.selected;
                                let item_w = ui.available_width();
                                let (item_rect, item_resp) =
                                    ui.allocate_exact_size(Vec2::new(item_w, 28.0), egui::Sense::click());

                                let bg = if is_selected {
                                    self.theme.accent_soft
                                } else if item_resp.hovered() {
                                    self.theme.surface_hover
                                } else {
                                    egui::Color32::TRANSPARENT
                                };
                                ui.painter().rect_filled(item_rect, Rounding::same(4.0), bg);

                                let text_color = if is_selected {
                                    self.theme.accent
                                } else {
                                    self.theme.text_primary
                                };
                                let text_pos = Pos2::new(item_rect.left() + 8.0, item_rect.center().y - 6.0);
                                ui.painter().text(
                                    text_pos,
                                    egui::Align2::LEFT_TOP,
                                    opt,
                                    FontId::proportional(12.5),
                                    text_color,
                                );

                                if is_selected {
                                    let check_pos = Pos2::new(item_rect.right() - 8.0, item_rect.center().y);
                                    ui.painter().text(
                                        check_pos,
                                        egui::Align2::RIGHT_CENTER,
                                        char::from(Icon::Check).to_string(),
                                        FontId::new(12.0, FontFamily::Name("lucide".into())),
                                        self.theme.accent,
                                    );
                                }

                                if item_resp.clicked() {
                                    *self.selected = idx;
                                    ui.memory_mut(|mem| mem.close_popup());
                                }
                            }
                        })
                    });

                // Safe click outside to close
                if ui.input(|i| i.pointer.any_click()) {
                    if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                        if !parent_rect.contains(pos) && !area_resp.response.rect.contains(pos) {
                            ui.memory_mut(|mem| mem.close_popup());
                        }
                    }
                }
            }

            response
        })
        .inner
    }
}
