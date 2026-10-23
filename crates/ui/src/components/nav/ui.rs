// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use egui::{Align2, CornerRadius, FontFamily, FontId, Response, RichText, Sense, Stroke, Ui, WidgetInfo, WidgetType};
use lucide_icons::Icon;

use super::{
    config::{
        BREADCRUMB_CHEVRON_SIZE, BREADCRUMB_TEXT_SIZE, BREADCRUMB_UNDERLINE_INSET, BREADCRUMB_UNDERLINE_THICKNESS,
        PAGE_BUTTON_ICON_SIZE, PAGE_BUTTON_RADIUS, PAGE_HEADER_DESCRIPTION_SIZE, PAGE_HEADER_GAP,
        PAGE_HEADER_TITLE_SIZE, PAGE_LABEL_FONT_SIZE, SECTION_HEADER_DESCRIPTION_SIZE, SECTION_HEADER_GAP,
        SECTION_HEADER_TITLE_SIZE, SIDEBAR_ITEM_PILL_INSET_X, SIDEBAR_ITEM_PILL_INSET_Y, SIDEBAR_ITEM_PILL_RADIUS,
        SIDEBAR_ITEM_PILL_WIDTH,
    },
    handler::{
        breadcrumb_item_color, can_navigate_next, can_navigate_prev, clamp_page, format_page_label, next_page,
        page_button_accessible_label, page_button_colors, prev_page,
    },
};
use crate::{components::interact::paint_focus_ring, tokens::FONT_SIZE_BADGE, DbProTheme};

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
    ui.painter()
        .rect_filled(rect, CornerRadius::same(PAGE_BUTTON_RADIUS as u8), fill);
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

/// Muted uppercase label above a field group (settings panels, sidebars).
pub fn section_label(ui: &mut Ui, text: impl Into<String>, theme: DbProTheme) -> Response {
    ui.label(
        RichText::new(text.into())
            .font(FontId::new(FONT_SIZE_BADGE, FontFamily::Name("ui_medium".into())))
            .strong()
            .color(theme.text_muted),
    )
}

/// Paints the accent pill that marks the active row in a navigation rail.
///
/// A rail's own fill is the chrome panel, which in light theme equals the
/// `Secondary` rest fill, so the pill — not the fill — is what makes selection
/// visible. Shared so every rail (sidebar, gallery, settings) marks it the same way.
pub fn paint_selection_pill(ui: &Ui, row: egui::Rect, theme: DbProTheme) {
    let pill_rect = egui::Rect::from_min_max(
        egui::pos2(
            row.left() + SIDEBAR_ITEM_PILL_INSET_X,
            row.top() + SIDEBAR_ITEM_PILL_INSET_Y,
        ),
        egui::pos2(
            row.left() + SIDEBAR_ITEM_PILL_INSET_X + SIDEBAR_ITEM_PILL_WIDTH,
            row.bottom() - SIDEBAR_ITEM_PILL_INSET_Y,
        ),
    );
    ui.painter().rect_filled(
        pill_rect,
        CornerRadius::same(SIDEBAR_ITEM_PILL_RADIUS as u8),
        theme.accent,
    );
}

/// Full-width sidebar row: ghost button when idle, secondary + accent pill when
/// the item is the active view.
// cc-scan:allow TOO_MANY_PARAMS — context params passed through
pub fn sidebar_item(ui: &mut Ui, icon: Icon, label: &str, active: bool, theme: DbProTheme) -> Response {
    let button = crate::components::button::Button::new(theme)
        .icon(icon)
        .text(label)
        .variant(if active {
            crate::components::button::ButtonVariant::Secondary
        } else {
            crate::components::button::ButtonVariant::Ghost
        })
        .size(crate::components::button::ButtonSize::Sm)
        .full_width(true)
        .left_aligned();
    let response = button.show(ui);
    if active {
        paint_selection_pill(ui, response.rect, theme);
    }
    response
}

#[cfg(test)]
#[path = "tests/ui_tests.rs"]
mod tests;
