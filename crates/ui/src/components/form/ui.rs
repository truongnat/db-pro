// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
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
        // Labels are noninteractive: use their font height rather than a button-sized row.
        let height = ui.fonts_mut(|fonts| fonts.row_height(&egui::FontId::proportional(LABEL_FONT_SIZE)));
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), height),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.x = LABEL_HELPER_GAP;
                let response = ui.label(
                    RichText::new(self.text.as_ref())
                        .size(LABEL_FONT_SIZE)
                        .strong()
                        .color(color),
                );
                if self.required {
                    ui.label(
                        RichText::new("*")
                            .size(LABEL_FONT_SIZE)
                            .strong()
                            .color(self.theme.danger),
                    );
                }
                response
            },
        )
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
    pub fn id_salt(mut self, salt: impl Hash + std::fmt::Debug) -> Self {
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
        // cc-scan:allow LINE_TOO_LONG — literal must not wrap
        // The input is the focus target, so attach required/error context to its semantic name; sibling painted labels cannot carry it.
        let access_label = compose_access_label(
            self.label.as_ref(),
            self.required,
            self.helper_text.as_deref(),
            self.error_text.as_deref(),
        );
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_label_marker_and_helper_gaps_have_one_spacing_owner() {
        for spacing in [egui::vec2(0.0, 0.0), egui::vec2(8.0, 4.0), egui::vec2(16.0, 16.0)] {
            let ctx = egui::Context::default();
            let theme = DbProTheme::light();
            DbProTheme::install_fonts(&ctx);
            theme.apply(&ctx);
            let mut value = String::from("database");
            let output = crate::test_frame::frame(&ctx, egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.set_width(300.0);
                    ui.spacing_mut().item_spacing = spacing;
                    FormField::new("Host", &mut value, "", theme)
                        .required(true)
                        .helper_text("Helper")
                        .show(ui);
                    assert_eq!(ui.spacing().item_spacing, spacing);
                });
            });
            let text = |name: &str| {
                output
                    .shapes
                    .iter()
                    .find_map(|shape| match &shape.shape {
                        egui::Shape::Text(text) if text.galley.text() == name => Some(text),
                        _ => None,
                    })
                    .expect("painted text")
            };
            let label = text("Host");
            let star = text("*");
            let helper = text("Helper");
            let frame = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Rect(rect) if rect.fill == theme.surface_editor => Some(rect.rect),
                    _ => None,
                })
                .expect("field frame");
            assert_eq!(star.pos.x - label.pos.x - label.galley.size().x, LABEL_HELPER_GAP);
            assert_eq!(frame.top() - label.pos.y - label.galley.size().y, LABEL_HELPER_GAP);
            assert_eq!(helper.pos.y - frame.bottom(), LABEL_HELPER_GAP);
        }
    }
}
