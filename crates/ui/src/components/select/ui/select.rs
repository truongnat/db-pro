use egui::{Frame, Id, Rect, Response, RichText, Sense, Stroke, Ui};
use lucide_icons::Icon;
use std::borrow::Cow;

use crate::components::animation::{hover_t, lerp_color};
use crate::components::interact::{combo_box_info, paint_focus_ring};
use crate::components::overlay::{floating_surface, screen_rect};
use crate::DbProTheme;

use super::super::handler::{
    calculate_menu_geometry, menu_min_content_width, menu_surface_margin, navigate_selection, selected_label,
    selection_from_click, should_close_for_key, should_close_on_outside_click, should_request_load_more,
    should_toggle_popup, trigger_accessibility_label, trigger_content_width, trigger_inner_margin, trigger_text_width,
    NavigationKeys, PopupBounds,
};
use super::option::{paint_option, SelectOption};
use crate::tokens::{FONT_SIZE_CAPTION, FONT_SIZE_SECTION_TITLE, ICON_TEXT_GAP, SPACE_XS};

// Block comment: The selected value is an index into the borrowed options slice. The optional
// load-more row is deliberately outside that index domain, so activating it requests data without
// changing selection or dismissing the popup.
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
                ui.add(
                    egui::Label::new(
                        RichText::new(lbl.as_ref())
                            .font(DbProTheme::ui_medium_font(FONT_SIZE_CAPTION))
                            .color(self.theme.text_secondary),
                    )
                    .halign(egui::Align::Min),
                );
                ui.add_space(SPACE_XS);
            }

            let current_text = selected_label(self.options, *self.selected);

            // Comment: Stable identity preserves egui popup state across frames.
            let popup_id = Id::new(self.id_salt);
            let is_open = ui.memory(|mem| mem.is_popup_open(popup_id));

            let trigger_btn = Frame {
                fill: self.theme.surface_editor,
                stroke: Stroke::new(crate::tokens::STROKE_THIN, self.theme.border_default),
                inner_margin: trigger_inner_margin(ui.spacing().button_padding.x, ui.spacing().button_padding.y),
                rounding: ui.style().visuals.widgets.inactive.rounding,
                ..Default::default()
            }
            .show(ui, |ui| {
                let content_width = trigger_content_width(width, ui.available_width());
                ui.set_width(content_width);
                ui.horizontal(|ui| {
                    let icon = if is_open { Icon::ChevronUp } else { Icon::ChevronDown };
                    let text_width = trigger_text_width(ui.available_width());
                    let text_response = ui.allocate_ui_with_layout(
                        egui::vec2(text_width, FONT_SIZE_SECTION_TITLE),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |text_ui| {
                            text_ui.set_min_width(text_width);
                            text_ui.add(
                                egui::Label::new(
                                    RichText::new(current_text)
                                        .size(crate::tokens::FONT_SIZE_UI_LABEL)
                                        .color(self.theme.text_primary),
                                )
                                .halign(egui::Align::Min)
                                .truncate(),
                            )
                        },
                    );
                    text_response.inner.on_hover_text(current_text);
                    ui.add_space(ICON_TEXT_GAP);
                    ui.label(
                        RichText::new(char::from(icon).to_string())
                            .font(crate::tokens::font_icon(crate::tokens::ICON_SM))
                            .color(self.theme.text_muted),
                    );
                });
            })
            .response;

            let response = trigger_btn.interact(Sense::click());
            let info_label = trigger_accessibility_label(self.label.as_deref(), current_text);
            response.widget_info(|| combo_box_info(true, &info_label));
            // Comment: Keyboard activation requires trigger focus; pointer clicks are independent.
            let keyboard_toggle =
                ui.input(|input| input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space));
            if should_toggle_popup(response.clicked(), response.has_focus(), keyboard_toggle) {
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
            let rounding = ui.style().visuals.widgets.inactive.rounding;
            ui.painter()
                .rect_stroke(response.rect, rounding, Stroke::new(crate::tokens::STROKE_THIN, border));
            if response.has_focus() {
                paint_focus_ring(ui, response.rect, rounding.nw, self.theme);
            }

            if ui.memory(|mem| mem.is_popup_open(popup_id)) {
                self.show_menu(ui, popup_id, response.rect);
            }

            response
        })
        .inner
    }

    fn show_menu(self, ui: &mut Ui, popup_id: Id, parent_rect: Rect) {
        // Flow comment:
        // 1. Read keyboard input, navigate real options, and close on Enter/Escape.
        // 2. Position the popup, then render options and the separate load-more action.
        // 3. Dismiss only when a click lands outside both trigger and popup.
        let key_actions = ui.input(|input| {
            (
                input.key_pressed(egui::Key::Escape),
                input.key_pressed(egui::Key::Enter),
                input.key_pressed(egui::Key::ArrowDown),
                input.key_pressed(egui::Key::ArrowUp),
            )
        });
        // Comment: Navigation is bounded to real options and does not wrap.
        *self.selected = navigate_selection(
            *self.selected,
            self.options.len(),
            NavigationKeys {
                arrow_down: key_actions.2,
                arrow_up: key_actions.3,
            },
        );
        if should_close_for_key(key_actions.0, key_actions.1) {
            ui.memory_mut(|mem| mem.close_popup());
        }

        let screen = screen_rect(ui);
        // Block comment: Geometry is resolved before creating the foreground area, allowing the menu
        // to flip or clamp within available screen space.
        let geo = calculate_menu_geometry(screen, parent_rect, self.options.len(), self.has_more);

        let area_resp = egui::Area::new(popup_id)
            .fixed_pos(geo.menu_pos)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                floating_surface(
                    self.theme,
                    ui.style().visuals.window_rounding.nw,
                    menu_surface_margin(ui.spacing().menu_margin.left),
                )
                .show(ui, |ui| {
                    ui.set_min_width(menu_min_content_width(geo.menu_width));
                    ui.set_max_width(geo.menu_width);
                    egui::ScrollArea::vertical()
                        .id_salt(popup_id.with("scroll"))
                        .max_height(geo.max_height)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            for (idx, opt) in self.options.iter().enumerate() {
                                let clicked = paint_option(
                                    ui,
                                    SelectOption {
                                        label: opt,
                                        selected: idx == *self.selected,
                                        theme: self.theme,
                                    },
                                )
                                .clicked();
                                if let Some(selected) = selection_from_click(clicked, idx) {
                                    *self.selected = selected;
                                    ui.memory_mut(|mem| mem.close_popup());
                                }
                            }
                            // Comment: This action row leaves selection and popup state unchanged.
                            if self.has_more {
                                let load = paint_option(
                                    ui,
                                    SelectOption {
                                        label: "Load more…",
                                        selected: false,
                                        theme: self.theme,
                                    },
                                );
                                if should_request_load_more(load.clicked(), self.has_more) {
                                    if let Some(flag) = self.load_more {
                                        *flag = true;
                                    }
                                }
                            }
                        });
                });
            });

        // Comment: Test both rectangles to preserve clicks on the trigger and inside the popup.
        if let Some(pos) = ui.input(|input| input.pointer.interact_pos()) {
            if should_close_on_outside_click(
                ui.input(|input| input.pointer.any_click()),
                pos,
                PopupBounds {
                    trigger: parent_rect,
                    menu: area_resp.response.rect,
                },
            ) {
                ui.memory_mut(|mem| mem.close_popup());
            }
        }
    }
}
