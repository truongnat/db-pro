// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use egui::{Frame, Id, Response, RichText, Sense, Stroke, TextureId, Ui};
use lucide_icons::Icon;
use std::borrow::Cow;

use crate::components::animation::{hover_t, lerp_color};
use crate::components::interact::{combo_box_info, paint_focus_ring};
use crate::components::overlay::screen_rect;
use crate::DbProTheme;

use super::super::handler::{
    calculate_menu_geometry, menu_min_content_width, navigate_selection, selected_label, selection_from_click,
    should_close_for_key, should_request_load_more, trigger_accessibility_label, trigger_content_width,
    trigger_inner_margin, trigger_text_width, NavigationKeys,
};
use super::option::{paint_option, SelectOption};
use crate::tokens::{FONT_SIZE_CAPTION, ICON_TEXT_GAP, SPACE_XS};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SelectSize {
    #[default]
    Default,
    Sm,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SelectVariant {
    #[default]
    Outline,
    Ghost,
}

// Block comment: The selected value is an index into the borrowed options slice. The optional
// load-more row is deliberately outside that index domain, so activating it requests data without
// changing selection or dismissing the popup.
pub struct Select<'a> {
    id_salt: Id,
    label: Option<Cow<'a, str>>,
    selected: &'a mut usize,
    options: &'a [String],
    /// Overrides the trigger text so callers storing keys/values can still
    /// show a friendly label (or raw value) without reordering `options`.
    selected_text: Option<Cow<'a, str>>,
    width: Option<f32>,
    /// Per-option leading icons (e.g. brand marks); same indexing as `options`.
    option_icons: Option<&'a [Option<egui::TextureId>]>,
    /// Per-option disabled flags; same indexing as `options`.
    option_disabled: Option<&'a [bool]>,
    has_more: bool,
    load_more: Option<&'a mut bool>,
    theme: DbProTheme,
    size: SelectSize,
    variant: SelectVariant,
}

impl<'a> Select<'a> {
    /// `id_salt` accepts any hashable value; `egui::Id` salts keep a shared
    /// popup namespace with callers that build cell/row IDs dynamically.
    pub fn new(
        id_salt: impl std::hash::Hash + std::fmt::Debug,
        selected: &'a mut usize,
        options: &'a [String],
    ) -> Self {
        Self {
            id_salt: Id::new(id_salt),
            label: None,
            selected,
            options,
            selected_text: None,
            width: None,
            option_icons: None,
            option_disabled: None,
            has_more: false,
            load_more: None,
            theme: DbProTheme::default(),
            size: SelectSize::default(),
            variant: SelectVariant::default(),
        }
    }

    /// Show `text` on the trigger instead of the label at `selected`.
    /// Selection bookkeeping still uses `options`; this only changes display.
    pub fn selected_text(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.selected_text = Some(text.into());
        self
    }

    pub fn theme(mut self, theme: DbProTheme) -> Self {
        self.theme = theme;
        self
    }

    pub fn size(mut self, size: SelectSize) -> Self {
        self.size = size;
        self
    }

    pub fn variant(mut self, variant: SelectVariant) -> Self {
        self.variant = variant;
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

    /// Provide one optional texture per option (same order as `options`) to
    /// paint a leading icon in the popup rows.
    pub fn option_icons(mut self, icons: &'a [Option<TextureId>]) -> Self {
        self.option_icons = Some(icons);
        self
    }

    /// Mark per-option rows as non-interactive (e.g. "coming soon" entries).
    pub fn option_disabled(mut self, flags: &'a [bool]) -> Self {
        self.option_disabled = Some(flags);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let compact = crate::components::button::SizeTokens::from_size(crate::components::ButtonSize::Sm);
        let (height, font_size, icon_size, margin) = match self.size {
            SelectSize::Default => (
                crate::tokens::component::input::INPUT_HEIGHT_DEFAULT,
                crate::tokens::FONT_SIZE_UI_LABEL,
                crate::tokens::ICON_SM,
                trigger_inner_margin(ui.spacing().button_padding.x, ui.spacing().button_padding.y),
            ),
            SelectSize::Sm => (
                compact.min_height,
                compact.font_size,
                compact.icon_size,
                egui::Margin::symmetric(compact.padding.x as i8, compact.padding.y as i8),
            ),
        };
        // egui 0.36 counts `2 * stroke.width` in the Frame's total size; subtract it so the
        // trigger keeps its token-driven outer height. Borderless (ghost) variants need no offset.
        let stroke_extra = if self.variant == SelectVariant::Ghost {
            0.0
        } else {
            2.0 * crate::tokens::STROKE_THIN
        };
        let content_height = height - stroke_extra - margin.topf() - margin.bottomf();

        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
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

            let current_text = match &self.selected_text {
                Some(text) => text.clone().into_owned(),
                None => selected_label(self.options, *self.selected).to_owned(),
            };

            // Comment: Stable identity preserves menu open state across frames.
            let popup_id = self.id_salt;
            let is_open = ui.ctx().data_mut(|d| d.get_temp::<bool>(popup_id).unwrap_or(false));

            let ghost = self.variant == SelectVariant::Ghost;
            let rounding = egui::CornerRadius::same(crate::tokens::RADIUS_INPUT as u8);
            let background = ghost.then(|| ui.painter().add(egui::Shape::Noop));
            let trigger_btn = Frame {
                fill: if ghost {
                    egui::Color32::TRANSPARENT
                } else {
                    self.theme.surface_editor
                },
                stroke: if ghost {
                    Stroke::NONE
                } else {
                    Stroke::new(crate::tokens::STROKE_THIN, self.theme.border_default)
                },
                inner_margin: margin,
                corner_radius: rounding,
                ..Default::default()
            }
            .show(ui, |ui| {
                let content_width =
                    trigger_content_width(width - margin.leftf() - margin.rightf(), ui.available_width());
                ui.set_width(content_width);
                ui.spacing_mut().interact_size.y = content_height;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.set_min_height(content_height);
                    let icon = if is_open { Icon::ChevronUp } else { Icon::ChevronDown };
                    let text_width = trigger_text_width(ui.available_width());
                    let text_response = ui.allocate_ui_with_layout(
                        egui::vec2(text_width, content_height),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |text_ui| {
                            text_ui.set_min_width(text_width);
                            text_ui.add(
                                egui::Label::new(
                                    RichText::new(&current_text)
                                        .size(font_size)
                                        .color(self.theme.text_primary),
                                )
                                .halign(egui::Align::Min)
                                .truncate(),
                            )
                        },
                    );
                    text_response.inner.on_hover_text(&current_text);
                    ui.add_space(ICON_TEXT_GAP);
                    ui.label(
                        RichText::new(char::from(icon).to_string())
                            .font(crate::tokens::font_icon(icon_size))
                            .color(self.theme.text_muted),
                    );
                });
            })
            .response;

            let response = trigger_btn.interact(Sense::click());
            let info_label = trigger_accessibility_label(self.label.as_deref(), &current_text);
            response.widget_info(|| combo_box_info(true, &info_label));
            // Both pointer and keyboard toggles write the same temp flag;
            // `!response.clicked()` prevents a double toggle on activation keys.
            let keyboard_toggle =
                ui.input(|input| input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space));
            if response.has_focus() && keyboard_toggle && !response.clicked() {
                ui.ctx().data_mut(|d| {
                    let open = !d.get_temp::<bool>(popup_id).unwrap_or(false);
                    d.insert_temp(popup_id, open);
                });
            }
            let hover = hover_t(
                ui.ctx(),
                response.id.with("hover"),
                response.hovered() || response.has_focus() || is_open,
            );
            let border = if is_open || response.has_focus() {
                self.theme.accent
            } else if ghost {
                egui::Color32::TRANSPARENT
            } else {
                lerp_color(self.theme.border_default, self.theme.border_strong, hover)
            };
            if let Some(background) = background {
                ui.painter().set(
                    background,
                    egui::Shape::rect_filled(
                        response.rect,
                        rounding,
                        lerp_color(egui::Color32::TRANSPARENT, self.theme.surface_hover, hover),
                    ),
                );
            }
            ui.painter().rect_stroke(
                response.rect,
                rounding,
                Stroke::new(crate::tokens::STROKE_THIN, border),
                egui::StrokeKind::Inside,
            );
            if response.has_focus() {
                paint_focus_ring(ui, response.rect, f32::from(rounding.nw), self.theme);
            }

            // Open state lives in egui temp data keyed by `popup_id`; we own
            // the toggle ourselves instead of `egui::Popup::menu`, whose
            // memory bookkeeping (Toggle + keep_popup_open + CloseOnClick*)
            // deadlocks inside modal dialogs (menu stuck closed or painted
            // under the card).
            if response.clicked() {
                ui.ctx().data_mut(|d| {
                    let open = !d.get_temp::<bool>(popup_id).unwrap_or(false);
                    d.insert_temp(popup_id, open);
                });
            }
            if ui.ctx().data_mut(|d| d.get_temp::<bool>(popup_id).unwrap_or(false)) {
                self.show_menu(ui, popup_id, &response);
            }

            response
        })
        .inner
    }

    fn show_menu(self, ui: &mut Ui, popup_id: Id, response: &egui::Response) {
        // Open state is a temp flag owned by `show` — no `egui::Popup`
        // bookkeeping (Toggle/keep_popup_open/CloseOnClick*), which deadlocks
        // inside modal dialogs. A plain `Area` paints on Foreground and
        // `move_to_top` each frame keeps it above the self-raising card.
        let close = |ctx: &egui::Context| {
            ctx.data_mut(|d| d.insert_temp(popup_id, false));
        };
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
            close(ui.ctx());
        }

        let screen = screen_rect(ui);
        let geo = calculate_menu_geometry(screen, response.rect, self.options.len(), self.has_more);

        let area = egui::Area::new(popup_id)
            .order(egui::Order::Foreground)
            .fixed_pos(geo.menu_pos)
            .constrain_to(screen.shrink(4.0))
            .show(ui.ctx(), |ui| {
                crate::components::overlay::floating_surface(
                    self.theme,
                    f32::from(ui.style().visuals.window_corner_radius.nw),
                    super::super::handler::menu_surface_margin(ui.spacing().menu_margin.leftf()),
                )
                .show(ui, |ui| {
                    ui.set_min_width(geo.menu_width);
                    egui::ScrollArea::vertical()
                        .id_salt(popup_id.with("scroll"))
                        .max_height(geo.max_height)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.set_min_width(menu_min_content_width(geo.menu_width));
                            for (idx, opt) in self.options.iter().enumerate() {
                                let clicked = paint_option(
                                    ui,
                                    SelectOption {
                                        label: opt,
                                        selected: idx == *self.selected,
                                        theme: self.theme,
                                        icon: self.option_icons.and_then(|icons| icons.get(idx).copied().flatten()),
                                        disabled: self
                                            .option_disabled
                                            .and_then(|flags| flags.get(idx))
                                            .copied()
                                            .unwrap_or(false),
                                    },
                                )
                                .clicked();
                                if let Some(selected) = selection_from_click(clicked, idx) {
                                    *self.selected = selected;
                                    close(ui.ctx());
                                }
                            }
                            // Comment: This action row leaves selection and open state unchanged.
                            if self.has_more {
                                let load = paint_option(
                                    ui,
                                    SelectOption {
                                        label: "Load more…",
                                        selected: false,
                                        theme: self.theme,
                                        icon: None,
                                        disabled: false,
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

        // Pin the menu as a sublayer of the trigger's layer: sublayers are
        // spliced directly above their parent in `Areas::end_pass`, which
        // `move_to_top` alone can't guarantee (its stable sort preserves the
        // historical [dim, card, popup] order). Skip when the trigger lives
        // on a different z-order (e.g. plain panels) — same-order required.
        let popup_layer = area.response.layer_id;
        ui.ctx().memory_mut(|m| {
            if response.layer_id.order == popup_layer.order {
                m.areas_mut().set_sublayer(response.layer_id, popup_layer);
            }
            m.areas_mut().move_to_top(popup_layer);
        });
        if std::env::var_os("DB_PRO_DEBUG_SELECT").is_some() {
            let probe = egui::pos2(geo.menu_pos.x + 20.0, geo.menu_pos.y + 10.0);
            let top = ui.ctx().layer_id_at(probe);
            eprintln!("[menu] area.rect={:?} layer={popup_layer:?} top_at_pos={top:?}", area.response.rect);
        }

        // Outside click closes — skip clicks inside the menu or on the
        // trigger (the trigger's own click handler toggles).
        let clicked_outside = ui.ctx().input(|i| {
            i.pointer.any_click()
                && i.pointer
                    .interact_pos()
                    .is_some_and(|pos| !area.response.rect.contains(pos) && !response.rect.contains(pos))
        });
        if clicked_outside {
            close(ui.ctx());
        }
    }
}
