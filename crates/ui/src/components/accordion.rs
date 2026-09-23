use crate::components::animation::hover_t;
use crate::DbProTheme;
use egui::{Align2, Color32, FontFamily, FontId, Pos2, Rounding, Sense, Stroke, Ui, Vec2};
use lucide_icons::Icon;
use std::collections::BTreeSet;

/// Whether an accordion allows only one item open at a time, or multiple items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccordionType {
    Single { collapsible: bool },
    Multiple,
}

pub struct AccordionItem<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub icon: Option<Icon>,
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

    pub fn icon(mut self, icon: Icon) -> Self {
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

pub struct Accordion {
    theme: DbProTheme,
}

impl Accordion {
    pub fn new(theme: DbProTheme) -> Self {
        Self { theme }
    }

    /// Renders a single-expansion accordion item.
    pub fn show_single<R>(
        &self,
        ui: &mut Ui,
        item: AccordionItem<'_>,
        selected_id: &mut Option<String>,
        collapsible: bool,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> Option<R> {
        let is_open = selected_id.as_deref() == Some(item.id);
        let id = ui.id().with(("accordion_item", item.id));

        let open_anim_t = ui.ctx().animate_bool_with_time(id.with("open_anim"), is_open, 0.2);

        let height = 40.0;
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(width, height),
            if item.disabled { Sense::hover() } else { Sense::click() },
        );

        if response.clicked() && !item.disabled {
            if is_open {
                if collapsible {
                    *selected_id = None;
                }
            } else {
                *selected_id = Some(item.id.to_string());
            }
        }

        let hover = hover_t(ui.ctx(), id.with("hover"), response.hovered() && !item.disabled);

        // Header Background
        if hover > 0.001 {
            let fill = self.theme.surface_hover.linear_multiply(hover);
            ui.painter().rect_filled(rect, Rounding::same(6.0), fill);
        }

        // Header Border bottom separator
        ui.painter().line_segment(
            [
                Pos2::new(rect.left(), rect.bottom() - 0.5),
                Pos2::new(rect.right(), rect.bottom() - 0.5),
            ],
            Stroke::new(1.0, self.theme.border_subtle),
        );

        let mut left_x = rect.left() + 12.0;
        let center_y = rect.center().y;

        // Leading icon if present
        if let Some(icon) = item.icon {
            let icon_color = if item.disabled {
                self.theme.text_disabled
            } else if is_open {
                self.theme.accent
            } else {
                self.theme.text_secondary
            };
            ui.painter().text(
                Pos2::new(left_x, center_y),
                Align2::LEFT_CENTER,
                char::from(icon).to_string(),
                FontId::new(14.0, FontFamily::Name("lucide".into())),
                icon_color,
            );
            left_x += 22.0;
        }

        // Title
        let title_color = if item.disabled {
            self.theme.text_disabled
        } else if is_open || response.hovered() {
            self.theme.text_primary
        } else {
            self.theme.text_secondary
        };

        ui.painter().text(
            Pos2::new(left_x, center_y),
            Align2::LEFT_CENTER,
            item.title,
            DbProTheme::ui_medium_font(13.0),
            title_color,
        );

        // Badge if present
        if let Some(badge) = item.badge {
            let badge_galley =
                ui.painter()
                    .layout_no_wrap(badge.to_string(), FontId::proportional(11.0), self.theme.text_muted);
            let badge_w = badge_galley.size().x + 12.0;
            let badge_rect = egui::Rect::from_center_size(
                Pos2::new(rect.right() - 36.0 - badge_w * 0.5, center_y),
                Vec2::new(badge_w, 18.0),
            );
            ui.painter()
                .rect_filled(badge_rect, Rounding::same(9.0), self.theme.surface_hover);
            ui.painter().galley(
                Pos2::new(badge_rect.left() + 6.0, badge_rect.top() + 2.5),
                badge_galley,
                Color32::PLACEHOLDER,
            );
        }

        // Chevron icon on trailing right
        let chevron_color = if item.disabled {
            self.theme.text_disabled
        } else {
            self.theme.text_secondary
        };
        let chevron_char = if open_anim_t > 0.5 {
            char::from(Icon::ChevronDown)
        } else {
            char::from(Icon::ChevronRight)
        };
        ui.painter().text(
            Pos2::new(rect.right() - 14.0, center_y),
            Align2::RIGHT_CENTER,
            chevron_char.to_string(),
            FontId::new(14.0, FontFamily::Name("lucide".into())),
            chevron_color,
        );

        // Animated disclosure content
        let mut content_result = None;
        if open_anim_t > 0.01 {
            let content_margin = egui::Margin {
                left: 12.0,
                right: 12.0,
                top: 8.0,
                bottom: 12.0,
            };

            egui::Frame::none().inner_margin(content_margin).show(ui, |ui| {
                ui.set_opacity(open_anim_t);
                content_result = Some(content(ui));
            });
        }

        content_result
    }

    /// Renders a multi-expansion accordion item.
    pub fn show_multi<R>(
        &self,
        ui: &mut Ui,
        item: AccordionItem<'_>,
        open_set: &mut BTreeSet<String>,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> Option<R> {
        let is_open = open_set.contains(item.id);
        let mut selected = if is_open { Some(item.id.to_string()) } else { None };
        let res = self.show_single(ui, item, &mut selected, true, content);
        if let Some(sel) = selected {
            open_set.insert(sel);
        }
        res
    }
}
