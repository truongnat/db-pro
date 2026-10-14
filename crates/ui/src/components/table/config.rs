//! Table-specific layout and painting metrics.
//!
//! Row height, header height, and corner radii come from the shared token scale.
//! The remaining values describe the table's own coordinate system.

use crate::tokens::{
    component::table::{TABLE_HEADER_HEIGHT, TABLE_ROW_HEIGHT_DEFAULT},
    RADIUS_SM, RADIUS_XS,
};

pub(crate) const CHECKBOX_COLUMN_WIDTH: f32 = 42.0;
pub(crate) const CHECKBOX_SIZE: f32 = 15.0;
pub(crate) const CHECKBOX_CORNER_RADIUS: f32 = RADIUS_XS;
pub(crate) const CHECKBOX_STROKE_WIDTH: f32 = 1.2;
pub(crate) const CHECKMARK_STROKE_WIDTH: f32 = 1.8;
pub(crate) const HEADER_HEIGHT: f32 = TABLE_HEADER_HEIGHT;
pub(crate) const HEADER_INNER_RADIUS: f32 = RADIUS_SM;
pub(crate) const TABLE_CORNER_RADIUS: f32 = 8.0;
pub(crate) const INNER_PADDING_X: f32 = 12.0;
pub(crate) const HEADER_TEXT_SIZE: f32 = 13.0;
pub(crate) const SORT_ICON_SIZE: f32 = 10.0;
pub(crate) const SORT_ICON_GAP: f32 = 4.0;
pub(crate) const SORT_ICON_FADE: f32 = 0.5;
pub(crate) const ROW_HEIGHT_DEFAULT: f32 = TABLE_ROW_HEIGHT_DEFAULT;
pub(crate) const EMPTY_BODY_HEIGHT: f32 = 80.0;
pub(crate) const EMPTY_BODY_RECT_HEIGHT: f32 = 90.0;
pub(crate) const EMPTY_BODY_TOP_SPACE: f32 = 20.0;
pub(crate) const EMPTY_ICON_SIZE: f32 = 20.0;
pub(crate) const EMPTY_ICON_GAP: f32 = 4.0;
pub(crate) const EMPTY_TEXT_SIZE: f32 = 13.0;
pub(crate) const MIN_NET_WIDTH: f32 = 100.0;
pub(crate) const FLEX_COLUMN_MIN_WIDTH: f32 = 80.0;
pub(crate) const SCALED_COLUMN_FALLBACK_WIDTH: f32 = 100.0;
pub(crate) const DEFAULT_COLUMN_WIDTH: f32 = 120.0;
pub(crate) const TABLE_WIDTH_BORDER_ALLOWANCE: f32 = 2.0;
pub(crate) const DIVIDER_STROKE_WIDTH: f32 = 1.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_metrics_preserve_the_existing_coordinate_contract() {
        assert_eq!(CHECKBOX_COLUMN_WIDTH, 42.0);
        assert_eq!(HEADER_HEIGHT, TABLE_HEADER_HEIGHT);
        assert_eq!(ROW_HEIGHT_DEFAULT, TABLE_ROW_HEIGHT_DEFAULT);
        assert_eq!(CHECKBOX_CORNER_RADIUS, RADIUS_XS);
        assert_eq!(HEADER_INNER_RADIUS, RADIUS_SM);
        assert_eq!(INNER_PADDING_X, 12.0);
    }
}
