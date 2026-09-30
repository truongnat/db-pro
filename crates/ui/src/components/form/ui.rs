use super::config::LABEL_FONT_SIZE;
use super::handler::compose_access_label;
use crate::components::input::Input;
use crate::tokens::LABEL_HELPER_GAP;
use crate::DbProTheme;
use egui::{Id, Response, RichText, Ui};
use std::borrow::Cow;
use std::hash::Hash;

pub struct Label<'a> {
    text: Cow<'a, str>,
    required: bool,
    enabled: bool,
    theme: DbProTheme,
}

impl<'a> Label<'a> {
    pub fn new(text: impl Into<Cow<'a, str>>, theme: DbProTheme) -> Self {
        Self {
            text: text.into(),
            required: false,
            enabled: true,
            theme,
        }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let color = if self.enabled {
            self.theme.text_secondary
        } else {
            self.theme.text_disabled
        };
        ui.horizontal(|ui| {
            let response = ui.label(
                RichText::new(self.text.as_ref())
                    .size(LABEL_FONT_SIZE)
                    .strong()
                    .color(color),
            );
            if self.required {
                ui.add_space(LABEL_HELPER_GAP);
                ui.label(
                    RichText::new("*")
                        .size(LABEL_FONT_SIZE)
                        .strong()
                        .color(self.theme.danger),
                );
            }
            response
        })
        .inner
    }
}

pub struct FormField<'a> {
    label: Cow<'a, str>,
    value: &'a mut String,
    placeholder: Cow<'a, str>,
    helper_text: Option<Cow<'a, str>>,
    error_text: Option<Cow<'a, str>>,
    required: bool,
    enabled: bool,
    id_salt: Option<Id>,
    theme: DbProTheme,
}

impl<'a> FormField<'a> {
    pub fn new(
        label: impl Into<Cow<'a, str>>,
        value: &'a mut String,
        placeholder: impl Into<Cow<'a, str>>,
        theme: DbProTheme,
    ) -> Self {
        Self {
            label: label.into(),
            value,
            placeholder: placeholder.into(),
            helper_text: None,
            error_text: None,
            required: false,
            enabled: true,
            id_salt: None,
            theme,
        }
    }
    /// Sets a stable unique ID salt for repeated labels; labels remain the default.
    pub fn id_salt(mut self, salt: impl Hash) -> Self {
        self.id_salt = Some(Id::new(salt));
        self
    }
    pub fn helper_text(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.helper_text = Some(text.into());
        self
    }
    pub fn error_text(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.error_text = Some(text.into());
        self
    }
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        // The input is the focus target, so attach required/error context to its semantic name; sibling painted labels cannot carry it.
        let access_label = compose_access_label(
            self.label.as_ref(),
            self.required,
            self.helper_text.as_deref(),
            self.error_text.as_deref(),
        );
        ui.vertical(|ui| {
            Label::new(self.label.as_ref(), self.theme)
                .required(self.required)
                .enabled(self.enabled)
                .show(ui);
            ui.add_space(LABEL_HELPER_GAP);
            let mut field = Input::new(self.value, self.placeholder.as_ref(), self.theme)
                .id_salt(self.id_salt.unwrap_or_else(|| Id::new(self.label.as_ref())))
                .access_label(access_label)
                .enabled(self.enabled);
            if let Some(error) = self.error_text.as_deref() {
                field = field.error_text(error);
            } else if let Some(helper) = self.helper_text.as_deref() {
                field = field.helper_text(helper);
            }
            field.show(ui)
        })
        .inner
    }
}
