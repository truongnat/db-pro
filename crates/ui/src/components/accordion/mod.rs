// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
mod config;
mod handler;
mod ui;

pub use ui::Accordion;

/// Whether an accordion allows only one item open at a time, or multiple items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccordionType {
    Single { collapsible: bool },
    Multiple,
}

pub struct AccordionItem<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub icon: Option<lucide_icons::Icon>,
    pub badge: Option<&'a str>,
    pub disabled: bool,
}

impl<'a> AccordionItem<'a> {
    pub fn new(id: &'a str, title: &'a str) -> Self {
        Self {
            id,
            title,
            icon: None,
            badge: None,
            disabled: false,
        }
    }

    pub fn icon(mut self, icon: lucide_icons::Icon) -> Self {
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
}
