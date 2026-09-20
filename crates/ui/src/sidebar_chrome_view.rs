//! Native sidebar chrome: connection launcher and primary query action.
use super::*;
use egui::{vec2, Align2, Pos2, Rect, Rounding, Sense, Stroke};
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SidebarChromeAction {
    OpenCommandPalette,
    NewConnection,
    NewQuery,
}

pub(super) struct SidebarChromeContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) active_name: &'a str,
    pub(super) command_palette_shortcut: &'a str,
    pub(super) new_connection_shortcut: &'a str,
    pub(super) new_query_shortcuts: &'a [String],
}

impl SidebarChromeContext<'_> {
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> Vec<SidebarChromeAction> {
        let mut actions = self.draw_connection_launcher(ui);
        ui.add_space(SPACE_XS);
        actions.extend(self.draw_new_query(ui));
        actions
    }

    fn draw_connection_launcher(&self, ui: &mut egui::Ui) -> Vec<SidebarChromeAction> {
        const PLUS_SLOT_W: f32 = 28.0;
        const SELECTOR_H: f32 = 24.0;
        const SEARCH_SLOT_W: f32 = 18.0;

        let mut actions = Vec::new();
        ui.add_space(SPACE_SM);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(SPACE_XS, 0.0);
            let selector_w = (ui.available_width() - PLUS_SLOT_W - ui.spacing().item_spacing.x).max(72.0);
            let (selector_rect, selector_response) =
                ui.allocate_exact_size(vec2(selector_w, SELECTOR_H), Sense::click());
            if selector_response.hovered() {
                ui.painter()
                    .rect_filled(selector_rect, Rounding::same(RADIUS_SM), self.theme.surface_hover);
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            self.paint_selector_label(ui, selector_rect, SEARCH_SLOT_W);
            if selector_response.clicked() {
                actions.push(SidebarChromeAction::OpenCommandPalette);
            }
            selector_response.on_hover_text(format!(
                "{}\nCommand Palette ({})",
                self.active_name, self.command_palette_shortcut
            ));

            if Button::new(self.theme)
                .icon(Icon::Plus)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .tooltip(format!("New Connection ({})", self.new_connection_shortcut))
                .show(ui)
                .clicked()
            {
                actions.push(SidebarChromeAction::NewConnection);
            }
        });
        actions
    }

    fn paint_selector_label(&self, ui: &mut egui::Ui, selector_rect: Rect, search_slot_width: f32) {
        let search_galley = ui.painter().layout_no_wrap(
            char::from(Icon::Search).to_string(),
            font_icon(ICON_XS),
            self.theme.text_secondary,
        );
        let name_max = (selector_rect.width() - SPACE_XS * 2.0 - search_slot_width - SPACE_XS).max(24.0);
        let name_galley = ui.painter().layout_job({
            let mut job = egui::text::LayoutJob::single_section(
                self.active_name.to_owned(),
                egui::TextFormat {
                    font_id: font_ui_label(),
                    color: self.theme.text_primary,
                    ..Default::default()
                },
            );
            job.wrap = egui::text::TextWrapping {
                max_width: name_max,
                max_rows: 1,
                break_anywhere: true,
                overflow_character: Some('…'),
            };
            job
        });
        ui.painter().galley(
            Pos2::new(
                selector_rect.left() + SPACE_XS,
                selector_rect.center().y - name_galley.size().y * 0.5,
            ),
            name_galley,
            self.theme.text_primary,
        );
        ui.painter().galley(
            Pos2::new(
                selector_rect.right() - SPACE_XS - search_galley.size().x,
                selector_rect.center().y - search_galley.size().y * 0.5,
            ),
            search_galley,
            self.theme.text_secondary,
        );
    }

    fn draw_new_query(&self, ui: &mut egui::Ui) -> Vec<SidebarChromeAction> {
        const NEW_QUERY_HEIGHT: f32 = BUTTON_HEIGHT_SM;
        let mut actions = Vec::new();
        let button_rect = Rect::from_min_size(ui.cursor().min, vec2(ui.available_width(), NEW_QUERY_HEIGHT));
        let response = ui.allocate_rect(button_rect, Sense::click());
        self.paint_new_query_button(ui, button_rect, response.hovered());
        if response.clicked() {
            actions.push(SidebarChromeAction::NewQuery);
        }
        response.on_hover_cursor(egui::CursorIcon::PointingHand);
        actions
    }

    fn paint_new_query_button(&self, ui: &mut egui::Ui, button_rect: Rect, hovered: bool) {
        ui.painter().rect_filled(
            button_rect,
            Rounding::same(RADIUS_SM),
            if hovered {
                self.theme.surface_hover
            } else {
                self.theme.surface_panel
            },
        );
        ui.painter().rect_stroke(
            button_rect,
            Rounding::same(RADIUS_SM),
            Stroke::new(
                STROKE_THIN,
                if hovered {
                    self.theme.border_default
                } else {
                    self.theme.border_subtle
                },
            ),
        );
        let left_center = Pos2::new(button_rect.left() + SPACE_MD, button_rect.center().y);
        ui.painter().text(
            left_center,
            Align2::LEFT_CENTER,
            char::from(Icon::SquarePen).to_string(),
            font_icon(ICON_SM),
            self.theme.text_primary,
        );
        ui.painter().text(
            Pos2::new(left_center.x + SPACE_LG + 2.0, left_center.y),
            Align2::LEFT_CENTER,
            "New query",
            font_caption(),
            self.theme.text_primary,
        );
        self.paint_shortcut_chips(ui, button_rect.right() - SPACE_MD, left_center.y);
    }

    fn paint_shortcut_chips(&self, ui: &egui::Ui, right_x: f32, center_y: f32) {
        let parts = self.new_query_shortcuts;
        let theme = self.theme;
        if parts.is_empty() {
            return;
        }
        let painter = ui.painter();
        let font = egui::FontId::monospace(10.5);
        let chip_pad_x = 5.0;
        let chip_pad_y = 2.0;
        let plus_gap = 2.0;
        let galleys: Vec<_> = parts
            .iter()
            .map(|part| painter.layout_no_wrap(part.clone(), font.clone(), theme.text_muted))
            .collect();
        let plus_galley = painter.layout_no_wrap("+".to_owned(), font, theme.text_muted);
        let mut total_w = galleys
            .iter()
            .map(|galley| chip_pad_x * 2.0 + galley.size().x)
            .sum::<f32>();
        if galleys.len() > 1 {
            total_w += (galleys.len() - 1) as f32 * (plus_gap * 2.0 + plus_galley.size().x);
        }
        let mut x = right_x - total_w;
        for (index, galley) in galleys.iter().enumerate() {
            if index > 0 {
                x += plus_gap;
                painter.galley(
                    Pos2::new(x, center_y - plus_galley.size().y * 0.5),
                    plus_galley.clone(),
                    theme.text_muted,
                );
                x += plus_galley.size().x + plus_gap;
            }
            let chip_w = galley.size().x + chip_pad_x * 2.0;
            let chip_h = galley.size().y + chip_pad_y * 2.0;
            let chip = Rect::from_min_size(Pos2::new(x, center_y - chip_h * 0.5), vec2(chip_w, chip_h));
            painter.rect_filled(chip, Rounding::same(4.0), theme.surface_elevated);
            painter.rect_stroke(chip, Rounding::same(4.0), Stroke::new(1.0, theme.border_subtle));
            painter.galley(
                Pos2::new(chip.left() + chip_pad_x, center_y - galley.size().y * 0.5),
                std::sync::Arc::clone(galley),
                theme.text_muted,
            );
            x += chip_w;
        }
    }
}
