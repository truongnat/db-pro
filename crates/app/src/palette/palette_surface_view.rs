// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use super::*;
use crate::components::Dialog;

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
            .show_framed_ctx(ctx, |frame| {
                frame.body(|ui| {
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
                    ScrollArea::new(self.theme)
                        .auto_shrink([false, true])
                        .max_height(320.0)
                        .show(ui, |ui| {
                            if items.is_empty() {
                                ui.add_space(16.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(RichText::new("No matching commands found").color(self.theme.text_muted));
                                });
                                ui.add_space(16.0);
                            }
                            for (index, item) in items.iter().enumerate() {
                                let selected = index == self.palette.selected;
                                let mut row = CommandItem::new(&item.title, &item.title)
                                    .icon(item.icon)
                                    .selected(selected);
                                if !item.subtitle.is_empty() {
                                    row = row.subtitle(&item.subtitle);
                                }
                                if let Some(shortcut) = &item.shortcut {
                                    row = row.shortcut(shortcut);
                                }
                                let item_resp = row.show(ui, self.theme);
                                if item_resp.hovered() {
                                    self.palette.selected = index;
                                }
                                if item_resp.clicked() {
                                    self.palette.selected = index;
                                    activate = true;
                                }
                            }
                        });
                });
                frame.footer(|ui| {
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
#[path = "tests/palette_surface_view_tests.rs"]
mod tests;
