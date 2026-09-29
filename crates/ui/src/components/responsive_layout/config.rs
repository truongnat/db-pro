//! Configuration normalization for responsive layout primitives.

/// Normalize a logical width or gap to a finite, non-negative value.
pub(crate) fn finite_nonnegative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

/// Keep column limits usable by the layout algorithm.
pub(crate) fn column_count(max_columns: usize) -> usize {
    max_columns.max(1)
}

/// Prevent gutters from consuming more than half of the available width.
pub(crate) fn bounded_gutter(available: f32, gutter: f32) -> f32 {
    finite_nonnegative(gutter).min(available * 0.5)
}
