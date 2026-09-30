use super::{config, handler, AccordionItem};
use crate::components::disclosure;
use crate::tokens::{FONT_SIZE_BADGE, FONT_SIZE_UI_LABEL, ICON_SM, ICON_TEXT_GAP, SPACE_MD, SPACE_SM};
use crate::DbProTheme;
use egui::{Align2, FontFamily, FontId, Pos2, Rect, Rounding, Sense, Ui, Vec2, WidgetInfo, WidgetType};
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
        let id = ui.id().with(("accordion_item", item.id));
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), disclosure::HEADER_HEIGHT),
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
        let mut body_state = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            id.with("body"),
            current_is_open,
        );
        body_state.set_open(current_is_open);
        let open_anim_t = body_state.openness(ui.ctx());
        response.widget_info(|| {
            WidgetInfo::selected(
                WidgetType::CollapsingHeader,
                !item.disabled,
                current_is_open,
                item.title,
            )
        });

        let hover_t = disclosure::paint_header_surface(ui, id, &response, self.theme, open_anim_t, item.disabled);
        let (icon_color, title_color) = disclosure::header_colors(disclosure::HeaderColorState {
            theme: &self.theme,
            disabled: item.disabled,
            emphasized: hover_t > 0.01 || current_is_open,
        });

        let center_y = rect.center().y;
        let mut left_x = rect.left() + SPACE_MD;

        // Leading icon rendering
        if let Some(icon) = item.icon {
            ui.painter().text(
                Pos2::new(left_x, center_y),
                Align2::LEFT_CENTER,
                char::from(icon).to_string(),
                FontId::new(ICON_SM, FontFamily::Name("lucide".into())),
                icon_color,
            );
            // Advance past the full icon box before painting the title; using only the
            // text gap makes icon-bearing headers visually overlap their labels.
            left_x += ICON_SM + ICON_TEXT_GAP;
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
        disclosure::paint_chevron(
            ui,
            disclosure::ChevronPaint {
                position: Pos2::new(chevron_x, center_y),
                align: Align2::RIGHT_CENTER,
                color: icon_color,
                open_t: open_anim_t,
            },
        );

        disclosure::show_body(ui, &mut body_state, content)
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
