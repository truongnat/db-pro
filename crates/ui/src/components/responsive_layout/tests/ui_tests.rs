//! Tests for `../ui.rs` (moved out of the source file).
use super::*;

#[test]
fn container_is_bounded_and_centered() {
    assert_eq!(
        container_width(900.0, Some(600.0), 20.0),
        ContainerWidth {
            content_width: 600.0,
            gutter: 150.0
        }
    );
    assert_eq!(
        container_width(30.0, Some(600.0), 20.0),
        ContainerWidth {
            content_width: 0.0,
            gutter: 15.0
        }
    );
}

#[test]
fn grid_respects_minimum_gap_and_column_cap() {
    assert_eq!(
        grid_metrics(500.0, 100.0, 20.0, 3),
        GridMetrics {
            columns: 3,
            cell_width: 153.33333,
            gap: 20.0
        }
    );
    assert_eq!(grid_metrics(219.0, 100.0, 20.0, 4).columns, 1);
    assert_eq!(grid_metrics(0.0, 100.0, 20.0, 4).cell_width, 0.0);
    assert_eq!(grid_metrics(900.0, 0.0, 0.0, 0).columns, 1);
}

#[test]
fn invalid_inputs_are_safe() {
    let result = grid_metrics(f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 2);
    assert_eq!(
        result,
        GridMetrics {
            columns: 2,
            cell_width: 0.0,
            gap: 0.0
        }
    );
}

#[test]
fn container_consumes_intrinsic_height_before_following_sibling() {
    let context = egui::Context::default();
    let mut child_bounds = Vec::new();
    let mut sibling_bounds = None;
    let _ = crate::test_frame::frame(&context, egui::RawInput::default(), |ui| {
        egui::CentralPanel::default().show(ui, |ui| {
            Container::new().max_width(100.0).show(ui, |content| {
                content.allocate_exact_size(Vec2::new(40.0, 30.0), Sense::hover());
                child_bounds.push(content.min_rect());
            });
            sibling_bounds = Some(ui.allocate_exact_size(Vec2::new(20.0, 10.0), Sense::hover()).0);
        });
    });
    let content = child_bounds[0];
    let sibling = sibling_bounds.unwrap();
    assert!(sibling.top() >= content.bottom());
    assert!(content.width() <= 100.0);
}

#[test]
fn container_sequential_siblings_and_narrow_bounds() {
    let context = egui::Context::default();
    let mut first_child = None;
    let mut second_child = None;
    let mut parent_resp_1 = None;
    let mut parent_resp_2 = None;
    let _ = crate::test_frame::frame(&context, egui::RawInput::default(), |ui| {
        egui::CentralPanel::default().show(ui, |ui| {
            let (r1, _) = Container::new().max_width(50.0).gutter(10.0).show(ui, |content| {
                content.allocate_exact_size(Vec2::new(30.0, 40.0), Sense::hover());
                first_child = Some(content.min_rect());
            });
            parent_resp_1 = Some(r1);

            let (r2, _) = Container::new().max_width(80.0).show(ui, |content| {
                content.allocate_exact_size(Vec2::new(20.0, 25.0), Sense::hover());
                second_child = Some(content.min_rect());
            });
            parent_resp_2 = Some(r2);
        });
    });

    let c1 = first_child.unwrap();
    let c2 = second_child.unwrap();
    let p1 = parent_resp_1.unwrap().rect;
    let p2 = parent_resp_2.unwrap().rect;

    assert!(
        p2.top() >= p1.bottom(),
        "Sequential container must stack below previous"
    );
    assert!(c2.top() >= c1.bottom(), "Child 2 must start below child 1");
    assert!(c1.width() <= 50.0);
    assert!(c2.width() <= 80.0);
}

#[test]
fn grid_cells_render_with_bounded_width_and_intrinsic_row_height() {
    let context = egui::Context::default();
    let mut bounds = Vec::new();
    let _ = crate::test_frame::frame(&context, egui::RawInput::default(), |ui| {
        egui::CentralPanel::default().show(ui, |ui| {
            ResponsiveGrid::new(80.0)
                .gap(8.0)
                .max_columns(2)
                .show(ui, [30.0, 50.0, 20.0], |cell, height| {
                    cell.allocate_exact_size(Vec2::new(40.0, height), Sense::hover());
                    bounds.push(cell.min_rect());
                });
        });
    });
    assert_eq!(bounds.len(), 3);
    assert!(bounds.iter().all(|rect| rect.width() <= 80.0));
    // First row contains bounds[0] and bounds[1], second row contains bounds[2]
    assert!(bounds[0].left() < bounds[1].left());
    assert_eq!(bounds[0].top(), bounds[1].top());
    assert!((bounds[2].top() - bounds[1].bottom() - 8.0).abs() < 0.1);
}
#[test]
fn grid_gutters_do_not_leak_into_cell_content_or_parent() {
    let context = egui::Context::default();
    let _ = crate::test_frame::frame(&context, egui::RawInput::default(), |ui| {
        egui::CentralPanel::default().show(ui, |ui| {
            let spacing = Vec2::new(8.0, 4.0);
            ui.spacing_mut().item_spacing = spacing;
            for gap in [0.0, 12.0, 16.0] {
                ResponsiveGrid::new(80.0)
                    .gap(gap)
                    .max_columns(2)
                    .show(ui, 0..3, |cell, _| {
                        assert_eq!(cell.spacing().item_spacing, spacing);
                        cell.allocate_exact_size(Vec2::new(40.0, 10.0), Sense::hover());
                    });
                assert_eq!(ui.spacing().item_spacing, spacing);
            }
        });
    });
}
