use crate::components::animation::{hover_t, lerp_color, overlay_t};
use crate::DbProTheme;
use egui::{Align2, Color32, FontFamily, FontId, Id, Pos2, Rect, Response, Rounding, Sense, Ui, Vec2};
use lucide_icons::Icon;

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
            TreeNodeKind::Server => Icon::Server,
            TreeNodeKind::Database => Icon::Database,
            TreeNodeKind::Schema => Icon::Folder,
            TreeNodeKind::Table => Icon::Table2,
            TreeNodeKind::View => Icon::Eye,
            TreeNodeKind::Column => Icon::Columns3,
            TreeNodeKind::PrimaryKey => Icon::Key,
            TreeNodeKind::ForeignKey => Icon::Link,
            TreeNodeKind::Index => Icon::Zap,
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

    pub fn selected(mut self, sel: bool) -> Self {
        self.selected = sel;
        self
    }

    pub fn expanded(mut self, exp: &'a mut bool) -> Self {
        self.expanded = Some(exp);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let row_height = 26.0;
        let (rect, mut resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), row_height), Sense::click());
        resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);

        let is_hovered = resp.hovered();
        let hover = hover_t(ui.ctx(), resp.id.with("hover"), is_hovered && !self.selected);
        if self.selected {
            ui.painter()
                .rect_filled(rect, Rounding::same(4.0), self.theme.surface_active);
            let ind_rect = Rect::from_min_size(rect.left_top(), Vec2::new(2.5, row_height));
            ui.painter()
                .rect_filled(ind_rect, Rounding::same(1.0), self.theme.accent);
        } else if hover > 0.001 {
            ui.painter().rect_filled(
                rect,
                Rounding::same(4.0),
                lerp_color(Color32::TRANSPARENT, self.theme.surface_hover, hover),
            );
        }

        let indent = self.depth as f32 * 14.0 + 8.0;
        let mut x = rect.left() + indent;
        let center_y = rect.center().y;

        // Chevron slot (14px)
        let chevron_slot = 14.0;
        if let Some(ref exp) = self.expanded {
            let expand_t = ui.ctx().animate_bool_with_time(
                resp.id.with("chev_anim"),
                **exp,
                crate::components::animation::OVERLAY_DURATION_SECS,
            );
            let chevron_color = if is_hovered || self.selected {
                self.theme.text_secondary
            } else {
                self.theme.text_muted
            };

            // Cross-fade chevron icons
            if expand_t < 0.99 {
                let alpha = ((1.0 - expand_t) * 255.0) as u8;
                let c = Color32::from_rgba_unmultiplied(chevron_color.r(), chevron_color.g(), chevron_color.b(), alpha);
                ui.painter().text(
                    Pos2::new(x + 6.0, center_y),
                    Align2::CENTER_CENTER,
                    char::from(Icon::ChevronRight).to_string(),
                    FontId::new(10.5, FontFamily::Name("lucide".into())),
                    c,
                );
            }
            if expand_t > 0.01 {
                let alpha = (expand_t * 255.0) as u8;
                let c = Color32::from_rgba_unmultiplied(chevron_color.r(), chevron_color.g(), chevron_color.b(), alpha);
                ui.painter().text(
                    Pos2::new(x + 6.0, center_y),
                    Align2::CENTER_CENTER,
                    char::from(Icon::ChevronDown).to_string(),
                    FontId::new(10.5, FontFamily::Name("lucide".into())),
                    c,
                );
            }
            x += chevron_slot;
        } else {
            x += chevron_slot;
        }

        // Node Icon (Lucide vector icon)
        let icon_color = match self.kind {
            TreeNodeKind::PrimaryKey => self.theme.warning,
            TreeNodeKind::ForeignKey => self.theme.info,
            _ if self.selected => self.theme.accent,
            _ => self.theme.text_secondary,
        };
        ui.painter().text(
            Pos2::new(x + 7.0, center_y),
            Align2::CENTER_CENTER,
            char::from(self.kind.icon()).to_string(),
            FontId::new(13.0, FontFamily::Name("lucide".into())),
            icon_color,
        );
        x += 17.0;

        // Name
        let name_color = if self.selected {
            self.theme.accent
        } else {
            self.theme.text_primary
        };
        ui.painter().text(
            Pos2::new(x, center_y),
            Align2::LEFT_CENTER,
            self.name,
            FontId::proportional(12.5),
            name_color,
        );

        // Detail (e.g. data type or count) on the right
        if let Some(detail_str) = self.detail {
            ui.painter().text(
                Pos2::new(rect.right() - 10.0, center_y),
                Align2::RIGHT_CENTER,
                detail_str,
                FontId::monospace(11.0),
                self.theme.text_tertiary,
            );
        }

        if resp.clicked() {
            if let Some(exp) = self.expanded {
                *exp = !*exp;
            }
        }

        resp
    }
}

/// Reveals nested tree rows with a height clip so expand/collapse is not a hard snap.
pub fn reveal_children(ui: &mut Ui, id: Id, open: bool, add_contents: impl FnOnce(&mut Ui)) {
    let t = overlay_t(ui.ctx(), id, open);
    if t <= 0.01 {
        return;
    }
    let height_id = id.with("content_h");
    let last_h = ui.ctx().data(|d| d.get_temp::<f32>(height_id)).unwrap_or(48.0);
    let clip_h = (last_h * t).max(1.0);
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, clip_h), Sense::hover());
    let mut used_h = last_h;
    ui.allocate_new_ui(
        egui::UiBuilder::new().max_rect(Rect::from_min_size(rect.min, Vec2::new(width, last_h.max(clip_h)))),
        |ui| {
            ui.set_clip_rect(rect);
            add_contents(ui);
            used_h = ui.min_rect().height().max(1.0);
        },
    );
    ui.ctx().data_mut(|d| d.insert_temp(height_id, used_h));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_icons() {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);

        let _ = ctx.run(Default::default(), |ctx| {
            let icon_font = FontId::new(13.0, egui::FontFamily::Name("lucide".into()));
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
                let ch = char::from(kind.icon());
                let galley = ctx.fonts(|f| f.layout_no_wrap(ch.to_string(), icon_font.clone(), Color32::WHITE));
                assert!(!galley.rows.is_empty(), "Icon for {:?} failed to layout", kind);
                assert!(galley.size().x > 0.0);
            }

            for chev in [Icon::ChevronDown, Icon::ChevronRight] {
                let ch = char::from(chev);
                let galley = ctx.fonts(|f| f.layout_no_wrap(ch.to_string(), icon_font.clone(), Color32::WHITE));
                assert!(!galley.rows.is_empty(), "Chevron {:?} failed to layout", chev);
                assert!(galley.size().x > 0.0);
            }
        });
    }
}
