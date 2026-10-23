//! Tests for `../diagram_view.rs` self-edge routing (#512).
use super::*;

/// One Standard-LOD card: `ER_NODE_WIDTH` wide, 184pt tall.
fn card(left: f32, top: f32) -> egui::Rect {
    egui::Rect::from_min_size(egui::pos2(left, top), egui::vec2(ER_NODE_WIDTH, 184.0))
}

fn self_route(rect: egui::Rect, edge_id: usize, zoom: f32) -> EdgeRoute {
    edge_route(EdgeRouteSpec {
        from: egui::pos2(rect.right(), rect.top() + 76.0),
        to: egui::pos2(rect.right(), rect.top() + 124.0),
        from_rect: rect,
        to_rect: rect,
        from_right: true,
        to_right: true,
        self_edge: true,
        edge_id,
        zoom,
    })
}

/// Old route (pre-#512): opposite-side anchors with a midpoint bend.
/// With `from` on the left edge and `to` on the right edge of the same
/// 280-wide card, the lane (|lane| <= 27*zoom) can never push the bend
/// outside the card (half-width 140*zoom) — the edge always crossed the
/// node body. Kept as the fail-before record; the fixed route must not
/// behave this way.
fn old_bend_x(from_x: f32, to_x: f32, edge_id: usize, zoom: f32) -> f32 {
    let lane = ((edge_id % 7) as f32 - 3.0) * 9.0 * zoom;
    (from_x + to_x) / 2.0 + lane
}

#[test]
fn old_route_bend_stays_inside_same_node_512() {
    // Card from x=100 to x=380 (280 wide, ER_NODE_WIDTH), zoom 1.
    let (left, right) = (100.0_f32, 380.0_f32);
    for edge_id in 0..14 {
        let bend = old_bend_x(left, right, edge_id, 1.0);
        assert!(
            (left..=right).contains(&bend),
            "old route crosses the card for edge {edge_id}: bend={bend}"
        );
    }
}

#[test]
fn self_edge_loops_outside_node_512() {
    let rect = card(100.0, 40.0);
    for zoom in [0.5, 1.0, 1.5] {
        for edge_id in 0..14 {
            let route = self_route(rect, edge_id, zoom);
            for (start, end) in route.segments() {
                assert!(
                    !segment_crosses_rect(start, end, rect),
                    "self-edge {edge_id} at zoom {zoom} crosses its own card: {start:?}->{end:?}"
                );
            }
            assert!(
                route.label_x() > rect.right(),
                "self-edge loop must stay right of the node: edge={edge_id} zoom={zoom} x={}",
                route.label_x()
            );
        }
    }
}

#[test]
fn self_edge_bend_fans_out_per_edge_512() {
    let rect = card(100.0, 40.0);
    let bends: Vec<f32> = (0..5).map(|id| self_route(rect, id, 1.0).label_x()).collect();
    for pair in bends.windows(2) {
        assert!(
            (pair[1] - pair[0]).abs() > 0.0,
            "parallel self-edges must not share one bend: {bends:?}"
        );
        assert!(
            (pair[1] - pair[0] - 8.0).abs() < f32::EPSILON,
            "fan-out step is 8px at zoom 1: {bends:?}"
        );
    }
}
