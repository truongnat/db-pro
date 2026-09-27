use super::{config, handler, AccordionItem};
use crate::components::animation::hover_t;
use crate::components::interact::paint_focus_ring;
use crate::tokens::{FONT_SIZE_BADGE, FONT_SIZE_UI_LABEL, ICON_SM, ICON_TEXT_GAP, RADIUS_SM, SPACE_MD, SPACE_SM};
use crate::DbProTheme;
use egui::{Align2, FontFamily, FontId, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2, WidgetInfo, WidgetType};
use lucide_icons::Icon;
use std::collections::BTreeSet;

pub struct Accordion {
    theme: DbProTheme,
}

impl Accordion {
    pub fn new(theme: DbProTheme) -> Self {
        Self { theme }
    }

    /// Renders a single-expansion accordion item.
    ///
    /// Flow:
    /// 1. Determines current open state and queries/updates the expand animation progress.
    /// 2. Allocates header interaction rect (ignoring clicks if item is disabled).
    /// 3. On header click, delegates state transition to `handler::toggle_single`.
    /// 4. Paints header surface hover effect, bottom divider, leading icon, title, badge, and trailing chevron.
    /// 5. If expanded animation progress > threshold, renders disclosure content in a padded frame with opacity fade.
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
        let open_anim_t =
            ui.ctx()
                .animate_bool_with_time(id.with("open_anim"), is_open, config::OPEN_ANIMATION_SECONDS);
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), config::HEADER_HEIGHT),
            if item.disabled { Sense::hover() } else { Sense::click() },
        );

        let keyboard_toggle = if item.disabled || !response.has_focus() {
            false
        } else {
            ui.input_mut(|input| {
                [egui::Key::Space, egui::Key::Enter]
                    .into_iter()
                    .any(|key| input.consume_key(egui::Modifiers::NONE, key))
            })
        };
        let should_toggle = handler::should_activate_header(response.clicked() || keyboard_toggle, item.disabled);
        if should_toggle {
            handler::toggle_single(selected_id, item.id, collapsible);
        }
        let current_is_open = selected_id.as_deref() == Some(item.id);
        response.widget_info(|| {
            WidgetInfo::selected(
                WidgetType::CollapsingHeader,
                !item.disabled,
                current_is_open,
                item.title,
            )
        });

        let hover = hover_t(ui.ctx(), id.with("hover"), response.hovered() && !item.disabled);
        if hover > 0.001 {
            ui.painter().rect_filled(
                rect,
                Rounding::same(6.0),
                self.theme.surface_hover.linear_multiply(hover),
            );
        }
        if response.has_focus() && !item.disabled {
            paint_focus_ring(ui, rect, RADIUS_SM, self.theme);
        }
        ui.painter().line_segment(
            [
                Pos2::new(rect.left(), rect.bottom() - config::HEADER_DIVIDER_INSET_Y),
                Pos2::new(rect.right(), rect.bottom() - config::HEADER_DIVIDER_INSET_Y),
            ],
            Stroke::new(config::HEADER_DIVIDER_WIDTH, self.theme.border_subtle),
        );

        let center_y = rect.center().y;
        let mut left_x = rect.left() + SPACE_MD;

        // Leading icon rendering
        if let Some(icon) = item.icon {
            let color = if item.disabled {
                self.theme.text_disabled
            } else if current_is_open {
                self.theme.accent
            } else {
                self.theme.text_secondary
            };
            ui.painter().text(
                Pos2::new(left_x, center_y),
                Align2::LEFT_CENTER,
                char::from(icon).to_string(),
                FontId::new(ICON_SM, FontFamily::Name("lucide".into())),
                color,
            );
            left_x += ICON_TEXT_GAP;
        }

        let chevron_x = rect.right() - config::CHEVRON_RIGHT_INSET;
        let badge_layout = item.badge.map(|badge| {
            let galley = ui.painter().layout_no_wrap(
                badge.to_owned(),
                FontId::proportional(FONT_SIZE_BADGE),
                self.theme.text_muted,
            );
            let width = (galley.size().x + SPACE_MD).min((chevron_x - rect.left()).max(0.0));
            let badge_rect = Rect::from_center_size(
                Pos2::new(chevron_x - ICON_TEXT_GAP - width * 0.5, center_y),
                Vec2::new(width, config::BADGE_HEIGHT),
            );
            (badge_rect, galley)
        });
        let title_right = badge_layout
            .as_ref()
            .map_or(chevron_x - ICON_TEXT_GAP, |(badge_rect, _)| {
                badge_rect.left() - SPACE_SM
            });

        // Title text rendering
        let title_color = if item.disabled {
            self.theme.text_disabled
        } else if current_is_open || response.hovered() {
            self.theme.text_primary
        } else {
            self.theme.text_secondary
        };
        let title_rect = Rect::from_min_max(
            Pos2::new(left_x, rect.top()),
            Pos2::new(title_right.max(left_x), rect.bottom()),
        );
        ui.painter().with_clip_rect(title_rect).text(
            Pos2::new(left_x, center_y),
            Align2::LEFT_CENTER,
            item.title,
            DbProTheme::ui_medium_font(FONT_SIZE_UI_LABEL),
            title_color,
        );

        // Optional badge pill rendering
        if let Some((badge_rect, galley)) = badge_layout {
            ui.painter().rect_filled(
                badge_rect,
                Rounding::same(config::BADGE_CORNER_RADIUS),
                self.theme.surface_hover,
            );
            ui.painter().with_clip_rect(badge_rect).galley(
                Pos2::new(
                    badge_rect.left() + config::BADGE_TEXT_OFFSET_X,
                    badge_rect.top() + config::BADGE_TEXT_OFFSET_Y,
                ),
                galley,
                self.theme.text_muted,
            );
        }

        // Trailing chevron indicator (down when opening, right when closed)
        let chevron_color = if item.disabled {
            self.theme.text_disabled
        } else {
            self.theme.text_secondary
        };
        let chevron = if open_anim_t > config::CHEVRON_OPEN_THRESHOLD {
            Icon::ChevronDown
        } else {
            Icon::ChevronRight
        };
        ui.painter().text(
            Pos2::new(chevron_x, center_y),
            Align2::RIGHT_CENTER,
            char::from(chevron).to_string(),
            FontId::new(ICON_SM, FontFamily::Name("lucide".into())),
            chevron_color,
        );

        // Content area rendering with smooth opacity transition
        let mut result = None;
        if open_anim_t > config::OPEN_CONTENT_THRESHOLD {
            let margin = egui::Margin {
                left: 12.0,
                right: 12.0,
                top: SPACE_SM,
                bottom: SPACE_MD,
            };
            egui::Frame::none().inner_margin(margin).show(ui, |ui| {
                ui.set_opacity(open_anim_t);
                result = Some(content(ui));
            });
        }
        result
    }

    /// Renders a multi-expansion accordion item.
    ///
    /// Preserves independent item open sets by capturing the item's prior open status,
    /// rendering through `show_single`, and synchronizing any transition via `handler::sync_multi_open_set`.
    pub fn show_multi<R>(
        &self,
        ui: &mut Ui,
        item: AccordionItem<'_>,
        open_set: &mut BTreeSet<String>,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> Option<R> {
        let was_open = open_set.contains(item.id);
        let item_id = item.id;
        let disabled = item.disabled;
        let mut selected = was_open.then(|| item_id.to_owned());
        let result = self.show_single(ui, item, &mut selected, true, content);
        handler::sync_multi_open_set(open_set, item_id, was_open, selected.as_deref(), disabled);
        result
    }
}
