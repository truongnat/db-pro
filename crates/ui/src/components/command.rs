use crate::components::animation::hover_t;
use crate::components::feedback::kbd_badge;
use crate::DbProTheme;
use egui::{
    Align2, Color32, FontFamily, FontId, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui, UiBuilder, Vec2,
};
use lucide_icons::Icon;

pub struct CommandInput<'a> {
    pub query: &'a mut String,
    pub placeholder: &'a str,
    pub theme: DbProTheme,
}

impl<'a> CommandInput<'a> {
    pub fn new(query: &'a mut String, theme: DbProTheme) -> Self {
        Self {
            query,
            placeholder: "Type a command or search...",
            theme,
        }
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let height = 44.0;
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());

        // Search icon on left
        let icon_x = rect.left() + 14.0;
        let center_y = rect.center().y;
        ui.painter().text(
            Pos2::new(icon_x, center_y),
            Align2::LEFT_CENTER,
            char::from(Icon::Search).to_string(),
            FontId::new(16.0, FontFamily::Name("lucide".into())),
            self.theme.text_secondary,
        );

        // Edit field
        let text_rect = Rect::from_min_max(
            Pos2::new(icon_x + 24.0, rect.top() + 8.0),
            Pos2::new(rect.right() - 32.0, rect.bottom() - 8.0),
        );

        let mut child_ui = ui.new_child(UiBuilder::new().max_rect(text_rect));
        let edit_resp = egui::TextEdit::singleline(self.query)
            .hint_text(self.placeholder)
            .font(DbProTheme::ui_medium_font(13.5))
            .text_color(self.theme.text_primary)
            .frame(false)
            .show(&mut child_ui)
            .response;

        // Bottom separator
        ui.painter().line_segment(
            [
                Pos2::new(rect.left(), rect.bottom() - 0.5),
                Pos2::new(rect.right(), rect.bottom() - 0.5),
            ],
            Stroke::new(1.0, self.theme.border_subtle),
        );

        response.union(edit_resp)
    }
}

pub struct CommandItem<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub subtitle: Option<&'a str>,
    pub icon: Option<Icon>,
    pub shortcut: Option<&'a str>,
    pub disabled: bool,
    pub selected: bool,
}

impl<'a> CommandItem<'a> {
    pub fn new(id: &'a str, title: &'a str) -> Self {
        Self {
            id,
            title,
            subtitle: None,
            icon: None,
            shortcut: None,
            disabled: false,
            selected: false,
        }
    }

    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn shortcut(mut self, shortcut: &'a str) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn show(self, ui: &mut Ui, theme: DbProTheme) -> Response {
        let height = 36.0;
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(width, height),
            if self.disabled { Sense::hover() } else { Sense::click() },
        );

        let hover = hover_t(
            ui.ctx(),
            response.id.with("cmd_hover"),
            (response.hovered() || self.selected) && !self.disabled,
        );

        if self.selected || hover > 0.001 {
            let fill = if self.selected {
                theme.surface_hover
            } else {
                theme.surface_hover.linear_multiply(hover)
            };
            ui.painter().rect_filled(rect, Rounding::same(6.0), fill);
        }

        let mut left_x = rect.left() + 10.0;
        let center_y = rect.center().y;

        if let Some(icon) = self.icon {
            let color = if self.disabled {
                theme.text_disabled
            } else if self.selected || response.hovered() {
                theme.text_primary
            } else {
                theme.text_secondary
            };
            ui.painter().text(
                Pos2::new(left_x, center_y),
                Align2::LEFT_CENTER,
                char::from(icon).to_string(),
                FontId::new(14.0, FontFamily::Name("lucide".into())),
                color,
            );
            left_x += 22.0;
        }

        let text_color = if self.disabled {
            theme.text_disabled
        } else if self.selected || response.hovered() {
            theme.text_primary
        } else {
            theme.text_primary.linear_multiply(0.9)
        };

        ui.painter().text(
            Pos2::new(left_x, center_y),
            Align2::LEFT_CENTER,
            self.title,
            DbProTheme::ui_medium_font(13.0),
            text_color,
        );

        if let Some(sub) = self.subtitle {
            let sub_x = left_x + ui.painter().layout_no_wrap(
                self.title.to_string(),
                DbProTheme::ui_medium_font(13.0),
                Color32::WHITE,
            ).size().x + 8.0;

            ui.painter().text(
                Pos2::new(sub_x, center_y),
                Align2::LEFT_CENTER,
                sub,
                FontId::proportional(11.5),
                theme.text_muted,
            );
        }

        if let Some(sc) = self.shortcut {
            let sc_rect = Rect::from_min_max(
                Pos2::new(rect.right() - 80.0, rect.top() + 6.0),
                Pos2::new(rect.right() - 8.0, rect.bottom() - 6.0),
            );
            let mut sc_ui = ui.new_child(UiBuilder::new().max_rect(sc_rect));
            sc_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                kbd_badge(ui, sc, theme);
            });
        }

        if self.disabled {
            response
        } else {
            response.on_hover_cursor(egui::CursorIcon::PointingHand)
        }
    }
}

pub struct CommandGroup<'a> {
    pub heading: &'a str,
}

impl<'a> CommandGroup<'a> {
    pub fn new(heading: &'a str) -> Self {
        Self { heading }
    }

    pub fn show<R>(self, ui: &mut Ui, theme: DbProTheme, content: impl FnOnce(&mut Ui) -> R) -> R {
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(self.heading)
                .font(DbProTheme::ui_medium_font(11.0))
                .color(theme.text_muted),
        );
        ui.add_space(2.0);
        content(ui)
    }
}

pub struct CommandEmpty<'a> {
    pub text: &'a str,
    pub theme: DbProTheme,
}

impl<'a> CommandEmpty<'a> {
    pub fn new(theme: DbProTheme) -> Self {
        Self {
            text: "No results found.",
            theme,
        }
    }

    pub fn text(mut self, text: &'a str) -> Self {
        self.text = text;
        self
    }

    pub fn show(self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(24.0);
            ui.label(
                egui::RichText::new(self.text)
                    .font(DbProTheme::ui_medium_font(13.0))
                    .color(self.theme.text_muted),
            );
            ui.add_space(24.0);
        });
    }
}
