use crate::components::selection::Radio;
use crate::DbProTheme;
use egui::{Response, Ui, WidgetInfo, WidgetType};

use super::handler::{handle_radio_option_click, next_enabled_option_index, radio_group_item_spacing};

pub struct RadioGroupOption<'a, T: Clone + PartialEq> {
    pub value: T,
    pub label: &'a str,
    pub description: Option<&'a str>,
    pub disabled: bool,
}

impl<'a, T: Clone + PartialEq> RadioGroupOption<'a, T> {
    pub fn new(value: T, label: &'a str) -> Self {
        Self {
            value,
            label,
            description: None,
            disabled: false,
        }
    }

    pub fn description(mut self, desc: &'a str) -> Self {
        self.description = Some(desc);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

pub struct RadioGroup<'a, T: Clone + PartialEq> {
    options: Vec<RadioGroupOption<'a, T>>,
    horizontal: bool,
    label: Option<&'a str>,
    theme: DbProTheme,
}

impl<'a, T: Clone + PartialEq> RadioGroup<'a, T> {
    pub fn new(theme: DbProTheme) -> Self {
        Self {
            options: Vec::new(),
            horizontal: false,
            label: None,
            theme,
        }
    }

    pub fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }

    /// Adds an accessible name for the radio group as a whole.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn option(mut self, opt: RadioGroupOption<'a, T>) -> Self {
        self.options.push(opt);
        self
    }

    pub fn show(self, ui: &mut Ui, selected: &mut T) -> Option<T> {
        let Self {
            options,
            horizontal,
            label,
            theme,
        } = self;
        let mut changed_to = None;
        let mut responses: Vec<(Response, T, bool)> = Vec::with_capacity(options.len());

        let group_response = ui
            .scope(|ui| {
                let render_items = |ui: &mut Ui, responses: &mut Vec<(Response, T, bool)>| {
                    for (index, opt) in options.into_iter().enumerate() {
                        let is_selected = opt.value == *selected;
                        let mut radio = Radio::new(is_selected, opt.label, theme).enabled(!opt.disabled);
                        if let Some(description) = opt.description {
                            radio = radio.description(description);
                        }

                        let response = ui.push_id(index, |ui| radio.show(ui)).inner;
                        if let Some(new_value) =
                            handle_radio_option_click(response.clicked(), is_selected, opt.disabled, &opt.value)
                        {
                            *selected = new_value.clone();
                            changed_to = Some(new_value);
                        }
                        responses.push((response, opt.value, opt.disabled));
                    }
                };

                let spacing = radio_group_item_spacing(horizontal);
                if horizontal {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = spacing;
                        render_items(ui, &mut responses);
                    });
                } else {
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing = spacing;
                        render_items(ui, &mut responses);
                    });
                }
            })
            .response;
        if let Some(label) = label {
            group_response.widget_info(|| WidgetInfo::labeled(WidgetType::RadioGroup, true, label));
        }

        // Resolve navigation after all responses exist so focus can move to any enabled sibling.
        if let Some(focused_index) = responses.iter().position(|(response, _, _)| response.has_focus()) {
            let key_action = ui.input_mut(|input| {
                for key in [egui::Key::Space, egui::Key::Enter] {
                    if input.consume_key(egui::Modifiers::NONE, key) {
                        return Some(None);
                    }
                }
                let (backward, forward) = if horizontal {
                    (egui::Key::ArrowLeft, egui::Key::ArrowRight)
                } else {
                    (egui::Key::ArrowUp, egui::Key::ArrowDown)
                };
                if input.consume_key(egui::Modifiers::NONE, backward) {
                    Some(Some(false))
                } else if input.consume_key(egui::Modifiers::NONE, forward) {
                    Some(Some(true))
                } else {
                    None
                }
            });

            match key_action {
                Some(None) => {
                    if let Some((_, value, disabled)) = responses.get(focused_index) {
                        if !disabled && *selected != *value {
                            *selected = value.clone();
                            changed_to = Some(value.clone());
                        }
                    }
                }
                Some(Some(forward)) => {
                    let disabled_options: Vec<bool> = responses.iter().map(|(_, _, disabled)| *disabled).collect();
                    if let Some(next_index) = next_enabled_option_index(&disabled_options, focused_index, forward) {
                        let (next_response, next_value, _) = &responses[next_index];
                        next_response.request_focus();
                        if *selected != *next_value {
                            *selected = next_value.clone();
                            changed_to = Some(next_value.clone());
                        }
                    }
                }
                None => {}
            }
        }

        changed_to
    }
}
