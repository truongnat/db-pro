use crate::DbProTheme;
use egui::{Align, Button, FontFamily, FontId, Frame, Margin, Response, RichText, Rounding, Stroke, TextEdit, Ui};
use lucide_icons::Icon;

pub struct Input<'a> {
    label: Option<&'a str>,
    value: &'a mut String,
    placeholder: &'a str,
    helper_text: Option<&'a str>,
    error_text: Option<&'a str>,
    leading_icon: Option<Icon>,
    clearable: bool,
    width: Option<f32>,
    enabled: bool,
    theme: DbProTheme,
}

pub type ShadcnInput<'a> = Input<'a>;

impl<'a> Input<'a> {
    pub fn new(value: &'a mut String, placeholder: &'a str, theme: DbProTheme) -> Self {
        Self {
            label: None,
            value,
            placeholder,
            helper_text: None,
            error_text: None,
            leading_icon: None,
            clearable: false,
            width: None,
            enabled: true,
            theme,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn helper_text(mut self, text: &'a str) -> Self {
        self.helper_text = Some(text);
        self
    }

    pub fn error_text(mut self, text: &'a str) -> Self {
        self.error_text = Some(text);
        self
    }

    pub fn leading_icon(mut self, icon: Icon) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn clearable(mut self, clearable: bool) -> Self {
        self.clearable = clearable;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let width = self.width.unwrap_or_else(|| ui.available_width());

        ui.vertical(|ui| {
            if let Some(label) = self.label {
                ui.label(
                    RichText::new(label)
                        .size(12.0)
                        .strong()
                        .color(self.theme.text_secondary),
                );
                ui.add_space(3.0);
            }

            let has_error = self.error_text.is_some();
            let border_stroke = if has_error {
                Stroke::new(1.0, self.theme.danger)
            } else {
                Stroke::new(1.0, self.theme.border_default)
            };

            let frame = Frame {
                fill: self.theme.surface_editor,
                stroke: border_stroke,
                inner_margin: Margin::symmetric(8.0, 4.0),
                rounding: Rounding::same(6.0),
                ..Default::default()
            };

            let response = frame
                .show(ui, |ui| {
                    ui.set_min_width(width.max(100.0) - 16.0);
                    ui.horizontal(|ui| {
                        if let Some(icon) = self.leading_icon {
                            ui.label(
                                RichText::new(char::from(icon).to_string())
                                    .font(FontId::new(14.0, FontFamily::Name("lucide".into())))
                                    .color(self.theme.text_muted),
                            );
                            ui.add_space(4.0);
                        }

                        let has_text = !self.value.is_empty();
                        let extra_width = if self.clearable && has_text { 24.0 } else { 0.0 };
                        let edit_response = ui.add_enabled(
                            self.enabled,
                            TextEdit::singleline(self.value)
                                .hint_text(RichText::new(self.placeholder).color(self.theme.text_muted))
                                .desired_width(ui.available_width() - extra_width)
                                .margin(Margin::ZERO)
                                .frame(false)
                                .text_color(self.theme.text_primary),
                        );

                        if self.clearable
                            && has_text
                            && ui
                                .add(
                                    Button::new(
                                        RichText::new(char::from(Icon::X).to_string())
                                            .font(FontId::new(12.0, FontFamily::Name("lucide".into())))
                                            .color(self.theme.text_muted),
                                    )
                                    .frame(false),
                                )
                                .on_hover_text("Clear")
                                .clicked()
                        {
                            self.value.clear();
                        }

                        edit_response
                    })
                    .inner
                })
                .inner;

            if let Some(err) = self.error_text {
                ui.add_space(2.0);
                ui.label(RichText::new(err).size(11.0).color(self.theme.danger));
            } else if let Some(helper) = self.helper_text {
                ui.add_space(2.0);
                ui.label(RichText::new(helper).size(11.0).color(self.theme.text_muted));
            }

            response
        })
        .inner
    }
}

pub struct PasswordInput<'a> {
    label: Option<&'a str>,
    value: &'a mut String,
    placeholder: &'a str,
    show_password: &'a mut bool,
    width: Option<f32>,
    theme: DbProTheme,
}

pub type ShadcnPasswordInput<'a> = PasswordInput<'a>;

impl<'a> PasswordInput<'a> {
    pub fn new(value: &'a mut String, placeholder: &'a str, show_password: &'a mut bool, theme: DbProTheme) -> Self {
        Self {
            label: None,
            value,
            placeholder,
            show_password,
            width: None,
            theme,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let width = self.width.unwrap_or_else(|| ui.available_width());

        ui.vertical(|ui| {
            if let Some(label) = self.label {
                ui.label(
                    RichText::new(label)
                        .size(12.0)
                        .strong()
                        .color(self.theme.text_secondary),
                );
                ui.add_space(3.0);
            }

            Frame {
                fill: self.theme.surface_editor,
                stroke: Stroke::new(1.0, self.theme.border_default),
                inner_margin: Margin::symmetric(8.0, 4.0),
                rounding: Rounding::same(6.0),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.set_min_width(width.max(100.0) - 16.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(char::from(Icon::Lock).to_string())
                            .font(FontId::new(14.0, FontFamily::Name("lucide".into())))
                            .color(self.theme.text_muted),
                    );
                    ui.add_space(4.0);

                    let edit_response = ui.add(
                        TextEdit::singleline(self.value)
                            .password(!*self.show_password)
                            .hint_text(RichText::new(self.placeholder).color(self.theme.text_muted))
                            .desired_width(ui.available_width() - 26.0)
                            .margin(Margin::ZERO)
                            .frame(false)
                            .text_color(self.theme.text_primary),
                    );

                    let eye_icon = if *self.show_password { Icon::EyeOff } else { Icon::Eye };
                    if ui
                        .add(
                            Button::new(
                                RichText::new(char::from(eye_icon).to_string())
                                    .font(FontId::new(13.0, FontFamily::Name("lucide".into())))
                                    .color(self.theme.text_muted),
                            )
                            .frame(false),
                        )
                        .on_hover_text(if *self.show_password { "Hide" } else { "Show" })
                        .clicked()
                    {
                        *self.show_password = !*self.show_password;
                    }

                    edit_response
                })
                .inner
            })
            .inner
        })
        .inner
    }
}

pub struct SearchInput<'a> {
    value: &'a mut String,
    placeholder: &'a str,
    shortcut: Option<&'a str>,
    width: Option<f32>,
    theme: DbProTheme,
}

pub type ShadcnSearchInput<'a> = SearchInput<'a>;

impl<'a> SearchInput<'a> {
    pub fn new(value: &'a mut String, placeholder: &'a str, theme: DbProTheme) -> Self {
        Self {
            value,
            placeholder,
            shortcut: None,
            width: None,
            theme,
        }
    }

    pub fn shortcut(mut self, shortcut: &'a str) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let width = self.width.unwrap_or_else(|| ui.available_width());

        Frame {
            fill: self.theme.surface_editor,
            stroke: Stroke::new(1.0, self.theme.border_default),
            inner_margin: Margin::symmetric(8.0, 5.0),
            rounding: Rounding::same(6.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.set_min_width(width.max(100.0) - 16.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(char::from(Icon::Search).to_string())
                        .font(FontId::new(14.0, FontFamily::Name("lucide".into())))
                        .color(self.theme.text_muted),
                );
                ui.add_space(4.0);

                let extra_width =
                    if self.shortcut.is_some() { 36.0 } else { 0.0 } + if !self.value.is_empty() { 20.0 } else { 0.0 };

                let edit = ui.add(
                    TextEdit::singleline(self.value)
                        .hint_text(RichText::new(self.placeholder).color(self.theme.text_muted))
                        .desired_width(ui.available_width() - extra_width)
                        .margin(Margin::ZERO)
                        .frame(false)
                        .text_color(self.theme.text_primary),
                );

                if !self.value.is_empty()
                    && ui
                        .add(
                            Button::new(
                                RichText::new(char::from(Icon::X).to_string())
                                    .font(FontId::new(12.0, FontFamily::Name("lucide".into())))
                                    .color(self.theme.text_muted),
                            )
                            .frame(false),
                        )
                        .clicked()
                {
                    self.value.clear();
                }

                if let Some(sc) = self.shortcut {
                    Frame {
                        fill: self.theme.surface_panel,
                        stroke: Stroke::new(1.0, self.theme.border_subtle),
                        inner_margin: Margin::symmetric(4.0, 1.0),
                        rounding: Rounding::same(4.0),
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        ui.label(RichText::new(sc).size(10.0).color(self.theme.text_muted));
                    });
                }

                edit
            })
            .inner
        })
        .inner
    }
}

pub struct Textarea<'a> {
    label: Option<&'a str>,
    value: &'a mut String,
    placeholder: &'a str,
    min_rows: usize,
    max_chars: Option<usize>,
    theme: DbProTheme,
}

pub type ShadcnTextarea<'a> = Textarea<'a>;

impl<'a> Textarea<'a> {
    pub fn new(value: &'a mut String, placeholder: &'a str, theme: DbProTheme) -> Self {
        Self {
            label: None,
            value,
            placeholder,
            min_rows: 3,
            max_chars: None,
            theme,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn min_rows(mut self, rows: usize) -> Self {
        self.min_rows = rows;
        self
    }

    pub fn max_chars(mut self, max: usize) -> Self {
        self.max_chars = Some(max);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            if let Some(label) = self.label {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(label)
                            .size(12.0)
                            .strong()
                            .color(self.theme.text_secondary),
                    );
                    if let Some(max) = self.max_chars {
                        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!("{}/{}", self.value.len(), max))
                                    .size(11.0)
                                    .color(self.theme.text_muted),
                            );
                        });
                    }
                });
                ui.add_space(3.0);
            }

            Frame {
                fill: self.theme.surface_editor,
                stroke: Stroke::new(1.0, self.theme.border_default),
                inner_margin: Margin::symmetric(8.0, 6.0),
                rounding: Rounding::same(6.0),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.add(
                    TextEdit::multiline(self.value)
                        .hint_text(RichText::new(self.placeholder).color(self.theme.text_muted))
                        .desired_rows(self.min_rows)
                        .desired_width(ui.available_width())
                        .frame(false)
                        .margin(Margin::ZERO)
                        .text_color(self.theme.text_primary),
                )
            })
            .inner
        })
        .inner
    }
}
