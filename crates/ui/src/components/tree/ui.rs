use crate::components::animation::{hover_t, lerp_color, overlay_t, OVERLAY_DURATION_SECS};
use crate::DbProTheme;
use egui::{Align2, Color32, FontFamily, FontId, Id, Response, Rounding, Sense, Ui, Vec2};

use super::config::{
    CHEVRON_ANIMATION_ID_SALT, CHEVRON_ICON_FONT_SIZE, CHILDREN_HEIGHT_ID_SALT, DETAIL_FONT_SIZE,
    HOVER_ANIMATION_ID_SALT, LABEL_FONT_SIZE, NODE_ICON_FONT_SIZE, ROW_HEIGHT, ROW_ROUNDING, SELECTED_ACCENT_ROUNDING,
    SELECTED_ACCENT_WIDTH,
};
use super::handler::{
    chevron_color, chevron_icon_layers, child_clip_rect, hover_animation_enabled, icon_color, label_color,
    measured_child_height, reveal_clip, row_background, row_height, row_layout, should_toggle_expanded, RowBackground,
    TreeNodeKind,
};

pub struct DatabaseTreeNode<'a> {
    name: &'a str,
    kind: TreeNodeKind,
    detail: Option<&'a str>,
    depth: usize,
    selected: bool,
    expanded: Option<&'a mut bool>,
    theme: DbProTheme,
}

impl<'a> DatabaseTreeNode<'a> {
    pub fn new(name: &'a str, kind: TreeNodeKind, depth: usize, theme: DbProTheme) -> Self {
        Self {
            name,
            kind,
            detail: None,
            depth,
            selected: false,
            expanded: None,
            theme,
        }
    }

    pub fn detail(mut self, detail: &'a str) -> Self {
        self.detail = Some(detail);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn expanded(mut self, expanded: &'a mut bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, mut response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), row_height()), Sense::click());
        response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

        // The UI layer first captures egui input signals, then asks the handler which visual and
        // state outcomes are active. Keeping these decisions centralized prevents row painting and
        // expansion behavior from drifting when the same tree semantics are reused elsewhere.
        let hovered = response.hovered();
        let has_expander = self.expanded.is_some();
        let hover = hover_t(
            ui.ctx(),
            response.id.with(HOVER_ANIMATION_ID_SALT),
            hover_animation_enabled(hovered, self.selected),
        );

        match row_background(self.selected, hover) {
            RowBackground::Selected => paint_selected_background(ui, rect, self.theme),
            RowBackground::Hovered(t) => {
                ui.painter().rect_filled(
                    rect,
                    Rounding::same(ROW_ROUNDING),
                    lerp_color(Color32::TRANSPARENT, self.theme.surface_hover, t),
                );
            }
            RowBackground::None => {}
        }

        let layout = row_layout(rect, self.depth);
        if let Some(expanded) = self.expanded.as_ref() {
            let progress = ui.ctx().animate_bool_with_time(
                response.id.with(CHEVRON_ANIMATION_ID_SALT),
                **expanded,
                OVERLAY_DURATION_SECS,
            );
            paint_chevrons(
                ui,
                layout.chevron_pos,
                progress,
                chevron_color(hovered, self.selected, self.theme),
            );
        }

        ui.painter().text(
            layout.icon_pos,
            Align2::CENTER_CENTER,
            char::from(self.kind.icon()).to_string(),
            FontId::new(NODE_ICON_FONT_SIZE, FontFamily::Name("lucide".into())),
            icon_color(self.kind, self.selected, self.theme),
        );
        ui.painter().text(
            layout.label_pos,
            Align2::LEFT_CENTER,
            self.name,
            FontId::proportional(LABEL_FONT_SIZE),
            label_color(self.selected, self.theme),
        );
        if let Some(detail) = self.detail {
            ui.painter().text(
                layout.detail_pos,
                Align2::RIGHT_CENTER,
                detail,
                FontId::monospace(DETAIL_FONT_SIZE),
                self.theme.text_tertiary,
            );
        }

        if should_toggle_expanded(response.clicked(), has_expander) {
            if let Some(expanded) = self.expanded {
                *expanded = !*expanded;
            }
        }

        response
    }
}

/// Reveals nested tree rows with an animated height clip.
pub fn reveal_children(ui: &mut Ui, id: Id, open: bool, add_contents: impl FnOnce(&mut Ui)) {
    let progress = overlay_t(ui.ctx(), id, open);
    let height_id = id.with(CHILDREN_HEIGHT_ID_SALT);
    let last_height = ui.ctx().data(|data| data.get_temp::<f32>(height_id));
    let Some(reveal) = reveal_clip(last_height, progress) else {
        return;
    };

    // The handler chooses the clip dimensions from stored measurements and animation progress;
    // egui-specific work remains here: allocate the visible strip, run child layout in a clipped
    // child UI, then persist the latest measured content height for the next animation frame.
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, reveal.clip_height), Sense::hover());
    let mut used_height = reveal.child_height;
    ui.allocate_new_ui(
        egui::UiBuilder::new().max_rect(child_clip_rect(rect, width, reveal.child_height)),
        |child_ui| {
            child_ui.set_clip_rect(rect);
            add_contents(child_ui);
            used_height = measured_child_height(child_ui.min_rect().height());
        },
    );
    ui.ctx().data_mut(|data| data.insert_temp(height_id, used_height));
}

fn paint_selected_background(ui: &mut Ui, rect: egui::Rect, theme: DbProTheme) {
    ui.painter()
        .rect_filled(rect, Rounding::same(ROW_ROUNDING), theme.surface_active);
    ui.painter().rect_filled(
        egui::Rect::from_min_size(rect.left_top(), Vec2::new(SELECTED_ACCENT_WIDTH, ROW_HEIGHT)),
        Rounding::same(SELECTED_ACCENT_ROUNDING),
        theme.accent,
    );
}

fn paint_chevrons(ui: &mut Ui, position: egui::Pos2, progress: f32, color: Color32) {
    let font = FontId::new(CHEVRON_ICON_FONT_SIZE, FontFamily::Name("lucide".into()));
    for layer in chevron_icon_layers(progress).into_iter().flatten() {
        let color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), layer.alpha);
        ui.painter().text(
            position,
            Align2::CENTER_CENTER,
            char::from(layer.icon).to_string(),
            font.clone(),
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_icons_layout() {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let _ = ctx.run(Default::default(), |ctx| {
            let font = FontId::new(NODE_ICON_FONT_SIZE, FontFamily::Name("lucide".into()));
            for kind in [
                TreeNodeKind::Server,
                TreeNodeKind::Database,
                TreeNodeKind::Schema,
                TreeNodeKind::Table,
                TreeNodeKind::View,
                TreeNodeKind::Column,
                TreeNodeKind::PrimaryKey,
                TreeNodeKind::ForeignKey,
                TreeNodeKind::Index,
            ] {
                let galley = ctx.fonts(|fonts| {
                    fonts.layout_no_wrap(char::from(kind.icon()).to_string(), font.clone(), Color32::WHITE)
                });
                assert!(!galley.rows.is_empty());
                assert!(galley.size().x > 0.0);
            }
        });
    }
}
