//! Layout decision helpers kept separate from widget presentation.

use super::config;

pub(crate) fn column_count(max_columns: usize) -> usize {
    config::column_count(max_columns)
}

pub(crate) fn bounded_gutter(available: f32, gutter: f32) -> f32 {
    config::bounded_gutter(available, gutter)
}
