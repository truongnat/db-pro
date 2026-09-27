use crate::components::animation::{hover_t, lerp_color};
use crate::DbProTheme;
use egui::{Align2, Color32, FontFamily, FontId, Rect, Response, Rounding, Sense, Ui, Vec2};
use lucide_icons::Icon;

use super::config::{DETAIL_INSET, ROW_HEIGHT};
use super::geometry::{after_chevron, after_icon, chevron_position, icon_position, node_origin, row_height};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeNodeKind {
    Server,
    Database,
    Schema,
    Table,
    View,
    Column,
    PrimaryKey,
    ForeignKey,
    Index,
}

impl TreeNodeKind {
    pub fn icon(&self) -> Icon {
        match self {
            Self::Server => Icon::Server,
            Self::Database => Icon::Database,
            Self::Schema => Icon::Folder,
            Self::Table => Icon::Table2,
            Self::View => Icon::Eye,
            Self::Column => Icon::Columns3,
            Self::PrimaryKey => Icon::Key,
            Self::ForeignKey => Icon::Link,
            Self::Index => Icon::Zap,
        }
    }
}

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
        let hovered = response.hovered();
        let hover = hover_t(ui.ctx(), response.id.with("hover"), hovered && !self.selected);
        if self.selected {
            ui.painter()
                .rect_filled(rect, Rounding::same(4.0), self.theme.surface_active);
            ui.painter().rect_filled(
                Rect::from_min_size(rect.left_top(), Vec2::new(2.5, ROW_HEIGHT)),
                Rounding::same(1.0),
                self.theme.accent,
            );
        } else if hover > 0.001 {
            ui.painter().rect_filled(
                rect,
                Rounding::same(4.0),
                lerp_color(Color32::TRANSPARENT, self.theme.surface_hover, hover),
            );
        }
        let (mut x, center_y) = node_origin(rect, self.depth);
        if let Some(expanded) = self.expanded.as_ref() {
            let progress = ui.ctx().animate_bool_with_time(
                response.id.with("chev_anim"),
                **expanded,
                crate::components::animation::OVERLAY_DURATION_SECS,
            );
            let color = if hovered || self.selected {
                self.theme.text_secondary
            } else {
                self.theme.text_muted
            };
            paint_chevrons(ui, chevron_position(x, center_y), progress, color);
            x = after_chevron(x);
        } else {
            x = after_chevron(x);
        }
        let icon_color = match self.kind {
            TreeNodeKind::PrimaryKey => self.theme.warning,
            TreeNodeKind::ForeignKey => self.theme.info,
            _ if self.selected => self.theme.accent,
            _ => self.theme.text_secondary,
        };
        ui.painter().text(
            icon_position(x, center_y),
            Align2::CENTER_CENTER,
            char::from(self.kind.icon()).to_string(),
            FontId::new(13.0, FontFamily::Name("lucide".into())),
            icon_color,
        );
        x = after_icon(x);
        ui.painter().text(
            egui::Pos2::new(x, center_y),
            Align2::LEFT_CENTER,
            self.name,
            FontId::proportional(12.5),
            if self.selected {
                self.theme.accent
            } else {
                self.theme.text_primary
            },
        );
        if let Some(detail) = self.detail {
            ui.painter().text(
                egui::Pos2::new(rect.right() - DETAIL_INSET, center_y),
                Align2::RIGHT_CENTER,
                detail,
                FontId::monospace(11.0),
                self.theme.text_tertiary,
            );
        }
        if response.clicked() {
            if let Some(expanded) = self.expanded {
                *expanded = !*expanded;
            }
        }
        response
    }
}

fn paint_chevrons(ui: &mut Ui, position: egui::Pos2, progress: f32, color: Color32) {
    let font = |alpha: u8| Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
    if progress < 0.99 {
        ui.painter().text(
            position,
            Align2::CENTER_CENTER,
            char::from(Icon::ChevronRight).to_string(),
            FontId::new(10.5, FontFamily::Name("lucide".into())),
            font(((1.0 - progress) * 255.0) as u8),
        );
    }
    if progress > 0.01 {
        ui.painter().text(
            position,
            Align2::CENTER_CENTER,
            char::from(Icon::ChevronDown).to_string(),
            FontId::new(10.5, FontFamily::Name("lucide".into())),
            font((progress * 255.0) as u8),
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
            let font = FontId::new(13.0, FontFamily::Name("lucide".into()));
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
