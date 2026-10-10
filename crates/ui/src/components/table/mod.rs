mod config;
mod handler;
mod ui;

#[cfg(test)]
mod tests;

use crate::DbProTheme;
use egui::{Color32, Pos2, Stroke};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableColumnAlign {
    Left,
    Center,
    Right,
}

pub struct TableColumn<'a> {
    pub title: &'a str,
    pub width: Option<f32>,
    pub align: TableColumnAlign,
    pub sortable: bool,
}

impl<'a> TableColumn<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            width: None,
            align: TableColumnAlign::Left,
            sortable: false,
        }
    }

    pub fn fixed(title: &'a str, width: f32) -> Self {
        Self {
            title,
            width: Some(width),
            align: TableColumnAlign::Left,
            sortable: false,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the minimum outer width from measured content, including cell padding.
    pub fn content_width(mut self, width: f32) -> Self {
        self.width = Some(width + config::INNER_PADDING_X * 2.0);
        self
    }

    pub fn align(mut self, align: TableColumnAlign) -> Self {
        self.align = align;
        self
    }

    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }
}

pub(crate) fn draw_crisp_checkmark(painter: &egui::Painter, center: Pos2, color: Color32) {
    let p1 = Pos2::new(center.x - 3.8, center.y - 0.2);
    let p2 = Pos2::new(center.x - 0.9, center.y + 2.8);
    let p3 = Pos2::new(center.x + 3.8, center.y - 2.8);
    painter.add(egui::epaint::PathShape::line(
        vec![p1, p2, p3],
        Stroke::new(config::CHECKMARK_STROKE_WIDTH, color),
    ));
}

pub(crate) fn draw_crisp_minus(painter: &egui::Painter, center: Pos2, color: Color32) {
    let p1 = Pos2::new(center.x - 3.5, center.y);
    let p2 = Pos2::new(center.x + 3.5, center.y);
    painter.line_segment([p1, p2], Stroke::new(config::CHECKMARK_STROKE_WIDTH, color));
}

pub struct Table<'a> {
    columns: &'a [TableColumn<'a>],
    theme: DbProTheme,
    selectable: bool,
    all_selected: bool,
    indeterminate: bool,
    sort_column: Option<usize>,
    sort_desc: bool,
    row_height: f32,
    show_vertical_grid: bool,
    /// Per-row accessible name for the row's click region — real content,
    /// e.g. the row's primary column. Without it a selectable row is an
    /// unnamed interactive node for assistive tech.
    row_label: Option<&'a dyn Fn(usize) -> String>,
}

impl<'a> Table<'a> {
    pub fn new(columns: &'a [TableColumn<'a>], theme: DbProTheme) -> Self {
        Self {
            columns,
            theme,
            selectable: false,
            all_selected: false,
            indeterminate: false,
            sort_column: None,
            sort_desc: false,
            row_height: config::ROW_HEIGHT_DEFAULT,
            show_vertical_grid: false,
            row_label: None,
        }
    }

    pub fn selectable(mut self, selectable: bool, all_selected: bool) -> Self {
        self.selectable = selectable;
        self.all_selected = all_selected;
        self
    }

    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    pub fn sort(mut self, sort_column: Option<usize>, sort_desc: bool) -> Self {
        self.sort_column = sort_column;
        self.sort_desc = sort_desc;
        self
    }

    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height;
        self
    }

    pub fn vertical_grid(mut self, show: bool) -> Self {
        self.show_vertical_grid = show;
        self
    }

    /// Accessible name for each row's click region — supply real row content
    /// (e.g. its primary column) so AT can announce the row.
    pub fn row_label(mut self, label: &'a dyn Fn(usize) -> String) -> Self {
        self.row_label = Some(label);
        self
    }
}
