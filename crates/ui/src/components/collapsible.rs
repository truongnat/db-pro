use crate::components::animation::hover_t;
use crate::DbProTheme;
use egui::{Align2, FontFamily, FontId, Pos2, Response, Rounding, Sense, Ui, Vec2};
use lucide_icons::Icon;

pub struct Collapsible<'a> {
    open: &'a mut bool,
    title: Option<&'a str>,
    icon: Option<Icon>,
    badge: Option<&'a str>,
    disabled: bool,
    theme: DbProTheme,
}

impl<'a> Collapsible<'a> {
    pub fn new(open: &'a mut bool, theme: DbProTheme) -> Self {
        Self {
            open,
            title: None,
            icon: None,
            badge: None,
            disabled: false,
            theme,
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn badge(mut self, badge: &'a str) -> Self {
        self.badge = Some(badge);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Shows collapsible with standard trigger header and custom content.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> (Response, Option<R>) {
        let id = ui.id().with("collapsible");
        let open_anim_t = ui.ctx().animate_bool_with_time(id.with("open_anim"), *self.open, 0.18);

        let height = 36.0;
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(width, height),
            if self.disabled { Sense::hover() } else { Sense::click() },
        );

        if response.clicked() && !self.disabled {
            *self.open = !*self.open;
        }

        let hover = hover_t(ui.ctx(), id.with("hover"), response.hovered() && !self.disabled);
        if hover > 0.001 {
            let fill = self.theme.surface_hover.linear_multiply(hover * 0.8);
            ui.painter().rect_filled(rect, Rounding::same(6.0), fill);
        }

        let mut left_x = rect.left() + 6.0;
        let center_y = rect.center().y;

        // Chevron on left
        let chevron_color = if self.disabled {
            self.theme.text_disabled
        } else {
            self.theme.text_secondary
        };
        let chevron_char = if open_anim_t > 0.5 {
            char::from(Icon::ChevronDown)
        } else {
            char::from(Icon::ChevronRight)
        };
        ui.painter().text(
            Pos2::new(left_x, center_y),
            Align2::LEFT_CENTER,
            chevron_char.to_string(),
            FontId::new(14.0, FontFamily::Name("lucide".into())),
            chevron_color,
        );
        left_x += 18.0;

        // Optional icon
        if let Some(icon) = self.icon {
            let icon_color = if self.disabled {
                self.theme.text_disabled
            } else {
                self.theme.text_secondary
            };
            ui.painter().text(
                Pos2::new(left_x, center_y),
                Align2::LEFT_CENTER,
                char::from(icon).to_string(),
                FontId::new(14.0, FontFamily::Name("lucide".into())),
                icon_color,
            );
            left_x += 20.0;
        }

        // Title
        if let Some(title) = self.title {
            let title_color = if self.disabled {
                self.theme.text_disabled
            } else if response.hovered() {
                self.theme.text_primary
            } else {
                self.theme.text_secondary
            };
            ui.painter().text(
                Pos2::new(left_x, center_y),
                Align2::LEFT_CENTER,
                title,
                DbProTheme::ui_medium_font(12.5),
                title_color,
            );
        }

        // Optional badge
        if let Some(badge) = self.badge {
            let badge_galley =
                ui.painter()
                    .layout_no_wrap(badge.to_string(), FontId::proportional(11.0), self.theme.text_muted);
            let badge_w = badge_galley.size().x + 10.0;
            let badge_rect = egui::Rect::from_center_size(
                Pos2::new(rect.right() - 14.0 - badge_w * 0.5, center_y),
                Vec2::new(badge_w, 16.0),
            );
            ui.painter()
                .rect_filled(badge_rect, Rounding::same(8.0), self.theme.surface_hover);
            ui.painter().galley(
                Pos2::new(badge_rect.left() + 5.0, badge_rect.top() + 2.0),
                badge_galley,
                egui::Color32::PLACEHOLDER,
            );
        }

        let mut content_res = None;
        if open_anim_t > 0.01 {
            let content_margin = egui::Margin {
                left: 20.0,
                right: 6.0,
                top: 4.0,
                bottom: 8.0,
            };
            egui::Frame::none().inner_margin(content_margin).show(ui, |ui| {
                ui.set_opacity(open_anim_t);
                content_res = Some(content(ui));
            });
        }

        (response, content_res)
    }
}
