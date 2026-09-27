use crate::components::interact::paint_focus_ring;
use crate::DbProTheme;
use egui::{Align2, FontFamily, FontId, Response, RichText, Rounding, Sense, Stroke, Ui, WidgetInfo, WidgetType};
use lucide_icons::Icon;

use super::config::{
    BREADCRUMB_CHEVRON_SIZE, BREADCRUMB_TEXT_SIZE, BREADCRUMB_UNDERLINE_INSET, BREADCRUMB_UNDERLINE_THICKNESS,
    PAGE_BUTTON_ICON_SIZE, PAGE_BUTTON_RADIUS, PAGE_HEADER_DESCRIPTION_SIZE, PAGE_HEADER_GAP, PAGE_HEADER_TITLE_SIZE,
    PAGE_LABEL_FONT_SIZE, SECTION_HEADER_DESCRIPTION_SIZE, SECTION_HEADER_GAP, SECTION_HEADER_TITLE_SIZE,
};
use super::handler::{
    breadcrumb_item_color, can_navigate_next, can_navigate_prev, clamp_page, format_page_label, next_page,
    page_button_accessible_label, page_button_colors, prev_page,
};

/// Controls navigation between numbered pages of items.
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
        *self.page = clamp_page(*self.page, self.page_count);
        ui.horizontal(|ui| {
            let can_prev = can_navigate_prev(self.enabled, *self.page);
            let prev = page_icon_button(ui, Icon::ChevronLeft, can_prev, self.theme);
            if can_prev && prev.clicked() {
                *self.page = prev_page(*self.page);
            }
            ui.label(
                RichText::new(format_page_label(*self.page, self.page_count))
                    .size(PAGE_LABEL_FONT_SIZE)
                    .color(self.theme.text_secondary),
            );
            let can_next = can_navigate_next(self.enabled, *self.page, self.page_count);
            let next = page_icon_button(ui, Icon::ChevronRight, can_next, self.theme);
            if can_next && next.clicked() {
                *self.page = next_page(*self.page, self.page_count);
            }
            prev.union(next)
        })
        .inner
    }
}

/// Helper to render an icon button for pagination steps.
pub fn page_icon_button(ui: &mut Ui, icon: Icon, enabled: bool, theme: DbProTheme) -> Response {
    let mut response = ui.add_enabled(enabled, egui::Button::new("").frame(false));
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, page_button_accessible_label(icon)));
    if !enabled {
        response = response.on_disabled_hover_text("No more pages");
    }

    let rect = response.rect;
    let (fill, color) = page_button_colors(enabled, response.hovered(), &theme);
    ui.painter().rect_filled(rect, Rounding::same(PAGE_BUTTON_RADIUS), fill);
    if response.has_focus() {
        paint_focus_ring(ui, rect, PAGE_BUTTON_RADIUS, theme);
    }
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        char::from(icon).to_string(),
        FontId::new(PAGE_BUTTON_ICON_SIZE, FontFamily::Name("lucide".into())),
        color,
    );
    if enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}

/// Represents an individual item in a breadcrumb trail.
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

/// Trail of links representing the hierarchical location within the application.
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
        ui.horizontal_wrapped(|ui| {
            for (index, item) in self.items.iter().enumerate() {
                if index > 0 {
                    ui.label(
                        RichText::new(char::from(Icon::ChevronRight).to_string())
                            .font(FontId::new(BREADCRUMB_CHEVRON_SIZE, FontFamily::Name("lucide".into())))
                            .color(self.theme.text_tertiary),
                    );
                }
                let color = breadcrumb_item_color(item.current, &self.theme);
                let response = ui.add(
                    egui::Button::new(RichText::new(item.label).size(BREADCRUMB_TEXT_SIZE).color(color))
                        .frame(false)
                        .sense(if item.current { Sense::hover() } else { Sense::click() }),
                );
                if !item.current && response.hovered() {
                    ui.painter().hline(
                        response.rect.x_range(),
                        response.rect.bottom() - BREADCRUMB_UNDERLINE_INSET,
                        Stroke::new(BREADCRUMB_UNDERLINE_THICKNESS, self.theme.border_strong),
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

/// Primary page title header with an optional supporting description.
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
                    .size(PAGE_HEADER_TITLE_SIZE)
                    .strong()
                    .color(self.theme.text_primary),
            );
            if let Some(description) = self.description {
                ui.add_space(PAGE_HEADER_GAP);
                ui.label(
                    RichText::new(description)
                        .size(PAGE_HEADER_DESCRIPTION_SIZE)
                        .color(self.theme.text_secondary),
                );
            }
        })
        .response
    }
}

/// Subsection title header for dividing cards and setting panels.
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
                    .size(SECTION_HEADER_TITLE_SIZE)
                    .strong()
                    .color(self.theme.text_primary),
            );
            if let Some(description) = self.description {
                ui.add_space(SECTION_HEADER_GAP);
                ui.label(
                    RichText::new(description)
                        .size(SECTION_HEADER_DESCRIPTION_SIZE)
                        .color(self.theme.text_muted),
                );
            }
        })
        .response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_ui(mut on_ui: impl FnMut(&mut egui::Ui)) {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| on_ui(ui));
        });
    }

    #[test]
    fn pagination_renders_and_updates_page_state() {
        let theme = DbProTheme::light();
        let mut page = 2;
        run_ui(|ui| {
            let resp = Pagination::new(&mut page, 5, theme).show(ui);
            assert!(resp.rect.is_finite());
            assert!(resp.rect.width() > 0.0);
        });
        assert_eq!(page, 2);
    }

    #[test]
    fn breadcrumbs_render_items() {
        let theme = DbProTheme::light();
        let items = [
            BreadcrumbItem::new("Databases"),
            BreadcrumbItem::new("Production").current(true),
        ];
        run_ui(|ui| {
            let clicked = Breadcrumb::new(&items, theme).show(ui);
            assert_eq!(clicked, None);
        });
    }

    #[test]
    fn page_and_section_headers_render_with_descriptions() {
        let theme = DbProTheme::light();
        run_ui(|ui| {
            let page_h = PageHeader::new("Users", theme)
                .description("Manage system users")
                .show(ui);
            assert!(page_h.rect.is_finite());

            let section_h = SectionHeader::new("Permissions", theme)
                .description("Role assignments")
                .show(ui);
            assert!(section_h.rect.is_finite());
        });
    }
}
