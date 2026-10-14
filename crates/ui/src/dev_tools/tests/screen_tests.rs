//! Tests for `../screen.rs` (moved out of the source file).
use egui::{Pos2, Rect};

use super::*;
use crate::dev_tools::geometry::GeometryRect;

/// One interactive widget on `layer`, parented to `parent` — the fixture
/// builder mirrors `audit::tests::widget` with a parent parameter.
fn widget_at(id: u64, _layer: u64, parent: u64, rect: Rect) -> WidgetGeometry {
    let l = scope_layer(1);
    WidgetGeometry {
        id: Id::new(id),
        id_debug: Id::new(id).short_debug_format(),
        parent_id: Id::new(parent),
        parent_debug: Id::new(parent).short_debug_format(),
        layer: l,
        layer_debug: l.short_debug_format(),
        rect: GeometryRect::from_egui(rect),
        interact_rect: GeometryRect::from_egui(rect),
        clipped: false,
        senses_click: true,
        senses_drag: false,
        focusable: false,
        enabled: true,
        widget_type: Some("Button".into()),
        label: None,
        hint_text: None,
        current_text_value: None,
        selected: None,
        value: None,
    }
}

fn snapshot(widgets: Vec<WidgetGeometry>) -> GeometrySnapshot {
    let mut groups: Vec<(egui::LayerId, Vec<WidgetGeometry>)> = Vec::new();
    for w in widgets {
        match groups.iter_mut().find(|(l, _)| *l == w.layer) {
            Some((_, ws)) => ws.push(w),
            None => groups.push((w.layer, vec![w])),
        }
    }
    GeometrySnapshot { layers: groups }
}

/// The scope layer all fixture widgets live on — one layer id, so the
/// fixture filter keeps them in scope.
fn scope_layer(salt: u64) -> egui::LayerId {
    egui::LayerId::new(egui::Order::Middle, Id::new(("scr", salt)))
}

/// A virtual 800×600 screen — the fixture scope used by every test.
/// Widgets land on `scope_layer(1)`; the scope filter matches that
/// family by order so sibling layers (a dialog) also stay visible.
fn scope() -> (ScreenScope, egui::LayerId) {
    let layer = scope_layer(1);
    (
        ScreenScope::Fixture {
            layer,
            rect: Rect::from_min_size(Pos2::new(0.0, 0.0), egui::vec2(800.0, 600.0)),
        },
        layer,
    )
}

fn analyze(widgets: Vec<WidgetGeometry>, sem: Option<&SemanticSnapshot>) -> ScreenAnalysis {
    let ctx = egui::Context::default();
    let (scope, _) = scope();
    analyze_screen(&ctx, &snapshot(widgets), sem, &scope)
}

fn findings_for<'a>(a: &'a ScreenAnalysis, rule: &str) -> Vec<&'a ScreenFinding> {
    a.findings.iter().filter(|f| f.rule == rule).collect()
}

/// Column layout knobs for the uniform-run fixture.
struct Column {
    parent: u64,
    x: f32,
    y0: f32,
    gap: f32,
    size: egui::Vec2,
    n: u64,
}

/// A uniform n-item column, all in one parent — the rhythm-fixture base.
fn column(c: Column) -> Vec<WidgetGeometry> {
    (0..c.n)
        .map(|i| {
            widget_at(
                1000 + i,
                1,
                c.parent,
                Rect::from_min_size(Pos2::new(c.x, c.y0 + i as f32 * (c.size.y + c.gap)), c.size),
            )
        })
        .collect()
}

// ── region detection + classification ────────────────────────────

#[test]
fn regions_split_on_whitespace_and_classify_chrome() {
    let ctx = egui::Context::default();
    let (scope, layer) = scope();
    let mut ws = Vec::new();
    // Top bar: 4 buttons in a row.
    for i in 0..4 {
        ws.push(widget_at(
            10 + i,
            1,
            100,
            Rect::from_min_size(Pos2::new(8.0 + i as f32 * 88.0, 4.0), egui::vec2(80.0, 28.0)),
        ));
    }
    // Rail: 7 items running down the left edge (55% of screen height —
    // tall enough to count as edge chrome, not a floating nav block).
    ws.extend(column(Column {
        parent: 200,
        x: 8.0,
        y0: 48.0,
        gap: 8.0,
        size: egui::vec2(120.0, 40.0),
        n: 7,
    }));
    // Content rows on the right.
    ws.extend(column(Column {
        parent: 300,
        x: 160.0,
        y0: 48.0,
        gap: 8.0,
        size: egui::vec2(200.0, 32.0),
        n: 5,
    }));
    let snap = snapshot(ws);
    let a = analyze_screen(&ctx, &snap, None, &scope);
    let kinds: Vec<_> = a.regions.iter().map(|r| r.kind.label()).collect();
    assert!(kinds.contains(&"top_bar"), "top strip must classify: {kinds:?}");
    assert!(kinds.contains(&"left_rail"), "left column must classify: {kinds:?}");
    assert!(
        kinds.contains(&"side_panel") || kinds.contains(&"content"),
        "interior column must classify: {kinds:?}"
    );
    let _ = layer;
}

#[test]
fn empty_scope_is_insufficient_not_clean() {
    let a = analyze(Vec::new(), None);
    assert!(!a.sufficient);
    assert!(a.gaps.contains(&AnalysisGap::NoGeometry));
    assert!(a.gaps.contains(&AnalysisGap::NoSemanticTree));
    assert!(a.hierarchy.nodes.is_none(), "missing data must read UNKNOWN");
}

#[test]
fn semantic_tree_fills_hierarchy() {
    use egui::accesskit::{Node, Role, Tree};
    let root = egui::accesskit_root_id().accesskit_id();
    let child = Id::new("c").accesskit_id();
    let mut r = Node::new(Role::Window);
    r.push_child(child);
    let sem = SemanticSnapshot::from_update(&egui::accesskit::TreeUpdate {
        nodes: vec![(root, r), (child, Node::new(Role::Button))],
        tree: Some(Tree::new(root)),
        focus: root,
        tree_id: egui::accesskit::TreeId::ROOT,
    })
    .unwrap();
    let a = analyze(Vec::new(), Some(&sem));
    assert_eq!(a.hierarchy.nodes, Some(2));
    assert_eq!(a.hierarchy.max_depth, Some(2));
}

// ── findings: fixtures prove fire and don't-fire ─────────────────

#[test]
fn off_screen_flags_widget_outside_viewport() {
    let mut ws = column(Column {
        parent: 1,
        x: 8.0,
        y0: 8.0,
        gap: 8.0,
        size: egui::vec2(100.0, 32.0),
        n: 4,
    });
    // Interactive rect fully outside the 800×600 fixture screen.
    ws.push(widget_at(
        9,
        1,
        2,
        Rect::from_min_size(Pos2::new(820.0, 10.0), egui::vec2(80.0, 28.0)),
    ));
    let a = analyze(ws, None);
    let hits = findings_for(&a, "screen.off_screen");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].kind, FindingKind::Objective);
    assert_eq!(hits[0].severity, Severity::Warning);
}

#[test]
fn off_screen_ignores_widget_inside() {
    let ws = column(Column {
        parent: 1,
        x: 8.0,
        y0: 8.0,
        gap: 8.0,
        size: egui::vec2(100.0, 32.0),
        n: 4,
    });
    let a = analyze(ws, None);
    assert!(findings_for(&a, "screen.off_screen").is_empty());
}

#[test]
fn spacing_outlier_flags_interior_break_not_edge() {
    // Uniform run: rows at 8px pitch, but row 4 is 100px late — interior
    // break with siblings on both sides.
    let mut ws = Vec::new();
    let pitches = [8.0, 8.0, 8.0, 100.0, 8.0];
    let mut y = 40.0;
    for (i, p) in pitches.iter().enumerate() {
        if i > 0 {
            y += p;
        }
        ws.push(widget_at(
            50 + i as u64,
            1,
            1,
            Rect::from_min_size(Pos2::new(200.0, y), egui::vec2(150.0, 32.0)),
        ));
        y += 32.0;
    }
    let a = analyze(ws, None);
    let hits = findings_for(&a, "screen.spacing_outlier");
    assert_eq!(hits.len(), 1);
    assert!(hits[0].evidence.contains("100px"));
    assert_eq!(hits[0].kind, FindingKind::Heuristic);
}

#[test]
fn spacing_outlier_skips_edge_anchored_gap() {
    // Same rhythm but the big gap ends the run — a bottom-pinned item,
    // which is anchoring, not a rhythm defect.
    let mut ws = Vec::new();
    let pitches = [8.0, 8.0, 8.0, 8.0, 100.0];
    let mut y = 40.0;
    for (i, p) in pitches.iter().enumerate() {
        if i > 0 {
            y += p;
        }
        ws.push(widget_at(
            60 + i as u64,
            1,
            1,
            Rect::from_min_size(Pos2::new(200.0, y), egui::vec2(150.0, 32.0)),
        ));
        y += 32.0;
    }
    let a = analyze(ws, None);
    assert!(
        findings_for(&a, "screen.spacing_outlier").is_empty(),
        "edge gap is anchoring, not a defect"
    );
}

#[test]
fn spacing_outlier_skips_mixed_size_group() {
    // Justified toolbar: same large gap but items vary in height → not a
    // uniform run, so no rhythm exists to break.
    let mut ws = Vec::new();
    let mut y = 40.0;
    let heights = [32.0, 24.0, 40.0, 28.0, 36.0, 30.0];
    for (i, h) in heights.iter().enumerate() {
        if i > 0 {
            y += if i == 3 { 100.0 } else { 8.0 };
        }
        ws.push(widget_at(
            70 + i as u64,
            1,
            1,
            Rect::from_min_size(Pos2::new(200.0, y), egui::vec2(150.0, *h)),
        ));
        y += h;
    }
    let a = analyze(ws, None);
    assert!(
        findings_for(&a, "screen.spacing_outlier").is_empty(),
        "mixed-size groups have no dominant rhythm to break"
    );
}

#[test]
fn layout_inversion_flags_dominant_side_panel() {
    let ctx = egui::Context::default();
    let (scope, layer) = scope();
    let mut ws = Vec::new();
    // Side-anchored, non-grid panel occupying 62% of the screen width.
    ws.extend(column(Column {
        parent: 1,
        x: 8.0,
        y0: 48.0,
        gap: 8.0,
        size: egui::vec2(480.0, 30.0),
        n: 8,
    }));
    // Narrow content column on the right.
    ws.extend(column(Column {
        parent: 2,
        x: 540.0,
        y0: 48.0,
        gap: 8.0,
        size: egui::vec2(200.0, 30.0),
        n: 8,
    }));
    let snap = snapshot(ws);
    let a = analyze_screen(&ctx, &snap, None, &scope);
    let hits = findings_for(&a, "screen.layout_inversion");
    assert_eq!(hits.len(), 1, "side panel outweighing content must flag");
    assert!(hits[0].evidence.contains('%'));
    let _ = layer;
}

#[test]
fn layout_inversion_passes_normal_layout() {
    let ctx = egui::Context::default();
    let (scope, _l) = scope();
    let mut ws = Vec::new();
    ws.extend(column(Column {
        parent: 1,
        x: 8.0,
        y0: 48.0,
        gap: 8.0,
        size: egui::vec2(120.0, 30.0),
        n: 8,
    }));
    ws.extend(column(Column {
        parent: 2,
        x: 200.0,
        y0: 48.0,
        gap: 8.0,
        size: egui::vec2(500.0, 30.0),
        n: 8,
    }));
    let snap = snapshot(ws);
    let a = analyze_screen(&ctx, &snap, None, &scope);
    assert!(findings_for(&a, "screen.layout_inversion").is_empty());
}
#[test]
fn layout_inversion_ignores_right_edge_content() {
    // The working surface legitimately fills to the right edge — IDE
    // panes are flush by design. Only a dominant LEFT pane inverts.
    let ctx = egui::Context::default();
    let (scope, _l) = scope();
    let mut ws = Vec::new();
    // Narrow left rail, content column flush to the right edge (200→800).
    ws.extend(column(Column {
        parent: 1,
        x: 8.0,
        y0: 48.0,
        gap: 8.0,
        size: egui::vec2(120.0, 30.0),
        n: 8,
    }));
    ws.extend(column(Column {
        parent: 2,
        x: 200.0,
        y0: 48.0,
        gap: 8.0,
        size: egui::vec2(600.0, 30.0),
        n: 8,
    }));
    let snap = snapshot(ws);
    let a = analyze_screen(&ctx, &snap, None, &scope);
    assert!(
        findings_for(&a, "screen.layout_inversion").is_empty(),
        "right-anchored work surface is not navigation"
    );
}

#[test]
fn no_primary_needs_busy_fragmented_screen() {
    // Sparse screen: two small regions, little interactive coverage —
    // a welcome screen legitimately has no dominant region.
    let ws = column(Column {
        parent: 1,
        x: 100.0,
        y0: 100.0,
        gap: 8.0,
        size: egui::vec2(100.0, 28.0),
        n: 4,
    });
    let a = analyze(ws, None);
    assert!(findings_for(&a, "screen.no_primary_region").is_empty());
}

#[test]
fn findings_report_kind_not_opinion() {
    // Every finding carries an explicit objective/heuristic split — the
    // contract that keeps measured defects separate from taste.
    let mut ws = column(Column {
        parent: 1,
        x: 8.0,
        y0: 8.0,
        gap: 8.0,
        size: egui::vec2(100.0, 32.0),
        n: 4,
    });
    ws.push(widget_at(
        9,
        1,
        2,
        Rect::from_min_size(Pos2::new(820.0, 10.0), egui::vec2(80.0, 28.0)),
    ));
    let a = analyze(ws, None);
    for f in &a.findings {
        assert!(matches!(f.kind, FindingKind::Objective | FindingKind::Heuristic));
        assert!(!f.evidence.is_empty(), "finding without evidence violates the contract");
    }
}
