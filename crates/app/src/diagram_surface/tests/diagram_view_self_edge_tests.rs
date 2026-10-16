//! Tests for `../diagram_view.rs` self-edge routing (#512).
    use super::*;

    fn bend(from_x: f32, to_x: f32, self_edge: bool, edge_id: usize, zoom: f32) -> f32 {
        edge_route_bend_x(EdgeBendSpec {
            from_x,
            to_x,
            self_edge,
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
    fn self_edge_bend_loops_outside_node_512() {
        // Both anchors on the right edge (x=380), possibly different rows.
        for zoom in [0.5, 1.0, 1.5] {
            for edge_id in 0..14 {
                let bend = bend(380.0, 380.0, true, edge_id, zoom);
                assert!(
                    bend > 380.0,
                    "self-edge loop must stay right of the node: edge={edge_id} zoom={zoom} bend={bend}"
                );
            }
        }
    }

    #[test]
    fn self_edge_bend_fans_out_per_edge_512() {
        let bends: Vec<f32> = (0..5).map(|id| bend(380.0, 380.0, true, id, 1.0)).collect();
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

    #[test]
    fn normal_edge_route_keeps_midpoint_lane_formula_512() {
        // Regression guard: non-self edges are untouched by the #512 branch.
        for edge_id in 0..14 {
            for zoom in [0.5, 1.0, 2.0] {
                let (from_x, to_x) = (100.0_f32, 500.0_f32);
                let expected = (from_x + to_x) / 2.0 + ((edge_id % 7) as f32 - 3.0) * 9.0 * zoom;
                let actual = bend(from_x, to_x, false, edge_id, zoom);
                assert!(
                    (actual - expected).abs() < f32::EPSILON,
                    "normal edge route changed: edge={edge_id} zoom={zoom}"
                );
            }
        }
    }
