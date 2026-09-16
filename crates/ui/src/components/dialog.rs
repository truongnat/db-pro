use crate::components::animation::{fade_alpha, faded_overlay, overlay_t, small_translate};
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

        // Close on Escape — but only if no other modal is stacked above this one.
        // The global modal stack (DbProApp::modal_stack) owns the authoritative close-on-escape
        // decision; this local handler is a fallback for standalone dialogs outside the stack.
        if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            *self.open = false;
        }

        let screen = screen_rect(ui);
        let mut inner = None;
        let theme = self.theme;
        let width = self.width.min((screen.width() - 32.0).max(80.0));
        // Leave room above and below for the vertical margins plus the title/close row.
        let max_content_height = (screen.height() - 120.0).max(120.0);
        let title = self.title;
        let description = self.description;
        let open = self.open;

        let prev_height = ui.ctx().data(|d| d.get_temp::<f32>(id.with("prev_height")));
        let target_y = if let Some(h) = prev_height {
            (screen.center().y - h * 0.5).clamp(
                screen.top() + 24.0,
                (screen.bottom() - h - 24.0).max(screen.top() + 24.0),
            )
        } else {
            (screen.center().y - 240.0).max(screen.top() + 32.0)
        };

        let target_x = (screen.center().x - width * 0.5).clamp(
            screen.left() + 16.0,
            (screen.right() - width - 16.0).max(screen.left() + 16.0),
        );

        let translate = small_translate(progress, DIALOG_TRANSLATE_PX);
        let origin = Pos2::new(target_x, target_y + translate);
        // Fade the card in/out using the same overlay progress that drives the dim layer,
        // so the card and backdrop share one animation curve (spec §26: 160-220ms).
        let card_alpha = fade_alpha(progress);

        let mut card_rect = None;
        // Capture keyboard focus when the dialog opens so Tab/Shift+Tab stay inside the card.
        // The Area is marked interactable so it participates in egui's focus routing.
        let card_area = Area::new(id.with("card"))
            .order(Order::Tooltip)
            .fixed_pos(origin)
            .interactable(true);
        let card_request_focus = *open && progress > 0.01;

        card_area.show(ui.ctx(), |ui| {
            ui.set_opacity(card_alpha);
            ui.set_width(width);
            // When the dialog first becomes visible, steer keyboard focus to the card so
            // subsequent Tab/Shift+Tab cycles stay inside the dialog body rather than
            // leaking to widgets behind the backdrop.
            if card_request_focus {
                ui.memory_mut(|m| m.request_focus(id.with("card")));
            }
            let res = paint_dialog_card(
                DialogCardPaint {
                    open,
                    title,
                    description,
                    width,
                    max_content_height,
                    theme,
                },
                ui,
                add_contents,
            );
            let rect = ui.min_rect();
            ui.ctx()
                .data_mut(|d| d.insert_temp(id.with("prev_height"), rect.height()));
            card_rect = Some(rect);
            inner = Some(res);
        });

        let dim_resp = Area::new(id.with("dim"))
            .order(Order::Foreground)
            .fixed_pos(screen.min)
            .interactable(true)
            .show(ui.ctx(), |ui| {
                ui.set_min_size(screen.size());
                let resp = ui.allocate_response(screen.size(), egui::Sense::click());
                paint_dim(
                    ui,
                    OverlayPaint {
                        screen,
                        overlay: theme.overlay,
                        progress,
                    },
                );
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

struct DialogCardPaint<'a> {
    open: &'a mut bool,
    title: &'a str,
    description: Option<&'a str>,
    width: f32,
    max_content_height: f32,
    theme: DbProTheme,
}

fn paint_dialog_card<R>(card: DialogCardPaint<'_>, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    let DialogCardPaint {
        open,
        title,
        description,
        width,
        max_content_height,
        theme,
    } = card;
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
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if close_icon_button(ui, theme).clicked() {
                    *open = false;
                }
            });
            ui.vertical(|ui| {
                ui.add(egui::Label::new(RichText::new(title).size(16.0).strong().color(theme.text_primary)).truncate())
                    .on_hover_text(title);
                if let Some(description) = description {
                    ui.add_space(4.0);
                    ui.add(egui::Label::new(RichText::new(description).size(13.0).color(theme.text_secondary)).wrap());
                }
            });
        });
        ui.add_space(16.0);
        // Cap the card height and scroll the body so long errors, verbose validation text
        // or large-font translations grow into a scroll instead of pushing the footer
        // actions (and the close button's row) off-screen.
        egui::ScrollArea::vertical()
            .max_height(max_content_height)
            .auto_shrink([true, true])
            .show(ui, |ui| add_contents(ui))
            .inner
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
        let width = self.width.min((screen.width() - 16.0).max(80.0));
        let title = self.title;
        let open = self.open;
        let x = screen.right() - width + small_translate(progress, SHEET_TRANSLATE_PX);

        Area::new(id)
            .order(Order::Foreground)
            .fixed_pos(screen.min)
            .interactable(true)
            .show(ui.ctx(), |ui| {
                ui.set_min_size(screen.size());
                paint_dim(
                    ui,
                    OverlayPaint {
                        screen,
                        overlay: theme.overlay,
                        progress,
                    },
                );
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
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if close_icon_button(ui, theme).clicked() {
                                    *open = false;
                                }
                            });
                            ui.add(
                                egui::Label::new(RichText::new(title).size(16.0).strong().color(theme.text_primary))
                                    .truncate(),
                            )
                            .on_hover_text(title);
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

struct OverlayPaint {
    screen: Rect,
    overlay: egui::Color32,
    progress: f32,
}

fn paint_dim(ui: &mut Ui, paint: OverlayPaint) {
    ui.painter().rect_filled(
        paint.screen,
        Rounding::ZERO,
        faded_overlay(paint.overlay, paint.progress),
    );
}

fn close_icon_button(ui: &mut Ui, theme: DbProTheme) -> Response {
    Button::new(theme)
        .icon(Icon::X)
        .size(ButtonSize::IconSm)
        .variant(ButtonVariant::Ghost)
        .show(ui)
}

pub struct DialogActionLabels<'a> {
    pub secondary: &'a str,
    pub primary: &'a str,
}

pub fn dialog_actions(ui: &mut Ui, theme: DbProTheme, labels: DialogActionLabels<'_>) -> (bool, bool) {
    let mut secondary_clicked = false;
    let mut primary_clicked = false;
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        primary_clicked = Button::new(theme).text(labels.primary).show(ui).clicked();
        ui.add_space(8.0);
        secondary_clicked = Button::new(theme)
            .text(labels.secondary)
            .variant(ButtonVariant::Ghost)
            .show(ui)
            .clicked();
    });
    (secondary_clicked, primary_clicked)
}
