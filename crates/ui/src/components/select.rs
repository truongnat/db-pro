use crate::components::animation::{hover_t, lerp_color};
use crate::components::interact::{paint_focus_ring, text_input_info};
use crate::components::overlay::{floating_surface, screen_rect};
use crate::DbProTheme;
use egui::{
    Color32, FontFamily, FontId, Frame, Id, Margin, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2,
};
use lucide_icons::Icon;

const ITEM_HEIGHT: f32 = 32.0;
const MAX_VISIBLE_ITEMS: usize = 8;
const MENU_PAD: f32 = 6.0;

pub struct Select<'a> {
    id_salt: &'a str,
    label: Option<&'a str>,
    selected: &'a mut usize,
    options: &'a [String],
    width: Option<f32>,
    has_more: bool,
    load_more: Option<&'a mut bool>,
    theme: DbProTheme,
}

impl<'a> Select<'a> {
    pub fn new(id_salt: &'a str, selected: &'a mut usize, options: &'a [String], theme: DbProTheme) -> Self {
        Self {
            id_salt,
            label: None,
            selected,
            options,
            width: None,
            has_more: false,
            load_more: None,
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

        ui.vertical(|ui| {
            if let Some(lbl) = self.label {
                ui.label(
                    RichText::new(lbl)
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
                ui.set_min_width((width - 20.0).max(80.0));
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
                });
            })
            .response;

            let response = trigger_btn.interact(Sense::click());
            response.widget_info(|| text_input_info(true, current_text));
            if response.clicked() {
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

        let extra = if self.has_more { 1 } else { 0 };
        let visible = self.options.len().saturating_add(extra).min(MAX_VISIBLE_ITEMS);
        let menu_h = visible as f32 * ITEM_HEIGHT + MENU_PAD * 2.0;
        let screen = screen_rect(ui);
        let space_below = (screen.bottom() - parent_rect.bottom()).max(0.0);
        let space_above = (parent_rect.top() - screen.top()).max(0.0);
        let open_up = dropdown_should_open_above(space_below, space_above, menu_h);
        let menu_pos = if open_up {
            Pos2::new(parent_rect.left(), parent_rect.top() - 4.0 - menu_h)
        } else {
            parent_rect.left_bottom() + Vec2::new(0.0, 4.0)
        };

        let area_resp = egui::Area::new(popup_id)
            .fixed_pos(menu_pos)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                floating_surface(self.theme, 10.0, Margin::symmetric(4.0, MENU_PAD)).show(ui, |ui| {
                    ui.set_min_width((parent_rect.width() - 8.0).max(160.0));
                    ui.set_max_width(parent_rect.width().max(160.0));
                    egui::ScrollArea::vertical()
                        .id_salt(popup_id.with("scroll"))
                        .max_height(MAX_VISIBLE_ITEMS as f32 * ITEM_HEIGHT)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            for (idx, opt) in self.options.iter().enumerate() {
                                if paint_option(ui, opt, idx == *self.selected, self.theme).clicked() {
                                    *self.selected = idx;
                                    ui.memory_mut(|mem| mem.close_popup());
                                }
                            }
                            if self.has_more {
                                let load = paint_option(ui, "Load more…", false, self.theme);
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

pub fn dropdown_should_open_above(space_below: f32, space_above: f32, menu_h: f32) -> bool {
    space_below < menu_h && space_above > space_below
}

fn paint_option(ui: &mut Ui, label: &str, selected: bool, theme: DbProTheme) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), ITEM_HEIGHT), Sense::click());
    let hover = hover_t(ui.ctx(), response.id.with("opt"), response.hovered() && !selected);
    let bg = if selected {
        theme.accent_soft
    } else {
        lerp_color(Color32::TRANSPARENT, theme.surface_hover, hover)
    };
    ui.painter().rect_filled(rect, Rounding::same(6.0), bg);
    let text_color = if selected { theme.accent } else { theme.text_primary };
    ui.painter().text(
        Pos2::new(rect.left() + 8.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.0),
        text_color,
    );
    if selected {
        ui.painter().text(
            Pos2::new(rect.right() - 8.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            char::from(Icon::Check).to_string(),
            FontId::new(12.0, FontFamily::Name("lucide".into())),
            theme.accent,
        );
    }
    response
}

#[cfg(test)]
mod tests {
    use super::dropdown_should_open_above;

    #[test]
    fn dropdown_flips_above_when_there_is_no_room_below() {
        assert!(dropdown_should_open_above(40.0, 280.0, 260.0));
        assert!(!dropdown_should_open_above(300.0, 40.0, 260.0));
        assert!(!dropdown_should_open_above(200.0, 200.0, 180.0));
    }
}
