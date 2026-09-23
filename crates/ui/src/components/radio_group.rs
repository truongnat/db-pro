use crate::components::selection::Radio;
use crate::DbProTheme;
use egui::{Ui, Vec2};

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
    theme: DbProTheme,
}

impl<'a, T: Clone + PartialEq> RadioGroup<'a, T> {
    pub fn new(theme: DbProTheme) -> Self {
        Self {
            options: Vec::new(),
            horizontal: false,
            theme,
        }
    }

    pub fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }

    pub fn option(mut self, opt: RadioGroupOption<'a, T>) -> Self {
        self.options.push(opt);
        self
    }

    pub fn show(self, ui: &mut Ui, selected: &mut T) -> Option<T> {
        let mut changed_to = None;

        let render_items = |ui: &mut Ui| {
            for opt in self.options {
                let is_selected = opt.value == *selected;
                let mut radio = Radio::new(is_selected, opt.label, self.theme).enabled(!opt.disabled);

                if let Some(desc) = opt.description {
                    radio = radio.description(desc);
                }

                let resp = radio.show(ui);
                if resp.clicked() && !is_selected && !opt.disabled {
                    *selected = opt.value.clone();
                    changed_to = Some(opt.value);
                }
            }
        };

        if self.horizontal {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(16.0, 0.0);
                render_items(ui);
            });
        } else {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(0.0, 8.0);
                render_items(ui);
            });
        }

        changed_to
    }
}
