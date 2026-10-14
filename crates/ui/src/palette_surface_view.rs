// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
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
                let response = crate::components::input::SearchInput::new(
                    &mut self.palette.query,
                    "Type a command or search…",
                    self.theme,
                )
                .show(ui);
                if self.palette.focus_requested {
                    response.request_focus();
                    self.palette.focus_requested = false;
                }

                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    let scopes = SearchScope::all();
                    let mut scope_idx = scopes
                        .iter()
                        .position(|scope| *scope == self.palette.scope)
                        .unwrap_or(0);
                    let idx_before = scope_idx;
                    let scope_labels: Vec<&str> = scopes.iter().map(|scope| scope.label()).collect();
                    crate::components::SegmentedTabs::new(&mut scope_idx, &scope_labels, self.theme).show(ui);
                    if scope_idx != idx_before && scope_idx < scopes.len() {
                        self.palette.scope = scopes[scope_idx];
                        self.palette.selected = 0;
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
                ScrollArea::new(self.theme).max_height(320.0).show(ui, |ui| {
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
                        item_resp.widget_info(|| {
                            egui::WidgetInfo::selected(
                                egui::WidgetType::SelectableLabel,
                                ui.is_enabled(),
                                selected,
                                &item.title,
                            )
                        });
                        if item_resp.hovered() {
                            self.palette.selected = index;
                        }
                        if item_resp.clicked() {
                            self.palette.selected = index;
                            activate = true;
                        }

                        if selected || item_resp.hovered() {
                            ui.painter().rect_filled(rect, egui::CornerRadius::same(6.0 as u8), item_fill);
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

