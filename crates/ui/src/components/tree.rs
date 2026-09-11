use crate::DbProTheme;
use egui::{Color32, FontId, Pos2, Rect, Response, Rounding, Sense, Ui, Vec2};
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
            TreeNodeKind::Table => Icon::Table,
            TreeNodeKind::View => Icon::Eye,
            TreeNodeKind::Column => Icon::Columns,
            TreeNodeKind::PrimaryKey => Icon::Key,
            TreeNodeKind::ForeignKey => Icon::Link,
            TreeNodeKind::Index => Icon::ListTree,
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

        // Background
        if self.selected {
            ui.painter()
                .rect_filled(rect, Rounding::same(4.0), self.theme.surface_active);
            let ind_rect = Rect::from_min_size(rect.left_top(), Vec2::new(2.5, row_height));
            ui.painter()
                .rect_filled(ind_rect, Rounding::same(1.0), self.theme.accent);
        } else if is_hovered {
            ui.painter()
                .rect_filled(rect, Rounding::same(4.0), self.theme.surface_hover);
        }

        let indent = self.depth as f32 * 14.0 + 8.0;
        let mut x = rect.left() + indent;

        // Chevron if expandable
        let is_container = self.expanded.is_some();
        if let Some(ref exp) = self.expanded {
            let chev_char = if **exp { "▾" } else { "▸" };
            let chev_galley = ui.painter().layout_no_wrap(
                chev_char.to_owned(),
                FontId::proportional(11.0),
                self.theme.text_secondary,
            );
            let chev_rect = Rect::from_min_size(Pos2::new(x, rect.center().y - 6.0), Vec2::new(12.0, 12.0));
            ui.painter().galley(chev_rect.min, chev_galley, Color32::PLACEHOLDER);
            x += 14.0;
        } else {
            x += if is_container { 14.0 } else { 4.0 };
        }

        // Icon
        let icon_font = FontId::new(13.0, egui::FontFamily::Name("lucide".into()));
        let icon_char = char::from(self.kind.icon()).to_string();
        let icon_galley = ui.painter().layout_no_wrap(
            icon_char,
            icon_font,
            if self.selected {
                self.theme.accent
            } else {
                self.theme.text_secondary
            },
        );
        ui.painter()
            .galley(Pos2::new(x, rect.center().y - 6.5), icon_galley, Color32::PLACEHOLDER);
        x += 18.0;

        // Name
        let name_font = FontId::proportional(12.5);
        let name_galley = ui
            .painter()
            .layout_no_wrap(self.name.to_owned(), name_font, self.theme.text_primary);
        ui.painter()
            .galley(Pos2::new(x, rect.center().y - 6.5), name_galley, Color32::PLACEHOLDER);

        // Detail (e.g. data type or count)
        if let Some(detail_str) = self.detail {
            let detail_font = FontId::monospace(11.0);
            let detail_galley =
                ui.painter()
                    .layout_no_wrap(detail_str.to_owned(), detail_font, self.theme.text_tertiary);
            ui.painter().galley(
                Pos2::new(rect.right() - detail_galley.size().x - 10.0, rect.center().y - 6.0),
                detail_galley,
                Color32::PLACEHOLDER,
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
