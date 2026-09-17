use egui::{Area, FontId, Frame, Id, Margin, Order, Pos2, Response, RichText, Rounding, Stroke, Ui};
use lucide_icons::Icon;
use std::borrow::Cow;
use std::hash::Hash;

use crate::components::animation::{fade_alpha, overlay_t, small_translate};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::DbProTheme;

use super::config::{DIALOG_RADIUS, DIALOG_TRANSLATE_PX, DIALOG_WIDTH};
use super::frame::DialogFrame;
use super::layout::{overlay_widget_id, paint_dim, screen_rect_fallback, OverlayPaint};

pub struct Dialog<'a> {
    open: &'a mut bool,
    title: Cow<'a, str>,
    description: Option<Cow<'a, str>>,
    width: f32,
    id_salt: Option<Id>,
    theme: DbProTheme,
}

impl<'a> Dialog<'a> {
    pub fn new(open: &'a mut bool, title: impl Into<Cow<'a, str>>, theme: DbProTheme) -> Self {
        Self {
            open,
            title: title.into(),
            description: None,
            width: DIALOG_WIDTH,
            id_salt: None,
            theme,
        }
    }

    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
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
        self.show_framed(ui, |frame| frame.body(add_contents))
    }

    pub fn show_ctx<R>(self, ctx: &egui::Context, add_contents: impl FnOnce(&mut Ui) -> R) -> Option<R> {
        self.show_framed_ctx(ctx, |frame| frame.body(add_contents))
    }

    pub fn show_framed<R>(self, ui: &mut Ui, add_frame: impl FnOnce(&mut DialogFrame<'_>) -> R) -> Option<R> {
        let id = overlay_widget_id(ui, self.id_salt, "dialog");
        self.show_framed_impl(ui.ctx(), id, add_frame)
    }

    pub fn show_framed_ctx<R>(
        self,
        ctx: &egui::Context,
        add_frame: impl FnOnce(&mut DialogFrame<'_>) -> R,
    ) -> Option<R> {
        let id = self.id_salt.unwrap_or_else(|| Id::new("dbpro_dialog_overlay"));
        self.show_framed_impl(ctx, id, add_frame)
    }

    fn show_framed_impl<R>(
        self,
        ctx: &egui::Context,
        id: Id,
        add_frame: impl FnOnce(&mut DialogFrame<'_>) -> R,
    ) -> Option<R> {
        let progress = overlay_t(ctx, id.with("motion"), *self.open);
        if progress <= 0.0 {
            return None;
        }

        if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            *self.open = false;
        }

        let screen = screen_rect_fallback(ctx);
        let mut inner = None;
        let theme = self.theme;
        let title = self.title;
        let description = self.description;
        let open = self.open;

        let prev_height = ctx.data(|d| d.get_temp::<f32>(id.with("prev_height")));
        let layout =
            crate::components::common_utils::calculate_dialog_layout(screen, self.width, prev_height, 16.0, 24.0);

        let translate = small_translate(progress, DIALOG_TRANSLATE_PX);
        let origin = Pos2::new(layout.target_pos.x, layout.target_pos.y + translate);
        let card_alpha = fade_alpha(progress);

        // 1. Dim backdrop layer (rendered first in Foreground)
        let mut backdrop_clicked = false;
        Area::new(id.with("dim"))
            .order(Order::Foreground)
            .fixed_pos(screen.min)
            .interactable(true)
            .show(ctx, |dim_ui| {
                dim_ui.set_min_size(screen.size());
                paint_dim(
                    dim_ui,
                    OverlayPaint {
                        screen,
                        overlay: theme.overlay,
                        progress,
                    },
                );
                let resp = dim_ui.allocate_response(screen.size(), egui::Sense::click());
                if resp.clicked() {
                    backdrop_clicked = true;
                }
            });

        // 2. Dialog Card layer (rendered second in Foreground on top of dim)
        let mut card_rect = None;
        let card_area = Area::new(id.with("card"))
            .order(Order::Foreground)
            .fixed_pos(origin)
            .interactable(true);

        card_area.show(ctx, |card_ui| {
            card_ui.set_opacity(card_alpha);
            card_ui.set_width(layout.width);
            card_ui.set_max_width(layout.width);
            let res = paint_dialog_card(
                DialogCardPaint {
                    open,
                    title: title.as_ref(),
                    description: description.as_deref(),
                    width: layout.width,
                    max_content_height: layout.max_content_height,
                    theme,
                },
                card_ui,
                add_frame,
            );
            let rect = card_ui.min_rect();
            card_ui
                .ctx()
                .data_mut(|d| d.insert_temp(id.with("prev_height"), rect.height()));
            card_rect = Some(rect);
            inner = Some(res);
        });

        if backdrop_clicked {
            if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
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

fn paint_dialog_card<R>(
    card: DialogCardPaint<'_>,
    ui: &mut Ui,
    add_frame: impl FnOnce(&mut DialogFrame<'_>) -> R,
) -> R {
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
        inner_margin: Margin::symmetric(24.0, 20.0),
        rounding: Rounding::same(DIALOG_RADIUS),
        shadow: theme.floating_shadow(),
        ..Default::default()
    }
    .show(ui, |ui| {
        let inner_w = (width - 48.0).max(80.0);
        ui.set_width(inner_w);
        ui.set_max_width(inner_w);

        // Header: Title & Description on left, Close button on top right
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(title)
                            .font(DbProTheme::ui_medium_font(16.0))
                            .color(theme.text_primary),
                    )
                    .truncate(),
                )
                .on_hover_text(title);
                if let Some(description) = description {
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(description)
                            .font(FontId::proportional(12.5))
                            .color(theme.text_muted),
                    );
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if close_icon_button(ui, theme).clicked() {
                    *open = false;
                }
            });
        });

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);

        let mut frame = DialogFrame {
            ui,
            max_content_height: (max_content_height - 64.0).max(80.0),
            inner_width: inner_w,
        };
        add_frame(&mut frame)
    })
    .inner
}

pub fn close_icon_button(ui: &mut Ui, theme: DbProTheme) -> Response {
    Button::new(theme)
        .icon(Icon::X)
        .size(ButtonSize::Icon)
        .variant(ButtonVariant::Ghost)
        .tooltip("Close (Esc)")
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
