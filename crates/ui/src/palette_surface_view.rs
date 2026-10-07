use super::*;
use crate::components::{kbd_badge, Dialog};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum PaletteSurfaceAction {
    Close,
    Activate(usize),
}

pub(super) struct PaletteSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) palette: &'a mut PaletteState,
    pub(super) items: &'a [PaletteItem],
    pub(super) title: &'a str,
    pub(super) description: &'a str,
}

fn scope_filter_label(ui: &mut egui::Ui, theme: DbProTheme, selected: bool, label: &str) -> egui::Response {
    let foreground = if selected {
        ui.visuals().selection.stroke.color
    } else {
        theme.text_primary
    };
    ui.selectable_label(selected, RichText::new(label).color(foreground))
}

impl<'a> PaletteSurfaceContext<'a> {
    pub(super) fn draw(&mut self, ctx: &egui::Context) -> Vec<PaletteSurfaceAction> {
        let items = self.items;
        let title = self.title;
        let description = self.description;
        let mut activate = false;
        let mut open = true;
        Dialog::new(&mut open, title, self.theme)
            .description(description)
            .width(580.0)
            .id_salt("palette_dialog")
            .show_ctx(ctx, |ui| {
                let response = ui.add(
                    TextEdit::singleline(&mut self.palette.query)
                        .hint_text(RichText::new("Type a command or search…").color(self.theme.text_muted))
                        .desired_width(ui.available_width())
                        .margin(egui::Margin::symmetric(12.0, 8.0))
                        .font(egui::FontId::proportional(13.5))
                        .text_color(self.theme.text_primary),
                );
                if self.palette.focus_requested {
                    response.request_focus();
                    self.palette.focus_requested = false;
                }

                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    ui.style_mut().visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
                    ui.style_mut().visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
                    for scope in SearchScope::all() {
                        let selected = self.palette.scope == *scope;
                        if scope_filter_label(ui, self.theme, selected, scope.label()).clicked() {
                            self.palette.scope = *scope;
                            self.palette.selected = 0;
                        }
                    }
                });

                if ui.input(|input| input.key_pressed(egui::Key::ArrowDown)) && !items.is_empty() {
                    self.palette.selected = (self.palette.selected + 1) % items.len();
                }
                if ui.input(|input| input.key_pressed(egui::Key::ArrowUp)) && !items.is_empty() {
                    self.palette.selected = if self.palette.selected == 0 {
                        items.len() - 1
                    } else {
                        self.palette.selected - 1
                    };
                }
                if ui.input(|input| input.key_pressed(egui::Key::Enter)) && !items.is_empty() {
                    activate = true;
                }

                ui.add_space(8.0);
                egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                    if items.is_empty() {
                        ui.add_space(16.0);
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("No matching commands found").color(self.theme.text_muted));
                        });
                        ui.add_space(16.0);
                    }
                    for (index, item) in items.iter().enumerate() {
                        let selected = index == self.palette.selected;
                        let item_fill = if selected {
                            self.theme.surface_hover
                        } else {
                            egui::Color32::TRANSPARENT
                        };
                        let (rect, item_resp) =
                            ui.allocate_exact_size(egui::vec2(ui.available_width(), 44.0), egui::Sense::click());
                        if item_resp.hovered() {
                            self.palette.selected = index;
                        }
                        if item_resp.clicked() {
                            self.palette.selected = index;
                            activate = true;
                        }

                        if selected || item_resp.hovered() {
                            ui.painter().rect_filled(rect, egui::Rounding::same(6.0), item_fill);
                        }

                        // Icon
                        let icon_char = char::from(item.icon).to_string();
                        ui.painter().text(
                            egui::pos2(rect.left() + 12.0, rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            icon_char,
                            egui::FontId::new(15.0, egui::FontFamily::Name("lucide".into())),
                            if selected {
                                self.theme.text_primary
                            } else {
                                self.theme.text_secondary
                            },
                        );

                        // Title and Subtitle
                        let text_x = rect.left() + 38.0;
                        ui.painter().text(
                            egui::pos2(text_x, rect.center().y - 8.0),
                            egui::Align2::LEFT_CENTER,
                            &item.title,
                            crate::DbProTheme::ui_medium_font(13.0),
                            self.theme.text_primary,
                        );
                        ui.painter().text(
                            egui::pos2(text_x, rect.center().y + 8.0),
                            egui::Align2::LEFT_CENTER,
                            &item.subtitle,
                            egui::FontId::proportional(11.5),
                            self.theme.text_muted,
                        );

                        if let Some(shortcut) = &item.shortcut {
                            let badge_rect = egui::Rect::from_min_max(
                                egui::pos2(rect.right() - 80.0, rect.top()),
                                rect.right_bottom(),
                            );
                            let mut badge_ui = ui.new_child(
                                egui::UiBuilder::new()
                                    .max_rect(badge_rect)
                                    .layout(egui::Layout::right_to_left(egui::Align::Center)),
                            );
                            badge_ui.set_clip_rect(badge_ui.clip_rect().intersect(badge_rect));
                            badge_ui.add_space(8.0);
                            kbd_badge(&mut badge_ui, shortcut, self.theme);
                        }
                    }
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("↑↓ Navigate · ↵ Select · Esc Close")
                            .size(11.0)
                            .color(self.theme.text_muted),
                    );
                });
            });

        if !open {
            self.palette.mode = None;
        }

        let mut actions = Vec::new();
        if !open {
            actions.push(PaletteSurfaceAction::Close);
        }
        if activate && self.palette.selected < items.len() {
            actions.push(PaletteSurfaceAction::Activate(self.palette.selected));
        }
        actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_scope_label_uses_the_theme_selection_foreground() {
        for theme in [DbProTheme::light(), DbProTheme::dark()] {
            let ctx = egui::Context::default();
            theme.apply(&ctx);
            let output = ctx.run(Default::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let _ = scope_filter_label(ui, theme, true, "Schema");
                });
            });

            let selected_foreground = ctx.style().visuals.selection.stroke.color;
            let actual_foreground = output.shapes.iter().find_map(|clipped| match &clipped.shape {
                egui::Shape::Text(text) if text.galley.job.text == "Schema" => {
                    text.galley.job.sections.first().map(|section| section.format.color)
                }
                _ => None,
            });

            assert_eq!(actual_foreground, Some(selected_foreground), "dark_mode={}", theme.dark_mode);
        }
    }
}
