use super::TableColumn;
use crate::components::table::config::{
    CHECKBOX_COLUMN_WIDTH, DIVIDER_STROKE_WIDTH, EMPTY_BODY_HEIGHT, HEADER_HEIGHT, MIN_NET_WIDTH,
    TABLE_WIDTH_BORDER_ALLOWANCE,
};

/// Column coordinates are computed once and reused by header, body, and gridline painting.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ColumnLayout {
    pub widths: Vec<f32>,
    pub offsets: Vec<f32>,
    pub checkbox_width: f32,
    pub total_width: f32,
}

pub(crate) fn build_column_layout(columns: &[TableColumn<'_>], available_width: f32, selectable: bool) -> ColumnLayout {
    let checkbox_width = if selectable { CHECKBOX_COLUMN_WIDTH } else { 0.0 };
    let net_width = (available_width - checkbox_width - TABLE_WIDTH_BORDER_ALLOWANCE).max(MIN_NET_WIDTH);
    let widths = compute_column_widths(columns, net_width);
    let offsets = column_offsets(&widths);
    let total_width = checkbox_width + widths.iter().sum::<f32>();

    ColumnLayout {
        widths,
        offsets,
        checkbox_width,
        total_width,
    }
}

fn compute_column_widths(columns: &[TableColumn<'_>], net_width: f32) -> Vec<f32> {
    let fixed_sum = columns.iter().filter_map(|column| column.width).sum::<f32>();
    let flex_count = columns.iter().filter(|column| column.width.is_none()).count();

    if flex_count > 0 {
        let flex_width =
            ((net_width - fixed_sum) / flex_count as f32).max(crate::components::table::config::FLEX_COLUMN_MIN_WIDTH);
        return columns
            .iter()
            .map(|column| column.width.unwrap_or(flex_width))
            .collect();
    }

    if fixed_sum > 0.0 && fixed_sum < net_width {
        let ratio = net_width / fixed_sum;
        return columns
            .iter()
            .map(|column| {
                column
                    .width
                    .unwrap_or(crate::components::table::config::SCALED_COLUMN_FALLBACK_WIDTH)
                    * ratio
            })
            .collect();
    }

    columns
        .iter()
        .map(|column| {
            column
                .width
                .unwrap_or(crate::components::table::config::DEFAULT_COLUMN_WIDTH)
        })
        .collect()
}

fn column_offsets(widths: &[f32]) -> Vec<f32> {
    let mut offsets = Vec::with_capacity(widths.len());
    let mut current = 0.0;
    for &width in widths {
        offsets.push(current);
        current += width;
    }
    offsets
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TableGeometry {
    pub header_height: f32,
    pub rows_height: f32,
    pub total_height: f32,
}

impl TableGeometry {
    pub(crate) fn new(row_count: usize, row_height: f32) -> Self {
        let rows_height = if row_count == 0 {
            EMPTY_BODY_HEIGHT
        } else {
            row_count as f32 * row_height
        };

        Self {
            header_height: HEADER_HEIGHT,
            rows_height,
            total_height: HEADER_HEIGHT + DIVIDER_STROKE_WIDTH + rows_height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectAllState {
    Unchecked,
    Indeterminate,
    Checked,
}

pub(crate) fn select_all_state(all_selected: bool, indeterminate: bool) -> SelectAllState {
    if all_selected {
        SelectAllState::Checked
    } else if indeterminate {
        SelectAllState::Indeterminate
    } else {
        SelectAllState::Unchecked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn columns() -> [TableColumn<'static>; 3] {
        [
            TableColumn::fixed("ID", 100.0),
            TableColumn::new("Name"),
            TableColumn::new("Status"),
        ]
    }

    #[test]
    fn layout_preserves_checkbox_space_and_column_offsets() {
        let layout = build_column_layout(&columns(), 500.0, true);

        assert_eq!(layout.checkbox_width, CHECKBOX_COLUMN_WIDTH);
        assert_eq!(layout.offsets, vec![0.0, 100.0, 278.0]);
        assert_eq!(layout.widths, vec![100.0, 178.0, 178.0]);
        assert_eq!(layout.total_width, 498.0);
    }

    #[test]
    fn geometry_keeps_empty_and_populated_table_heights_distinct() {
        assert_eq!(TableGeometry::new(0, 44.0).rows_height, EMPTY_BODY_HEIGHT);
        assert_eq!(TableGeometry::new(3, 44.0).rows_height, 132.0);
        assert_eq!(TableGeometry::new(3, 44.0).total_height, 169.0);
    }

    #[test]
    fn select_all_state_prioritizes_checked_over_indeterminate() {
        assert_eq!(select_all_state(false, false), SelectAllState::Unchecked);
        assert_eq!(select_all_state(false, true), SelectAllState::Indeterminate);
        assert_eq!(select_all_state(true, true), SelectAllState::Checked);
    }
}
