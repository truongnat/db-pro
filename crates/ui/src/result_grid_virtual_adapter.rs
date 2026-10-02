//! Incremental vertical result-grid virtualization adapter.
//!
//! The adapter receives the filtered projection's row count and egui's host scroll
//! offset. DB Pro retains projection, column, selection, and cell-rendering state.

use rs_ui_core::{Point, Size};
use rs_ui_runtime::{ScrollState, VirtualGrid, VirtualViewportError};

pub(crate) const RESULT_GRID_ROW_HEIGHT: f32 = 28.0;
const ROW_OVERSCAN: usize = 1;

pub(crate) struct VirtualGridAdapterInput {
    pub(crate) row_count: usize,
    pub(crate) viewport_height: f32,
    pub(crate) scroll_offset_y: f32,
}

#[derive(Default)]
pub(crate) struct ResultGridVirtualRuntime {
    grid: Option<VirtualGrid>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VirtualGridWindow {
    pub(crate) positions: std::ops::Range<usize>,
}

pub(crate) fn adapt_virtual_grid_window(
    runtime: &mut ResultGridVirtualRuntime,
    input: VirtualGridAdapterInput,
) -> Result<VirtualGridWindow, VirtualViewportError> {
    let viewport = Size::new(0.0, input.viewport_height);
    let content = Size::new(
        0.0,
        input.row_count as f32 * RESULT_GRID_ROW_HEIGHT,
    );
    let mut scroll = ScrollState::new(viewport, content);
    scroll.set_offset(Point::new(0.0, input.scroll_offset_y));

    if runtime
        .grid
        .as_ref()
        .is_none_or(|grid| grid.rows.item_count() != input.row_count)
    {
        runtime.grid = Some(VirtualGrid::new_fixed(
            input.row_count,
            0,
            RESULT_GRID_ROW_HEIGHT,
            1.0,
            viewport,
            ROW_OVERSCAN,
        )?);
    }
    let Some(grid) = runtime.grid.as_mut() else {
        return Err(VirtualViewportError::InvalidExtent);
    };
    grid.set_viewport(viewport);
    grid.set_scroll_offset(scroll.offset);

    Ok(VirtualGridWindow {
        positions: grid.rows.first_visible..grid.rows.last_visible,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_window_uses_filtered_projection_row_count() {
        let mut runtime = ResultGridVirtualRuntime::default();
        let window = adapt_virtual_grid_window(
            &mut runtime,
            VirtualGridAdapterInput {
                row_count: 100,
                viewport_height: RESULT_GRID_ROW_HEIGHT * 2.0,
                scroll_offset_y: RESULT_GRID_ROW_HEIGHT * 50.0,
            },
        )
        .unwrap();

        assert_eq!(window.positions, 49..53);
    }

    #[test]
    fn virtual_window_handles_empty_and_single_row_projections() {
        let mut runtime = ResultGridVirtualRuntime::default();
        for row_count in [0, 1] {
            let window = adapt_virtual_grid_window(
                &mut runtime,
                VirtualGridAdapterInput {
                    row_count,
                    viewport_height: RESULT_GRID_ROW_HEIGHT * 2.0,
                    scroll_offset_y: 0.0,
                },
            )
            .unwrap();
            assert_eq!(window.positions, 0..row_count);
        }
    }

    #[test]
    fn virtual_window_clamps_middle_and_end_offsets_and_limits_overscan() {
        let mut runtime = ResultGridVirtualRuntime::default();
        let input = |scroll_offset_y| VirtualGridAdapterInput {
            row_count: 10,
            viewport_height: RESULT_GRID_ROW_HEIGHT * 2.0,
            scroll_offset_y,
        };

        let first = adapt_virtual_grid_window(&mut runtime, input(0.0)).unwrap();
        assert_eq!(first.positions, 0..3);
        let middle = adapt_virtual_grid_window(
            &mut runtime,
            input(RESULT_GRID_ROW_HEIGHT * 3.0),
        )
        .unwrap();
        assert_eq!(middle.positions, 2..6);
        let end = adapt_virtual_grid_window(&mut runtime, input(f32::MAX)).unwrap();
        assert_eq!(end.positions, 7..10);
        assert!(end.positions.end <= 10);
    }

    #[test]
    fn virtual_window_arithmetic_stays_in_bounds_through_one_million_rows() {
        for row_count in [10_000, 100_000, 1_000_000] {
            let mut runtime = ResultGridVirtualRuntime::default();
            let offset = row_count as f32 * RESULT_GRID_ROW_HEIGHT * 0.5;
            let window = adapt_virtual_grid_window(
                &mut runtime,
                VirtualGridAdapterInput {
                    row_count,
                    viewport_height: RESULT_GRID_ROW_HEIGHT * 2.0,
                    scroll_offset_y: offset,
                },
            )
            .unwrap();
            assert!(window.positions.start < window.positions.end);
            assert!(window.positions.end <= row_count);
            assert!(window.positions.len() <= 4);
        }
    }
}
