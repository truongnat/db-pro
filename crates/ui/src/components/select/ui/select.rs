// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use std::borrow::Cow;

use egui::{Frame, Id, Margin, Response, RichText, Sense, Stroke, TextureId, Ui};
use lucide_icons::Icon;

use super::{
    super::{
        config::MIN_TRIGGER_WIDTH,
        handler::{
            calculate_menu_geometry, menu_key_action, menu_max_width, menu_min_content_width, menu_row_width,
            navigate_selection, selected_label, selection_from_click, should_request_load_more,
            trigger_accessibility_label, trigger_content_width, trigger_inner_margin, trigger_preferred_width,
            trigger_text_width, widest_label_width, MenuKeyAction, NavigationKeys,
        },
    },
    option::{paint_option, SelectOption},
};
use crate::{
    components::{
        animation::{hover_t, lerp_color},
        interact::{combo_box_info, paint_focus_ring},
        overlay::screen_rect,
    },
    tokens::{FONT_SIZE_UI_LABEL, ICON_TEXT_GAP},
    DbProTheme,
};

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

/// Token-driven trigger box, resolved once per frame from `SelectSize`/`SelectVariant`.
struct TriggerMetrics {
    height: f32,
    font_size: f32,
    icon_size: f32,
    margin: Margin,
    stroke_extra: f32,
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
    /// `None` resolves from the egui visuals at show time — the light default
    /// used to leak a light dropdown into dark dialogs when a caller forgot it.
    theme: Option<DbProTheme>,
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
            theme: None,
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
        self.theme = Some(theme);
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
        let width = match self.width {
            Some(width) => width,
            // A labeled Select belongs to a form column and follows the field
            // rhythm (full width). An unlabeled one hugs its widest option: the
            // old `available_width()` default stretched a one-word choice across
            // the whole row, and the popup — which follows the trigger — with it.
            None => {
                if self.label.is_some() {
                    ui.available_width()
                } else {
                    self.preferred_trigger_width(ui)
                }
            }
        };
        // A label forces a vertical label+trigger stack, but only then. An unlabeled
        // Select must be allocated directly in the parent layout: wrapping it in a
        // top_down child re-anchored it to the top of horizontal rows and to the left
        // of right-to-left toolbars (UI-04).
        if let Some(label) = self.label.clone() {
            let theme = self.theme_for(ui);
            return ui
                .vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.set_width(width);
                    ui.set_max_width(width);
                    // Same label component and gap as `FormField`/`Input`, so a
                    // labeled Select sits in the form rhythm instead of inventing
                    // its own caption typography.
                    crate::components::Label::new(label, theme).show(ui);
                    ui.add_space(crate::tokens::LABEL_HELPER_GAP);
                    self.show_body(ui, width)
                })
                .inner;
        }
        self.show_body(ui, width)
    }

    fn theme_for(&self, ui: &Ui) -> DbProTheme {
        self.theme.unwrap_or_else(|| DbProTheme::from_visuals(ui.visuals()))
    }

    /// Outer box of the trigger, shared by the paint path and the width
    /// measurement so a measured label can never disagree with what is drawn.
    fn trigger_metrics(&self, ui: &Ui) -> TriggerMetrics {
        let compact = crate::components::button::SizeTokens::from_size(crate::components::ButtonSize::Sm);
        let (height, font_size, icon_size, margin) = match self.size {
            SelectSize::Default => (
                crate::tokens::component::input::INPUT_HEIGHT_DEFAULT,
                FONT_SIZE_UI_LABEL,
                crate::tokens::ICON_SM,
                trigger_inner_margin(ui.spacing().button_padding.x, ui.spacing().button_padding.y),
            ),
            SelectSize::Sm => (
                compact.min_height,
                compact.font_size,
                compact.icon_size,
                Margin::symmetric(compact.padding.x as i8, compact.padding.y as i8),
            ),
        };
        // egui 0.36 counts `2 * stroke.width` in the Frame's total size; subtract it so the
        // trigger keeps its token-driven outer box (height and width). Borderless (ghost)
        // variants need no offset.
        let stroke_extra = if self.variant == SelectVariant::Ghost {
            0.0
        } else {
            2.0 * crate::tokens::STROKE_THIN
        };
        TriggerMetrics {
            height,
            font_size,
            icon_size,
            margin,
            stroke_extra,
        }
    }

    fn preferred_trigger_width(&self, ui: &Ui) -> f32 {
        let metrics = self.trigger_metrics(ui);
        let mut labels: Vec<&str> = self.options.iter().map(String::as_str).collect();
        if let Some(text) = &self.selected_text {
            labels.push(text);
        }
        // A label wider than the space left after the trigger chrome can only ever be
        // clamped, so stop measuring there instead of scanning the whole option list.
        let chrome = trigger_preferred_width(0.0, metrics.margin, metrics.stroke_extra);
        let cap = (ui.available_width() - chrome).max(0.0);
        let widest = widest_label_width(ui, &labels, &egui::FontId::proportional(metrics.font_size), cap);
        trigger_preferred_width(widest, metrics.margin, metrics.stroke_extra)
            .max(MIN_TRIGGER_WIDTH)
            .min(ui.available_width())
    }

    /// Widest option row, so the popup never ellipsizes a choice it just offered.
    /// The check indicator and icon column are reserved unconditionally: any row
    /// can be the selected one, and the menu must not resize while navigating.
    fn menu_content_width(&self, ui: &Ui) -> f32 {
        let labels: Vec<&str> = self.options.iter().map(String::as_str).collect();
        let has_icon = self
            .option_icons
            .is_some_and(|icons| icons.iter().any(|icon| icon.is_some()));
        let extras = menu_row_width(0.0, has_icon, true, ui.spacing().button_padding.x);
        let cap = (menu_max_width(screen_rect(ui)) - extras).max(0.0);
        let widest = widest_label_width(ui, &labels, &egui::FontId::proportional(FONT_SIZE_UI_LABEL), cap);
        menu_row_width(widest, has_icon, true, ui.spacing().button_padding.x)
    }

    fn show_body(self, ui: &mut Ui, width: f32) -> Response {
        let theme = self.theme_for(ui);
        let enabled = ui.is_enabled();
        let TriggerMetrics {
            height,
            font_size,
            icon_size,
            margin,
            stroke_extra,
        } = self.trigger_metrics(ui);
        let content_height = height - stroke_extra - margin.topf() - margin.bottomf();

        let current_text = match &self.selected_text {
            Some(text) => text.clone().into_owned(),
            None => selected_label(self.options, *self.selected).to_owned(),
        };

        // Stable identity preserves menu open state across frames.
        let popup_id = self.id_salt;
        let draft_id = popup_id.with("draft");
        let was_open = ui.ctx().data_mut(|d| d.get_temp::<bool>(popup_id).unwrap_or(false));

        let ghost = self.variant == SelectVariant::Ghost;
        let rounding = egui::CornerRadius::same(crate::tokens::RADIUS_INPUT as u8);
        let background = ghost.then(|| ui.painter().add(egui::Shape::Noop));
        let trigger_btn = Frame {
            fill: if !enabled {
                theme.surface_2
            } else if ghost {
                egui::Color32::TRANSPARENT
            } else {
                theme.surface_editor
            },
            stroke: if !enabled {
                Stroke::new(crate::tokens::STROKE_THIN, theme.border_subtle)
            } else if ghost {
                Stroke::NONE
            } else {
                Stroke::new(crate::tokens::STROKE_THIN, theme.border_default)
            },
            inner_margin: margin,
            corner_radius: rounding,
            ..Default::default()
        }
        .show(ui, |ui| {
            let content_width = trigger_content_width(
                width - margin.leftf() - margin.rightf() - stroke_extra,
                ui.available_width(),
            );
            ui.set_width(content_width);
            ui.spacing_mut().interact_size.y = content_height;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.set_min_height(content_height);
                let icon = if was_open { Icon::ChevronUp } else { Icon::ChevronDown };
                let text_width = trigger_text_width(ui.available_width());
                let text_response = ui.allocate_ui_with_layout(
                    egui::vec2(text_width, content_height),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |text_ui| {
                        text_ui.set_min_width(text_width);
                        text_ui.add(
                            egui::Label::new(RichText::new(&current_text).size(font_size).color(if enabled {
                                theme.text_primary
                            } else {
                                theme.text_disabled
                            }))
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
                        .color(if enabled { theme.text_muted } else { theme.text_disabled }),
                );
            });
        })
        .response;

        let response = trigger_btn.interact(if enabled { Sense::click() } else { Sense::hover() });
        let info_label = trigger_accessibility_label(self.label.as_deref(), &current_text);
        response.widget_info(|| combo_box_info(enabled, &info_label));
        // Open state lives in egui temp data keyed by `popup_id`; we own the
        // toggle ourselves instead of `egui::Popup::menu`, whose bookkeeping
        // (Toggle + keep_popup_open + CloseOnClick*) deadlocks inside modal
        // dialogs (menu stuck closed or painted under the card). A pointer
        // click and an activation key are the same toggle; the explicit
        // keyboard path only covers the frame where egui did not mark the
        // response clicked, and never re-toggles an open menu (Enter there
        // commits the draft selection in `show_menu`).
        let keyboard_toggle = enabled
            && response.has_focus()
            && !was_open
            && ui.input(|input| input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space));
        let toggled = enabled && (response.clicked() || keyboard_toggle);
        let mut just_opened = false;
        if toggled {
            let open = !was_open;
            just_opened = open;
            let seeded = *self.selected;
            ui.ctx().data_mut(|d| {
                d.insert_temp(popup_id, open);
                if open {
                    d.insert_temp(draft_id, seeded);
                }
            });
        }
        let is_open = if toggled { !was_open } else { was_open };
        let hover = hover_t(
            ui.ctx(),
            response.id.with("hover"),
            enabled && (response.hovered() || response.has_focus() || is_open),
        );
        let border = if !enabled {
            theme.border_subtle
        } else if is_open || response.has_focus() {
            theme.accent
        } else if ghost {
            egui::Color32::TRANSPARENT
        } else {
            lerp_color(theme.border_default, theme.border_strong, hover)
        };
        if let Some(background) = background {
            ui.painter().set(
                background,
                egui::Shape::rect_filled(
                    response.rect,
                    rounding,
                    lerp_color(egui::Color32::TRANSPARENT, theme.surface_hover, hover),
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
            paint_focus_ring(ui, response.rect, f32::from(rounding.nw), theme);
        }

        if is_open {
            self.show_menu(ui, popup_id, &response, theme, just_opened);
        }

        response
    }

    fn show_menu(self, ui: &mut Ui, popup_id: Id, response: &egui::Response, theme: DbProTheme, just_opened: bool) {
        // Open state is a temp flag owned by `show` — no `egui::Popup`
        // bookkeeping (Toggle/keep_popup_open/CloseOnClick*), which deadlocks
        // inside modal dialogs. A plain `Area` paints on Foreground and
        // `move_to_top` each frame keeps it above the self-raising card.
        let close = |ctx: &egui::Context| {
            ctx.data_mut(|d| d.insert_temp(popup_id, false));
        };
        let draft_id = popup_id.with("draft");
        let mut draft = ui
            .ctx()
            .data_mut(|d| d.get_temp::<usize>(draft_id))
            .unwrap_or(*self.selected);
        let key_actions = ui.input(|input| {
            (
                input.key_pressed(egui::Key::Escape),
                input.key_pressed(egui::Key::Enter),
                input.key_pressed(egui::Key::ArrowDown),
                input.key_pressed(egui::Key::ArrowUp),
            )
        });
        // Arrows move a draft highlight; the committed selection changes only
        // on Enter/click, so Escape leaves the previous value untouched.
        let narrowed = navigate_selection(
            draft,
            self.options.len(),
            NavigationKeys {
                arrow_down: key_actions.2,
                arrow_up: key_actions.3,
            },
        );
        if narrowed != draft {
            draft = narrowed;
            ui.ctx().data_mut(|d| d.insert_temp(draft_id, draft));
        }
        if !just_opened {
            match menu_key_action(key_actions.0, key_actions.1) {
                MenuKeyAction::Commit => {
                    *self.selected = draft;
                    close(ui.ctx());
                    return;
                }
                MenuKeyAction::Cancel => {
                    close(ui.ctx());
                    return;
                }
                MenuKeyAction::None => {}
            }
        }

        let screen = screen_rect(ui);
        let content_width = self.menu_content_width(ui);
        let geo = calculate_menu_geometry(screen, response.rect, self.options.len(), self.has_more, content_width);

        let area = egui::Area::new(popup_id)
            .order(egui::Order::Foreground)
            .fixed_pos(geo.menu_pos)
            .constrain_to(screen.shrink(4.0))
            .show(ui.ctx(), |ui| {
                crate::components::overlay::floating_surface(
                    theme,
                    f32::from(ui.style().visuals.window_corner_radius.nw),
                    super::super::handler::menu_surface_margin(ui.spacing().menu_margin.leftf()),
                )
                .show(ui, |ui| {
                    // Pin the width, not just a minimum: rows allocate
                    // `available_width()`, so an unpinned popup reports itself as
                    // wide as everything to the right of the trigger (a 598px
                    // panel for a 90px option list) and no `auto_shrink` can undo
                    // a content size that big.
                    ui.set_width(geo.menu_width);
                    egui::ScrollArea::vertical()
                        .id_salt(popup_id.with("scroll"))
                        .max_height(geo.max_height)
                        .auto_shrink([true, true])
                        .show(ui, |ui| {
                            ui.set_width(menu_min_content_width(geo.menu_width));
                            for (idx, opt) in self.options.iter().enumerate() {
                                let clicked = paint_option(
                                    ui,
                                    SelectOption {
                                        label: opt,
                                        selected: idx == draft,
                                        theme,
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
                                        theme,
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
            tracing::debug!(
                target: "db_pro::ui::select",
                menu_rect = ?area.response.rect,
                ?popup_layer,
                ?top,
                "select menu layer debug"
            );
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
