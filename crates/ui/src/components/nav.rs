use crate::DbProTheme;
use egui::{Align2, FontFamily, FontId, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use lucide_icons::Icon;

const PAGE_BUTTON_SIZE: f32 = 28.0;

pub struct Pagination<'a> {
    page: &'a mut usize,
    page_count: usize,
    enabled: bool,
    theme: DbProTheme,
}

impl<'a> Pagination<'a> {
    pub fn new(page: &'a mut usize, page_count: usize, theme: DbProTheme) -> Self {
        Self {
            page,
            page_count: page_count.max(1),
            enabled: true,
            theme,
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        *self.page = (*self.page).clamp(1, self.page_count);
        ui.horizontal(|ui| {
            let can_prev = self.enabled && *self.page > 1;
            let prev = page_icon_button(ui, Icon::ChevronLeft, can_prev, self.theme);
            if can_prev && prev.clicked() {
                *self.page -= 1;
            }
            ui.label(
                RichText::new(format!("{} / {}", *self.page, self.page_count))
                    .size(12.5)
                    .color(self.theme.text_secondary),
            );
            let can_next = self.enabled && *self.page < self.page_count;
            let next = page_icon_button(ui, Icon::ChevronRight, can_next, self.theme);
            if can_next && next.clicked() {
                *self.page += 1;
            }
            prev.union(next)
        })
        .inner
    }
}

fn page_icon_button(ui: &mut Ui, icon: Icon, enabled: bool, theme: DbProTheme) -> Response {
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, mut response) = ui.allocate_exact_size(Vec2::splat(PAGE_BUTTON_SIZE), sense);
    if !enabled {
        response = response.on_disabled_hover_text("No more pages");
    }
    let fill = if enabled && response.hovered() {
        theme.surface_hover
    } else {
        theme.surface_panel
    };
    ui.painter().rect_filled(rect, Rounding::same(6.0), fill);
    if response.has_focus() {
        ui.painter()
            .rect_stroke(rect, Rounding::same(6.0), Stroke::new(1.5, theme.accent));
    }
    let color = if enabled {
        theme.text_primary
    } else {
        theme.text_disabled
    };
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        char::from(icon).to_string(),
        FontId::new(14.0, FontFamily::Name("lucide".into())),
        color,
    );
    if enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}

pub struct BreadcrumbItem<'a> {
    pub label: &'a str,
    pub current: bool,
}

impl<'a> BreadcrumbItem<'a> {
    pub fn new(label: &'a str) -> Self {
        Self { label, current: false }
    }

    pub fn current(mut self, current: bool) -> Self {
        self.current = current;
        self
    }
}

pub struct Breadcrumb<'a> {
    items: &'a [BreadcrumbItem<'a>],
    theme: DbProTheme,
}

impl<'a> Breadcrumb<'a> {
    pub fn new(items: &'a [BreadcrumbItem<'a>], theme: DbProTheme) -> Self {
        Self { items, theme }
    }

    pub fn show(self, ui: &mut Ui) -> Option<usize> {
        let mut clicked = None;
        ui.horizontal(|ui| {
            for (index, item) in self.items.iter().enumerate() {
                if index > 0 {
                    ui.label(
                        RichText::new(char::from(Icon::ChevronRight).to_string())
                            .font(FontId::new(12.0, FontFamily::Name("lucide".into())))
                            .color(self.theme.text_tertiary),
                    );
                }
                let color = if item.current {
                    self.theme.text_primary
                } else {
                    self.theme.text_secondary
                };
                let response = ui.add(
                    egui::Button::new(RichText::new(item.label).size(12.5).color(color))
                        .frame(false)
                        .sense(if item.current { Sense::hover() } else { Sense::click() }),
                );
                if !item.current && response.hovered() {
                    ui.painter().hline(
                        response.rect.x_range(),
                        response.rect.bottom() - 1.0,
                        Stroke::new(1.0, self.theme.border_strong),
                    );
                }
                if response.clicked() {
                    clicked = Some(index);
                }
            }
        });
        clicked
    }
}

pub struct PageHeader<'a> {
    title: &'a str,
    description: Option<&'a str>,
    theme: DbProTheme,
}

impl<'a> PageHeader<'a> {
    pub fn new(title: &'a str, theme: DbProTheme) -> Self {
        Self {
            title,
            description: None,
            theme,
        }
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.label(
                RichText::new(self.title)
                    .size(20.0)
                    .strong()
                    .color(self.theme.text_primary),
            );
            if let Some(description) = self.description {
                ui.add_space(4.0);
                ui.label(RichText::new(description).size(13.0).color(self.theme.text_secondary));
            }
        })
        .response
    }
}

pub struct SectionHeader<'a> {
    title: &'a str,
    description: Option<&'a str>,
    theme: DbProTheme,
}

impl<'a> SectionHeader<'a> {
    pub fn new(title: &'a str, theme: DbProTheme) -> Self {
        Self {
            title,
            description: None,
            theme,
        }
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.label(
                RichText::new(self.title)
                    .size(14.0)
                    .strong()
                    .color(self.theme.text_primary),
            );
            if let Some(description) = self.description {
                ui.add_space(2.0);
                ui.label(RichText::new(description).size(12.0).color(self.theme.text_muted));
            }
        })
        .response
    }
}
