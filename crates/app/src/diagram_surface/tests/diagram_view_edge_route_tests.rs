//! Routing tests for `../diagram_view.rs`: no FK edge segment may run through a
//! card. The fixture mirrors the capture schema — 10 tables in a 4-column grid
//! whose layered layout puts four parent/child pairs in the same grid column,
//! where the anchors' midpoint is inside both rects.
use super::*;

fn card(left: f32, top: f32, height: f32) -> egui::Rect {
    egui::Rect::from_min_size(egui::pos2(left, top), egui::vec2(ER_NODE_WIDTH, height))
}

fn scaled(rect: egui::Rect, zoom: f32) -> egui::Rect {
    egui::Rect::from_min_max(rect.min * zoom, rect.max * zoom)
}

/// Route between two cards, with both anchors on their second visible column
/// (`+76`), which is inside every LOD's card height.
fn route_between(from: egui::Rect, to: egui::Rect, edge_id: usize, zoom: f32) -> EdgeRoute {
    let (from_right, to_right) = edge_anchor_sides(from, to);
    edge_route(EdgeRouteSpec {
        from: egui::pos2(if from_right { from.right() } else { from.left() }, from.top() + 76.0),
        to: egui::pos2(if to_right { to.right() } else { to.left() }, to.top() + 76.0),
        from_rect: from,
        to_rect: to,
        from_right,
        to_right,
        self_edge: false,
        edge_id,
        zoom,
    })
}

fn assert_clear(route: &EdgeRoute, rects: &[(&str, egui::Rect)]) {
    for (start, end) in route.segments() {
        for (name, rect) in rects {
            assert!(
                !segment_crosses_rect(start, end, *rect),
                "segment {start:?}->{end:?} runs through `{name}`"
            );
        }
    }
}

#[test]
fn stacked_cards_route_outside_both_rects() {
    // Parent above child in one grid column: both anchor sides read "left" and
    // the midpoint of the anchors is the card center, inside both rects.
    let parent = card(48.0, 48.0, 184.0);
    let child = card(48.0, 308.0, 184.0);
    assert_eq!(edge_anchor_sides(parent, child), (true, true));
    for edge_id in 0..14 {
        let route = route_between(parent, child, edge_id, 1.0);
        assert_clear(&route, &[("parent", parent), ("child", child)]);
        assert!(
            route.label_x() > parent.right(),
            "stacked bend must sit in the column gap right of the pair: edge={edge_id} x={}",
            route.label_x()
        );
    }
}

#[test]
fn far_pair_detours_through_the_row_gap() {
    // Two columns and two rows apart: a bend at the anchor midpoint would run
    // along an anchor row and cut through the cards between the endpoints.
    let source = card(1140.0, 568.0, 184.0);
    let target = card(48.0, 48.0, 184.0);
    let route = route_between(source, target, 10, 1.0);
    assert_clear(&route, &[("source", source), ("target", target)]);
    // The run that carries the edge across columns sits in the row gap directly
    // below the target row, not inside the row between the two cards.
    let across = route.segments().nth(2).expect("detour segment");
    assert!(
        (across.0.y - across.1.y).abs() < f32::EPSILON,
        "third segment must be horizontal"
    );
    assert!(
        (across.0.y - (target.bottom() + ER_GAP_Y / 2.0)).abs() < f32::EPSILON,
        "detour must use the row gap: y={} expected={}",
        across.0.y,
        target.bottom() + ER_GAP_Y / 2.0
    );
}

#[test]
fn adjacent_columns_share_one_vertical_run() {
    // Neighbouring columns: both channels are the same gap, so the route stays
    // the plain three-segment step it always was.
    let left = card(48.0, 308.0, 184.0);
    let right = card(412.0, 308.0, 184.0);
    let route = route_between(left, right, 1, 1.0);
    assert_eq!(
        route.points[1].x, route.points[4].x,
        "adjacent columns must share the exit and entry channel"
    );
    assert_clear(&route, &[("left", left), ("right", right)]);
}

fn column(name: &str) -> crate::runtime::UiSchemaColumn {
    crate::runtime::UiSchemaColumn {
        name: name.to_owned(),
        data_type: "INTEGER".to_owned(),
        nullable: false,
        is_primary_key: false,
    }
}

fn table(name: &str, columns: &[&str], foreign_keys: &[(&str, &str)]) -> UiTableSummary {
    UiTableSummary {
        schema: "main".to_owned(),
        name: name.to_owned(),
        row_count: None,
        columns: columns.iter().copied().map(column).collect(),
        foreign_keys: foreign_keys
            .iter()
            .map(|(from_column, to_table)| crate::runtime::UiSchemaForeignKey {
                name: format!("{name}_{from_column}_fk"),
                from_columns: vec![(*from_column).to_owned()],
                to_schema: "main".to_owned(),
                to_table: (*to_table).to_owned(),
                to_columns: vec!["id".to_owned()],
            })
            .collect(),
    }
}

/// The `DB_PRO_CAPTURE_DIAGRAM` schema: 10 tables, 12 foreign keys, one
/// connected component.
fn fixture_tables() -> Vec<UiTableSummary> {
    vec![
        table("customers", &["id", "full_name", "email", "created_at"], &[]),
        table(
            "addresses",
            &["id", "customer_id", "street", "city", "country_code"],
            &[("customer_id", "customers")],
        ),
        table(
            "orders",
            &["id", "customer_id", "status", "ordered_at", "total"],
            &[("customer_id", "customers")],
        ),
        table(
            "order_items",
            &["order_id", "product_id", "quantity", "unit_price"],
            &[("order_id", "orders"), ("product_id", "products")],
        ),
        table(
            "products",
            &["id", "category_id", "name", "price", "stock"],
            &[("category_id", "categories")],
        ),
        table(
            "categories",
            &["id", "name", "parent_id"],
            &[("parent_id", "categories")],
        ),
        table(
            "payments",
            &["id", "order_id", "method", "amount", "paid_at"],
            &[("order_id", "orders")],
        ),
        table(
            "shipments",
            &["id", "order_id", "address_id", "carrier", "shipped_at"],
            &[("order_id", "orders"), ("address_id", "addresses")],
        ),
        table(
            "reviews",
            &["id", "product_id", "customer_id", "rating", "comment"],
            &[("product_id", "products"), ("customer_id", "customers")],
        ),
        table(
            "inventory_logs",
            &["id", "product_id", "delta", "logged_at"],
            &[("product_id", "products")],
        ),
    ]
}

#[test]
fn fixture_edges_never_cross_a_card() {
    let tables = fixture_tables();
    // (zoom, card height the LOD lays out) — Compact, Standard, Detailed.
    for (zoom, node_height) in [(0.4375_f32, 40.0_f32), (0.5745, 184.0), (1.2, 328.0)] {
        assert!(
            (diagram_node_height(zoom) - node_height).abs() < f32::EPSILON,
            "zoom {zoom} must lay out {node_height}pt cards"
        );
        let lod = ErLod::from_zoom(zoom);
        let graph = ErGraph::build(&tables, 1, (4usize, node_height));
        assert_eq!(graph.edges.len(), 12);

        let mut stacked = 0;
        for edge in &graph.edges {
            if edge.source == edge.target {
                continue;
            }
            let from = graph.nodes[edge.source].world_rect;
            let to = graph.nodes[edge.target].world_rect;
            if from.left() < to.right() && to.left() < from.right() {
                stacked += 1;
            }
        }
        assert_eq!(stacked, 4, "fixture keeps four parent/child pairs in one column");

        for edge in &graph.edges {
            let source = &graph.nodes[edge.source];
            let target = &graph.nodes[edge.target];
            let self_edge = edge.source == edge.target;
            let (from_right, to_right) = edge_anchor_sides(source.world_rect, target.world_rect);
            let from_spec = ErAnchorSpec {
                right_side: from_right,
                max_columns: lod.max_columns(),
            };
            let to_spec = ErAnchorSpec {
                right_side: to_right,
                max_columns: lod.max_columns(),
            };
            let from_world = source.column_anchor(edge.foreign_key.from_columns.first().map(String::as_str), from_spec);
            let to_world = target.column_anchor(edge.foreign_key.to_columns.first().map(String::as_str), to_spec);
            // The paint path works in screen space: world geometry scaled by the
            // zoom, with gaps and lanes scaled with it.
            let route = edge_route(EdgeRouteSpec {
                from: from_world * zoom,
                to: to_world * zoom,
                from_rect: scaled(source.world_rect, zoom),
                to_rect: scaled(target.world_rect, zoom),
                from_right,
                to_right,
                self_edge,
                edge_id: edge.id,
                zoom,
            });
            for (start, end) in route.segments() {
                for node in &graph.nodes {
                    assert!(
                        !segment_crosses_rect(start, end, scaled(node.world_rect, zoom)),
                        "edge {} ({} → {}) crosses `{}` at zoom {zoom}",
                        edge.id,
                        source.table.name,
                        target.table.name,
                        node.table.name
                    );
                }
            }
        }
    }
}
