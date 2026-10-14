//! Tests for `../audit.rs` (moved out of the source file).
use egui::{Pos2, Rect};

use super::*;
use crate::dev_tools::geometry::GeometryRect;

/// Minimal widget fixture — rule-irrelevant fields fixed so tests read as
/// pure geometry.
pub(crate) fn widget(id: u64, layer_id: u64, rect: Rect, interact: Rect) -> WidgetGeometry {
    let layer = egui::LayerId {
        order: egui::Order::Middle,
        id: Id::new(layer_id),
    };
    WidgetGeometry {
        id: Id::new(id),
        id_debug: Id::new(id).short_debug_format(),
        parent_id: Id::new("ui"),
        parent_debug: "ui".into(),
        layer,
        layer_debug: layer.short_debug_format(),
        rect: GeometryRect::from_egui(rect),
        interact_rect: GeometryRect::from_egui(interact),
        clipped: rect != interact,
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

fn issues_for<'r>(report: &'r AuditReport, rule: &str) -> Vec<&'r AuditIssue> {
    report.issues.iter().filter(|i| i.rule_id == rule).collect()
}

fn rule_stats<'r>(report: &'r AuditReport, rule: &str) -> &'r RuleStats {
    &report.rules.iter().find(|r| r.rule_id == rule).unwrap().stats
}

// ── geometry.invalid ─────────────────────────────────────────────

#[test]
fn invalid_geometry_flags_nan() {
    let ctx = egui::Context::default();
    // egui::Rect panics on NaN construction — inject it post-build.
    let mut w = widget(
        1,
        1,
        Rect::from_min_size(Pos2::ZERO, egui::vec2(10.0, 10.0)),
        Rect::from_min_size(Pos2::ZERO, egui::vec2(10.0, 10.0)),
    );
    w.rect.min_x = f32::NAN;
    let report = run_audit(&ctx, &snapshot(vec![w]), None);
    let issues = issues_for(&report, "geometry.invalid");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].severity, Severity::Error);
    assert_eq!(issues[0].confidence, Confidence::High);
    assert!(issues[0].evidence.contains("non-finite"));
}

#[test]
fn invalid_geometry_flags_inverted() {
    let ctx = egui::Context::default();
    let r = Rect::from_min_max(Pos2::new(10.0, 0.0), Pos2::new(5.0, 10.0));
    let report = run_audit(&ctx, &snapshot(vec![widget(1, 1, r, r)]), None);
    let issues = issues_for(&report, "geometry.invalid");
    assert_eq!(issues.len(), 1);
    assert!(issues[0].evidence.contains("inverted"));
}

#[test]
fn invalid_geometry_passes_normal_rect() {
    let ctx = egui::Context::default();
    let r = Rect::from_min_size(Pos2::ZERO, egui::vec2(50.0, 20.0));
    let report = run_audit(&ctx, &snapshot(vec![widget(1, 1, r, r)]), None);
    assert!(issues_for(&report, "geometry.invalid").is_empty());
}

// ── interactive.zero_size ────────────────────────────────────────

#[test]
fn zero_size_flags_unclipped_zero_interact() {
    let ctx = egui::Context::default();
    // Genuine zero hitbox, no clipping → reachable in the tree but dead.
    let mut w = widget(1, 1, Rect::from_min_size(Pos2::ZERO, egui::vec2(0.0, 0.0)), Rect::ZERO);
    w.clipped = false;
    let report = run_audit(&ctx, &snapshot(vec![w]), None);
    let issues = issues_for(&report, "interactive.zero_size");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].severity, Severity::Error);
    assert!(issues[0].evidence.contains("floor"));
}

#[test]
fn zero_size_skips_clipped_widget() {
    let ctx = egui::Context::default();
    // rect≠interact ⇒ clipped=true by fixture — parent clip shrank the
    // hitbox, geometry itself is fine → SKIP, don't guess.
    let w = widget(
        1,
        1,
        Rect::from_min_size(Pos2::ZERO, egui::vec2(50.0, 20.0)),
        Rect::from_min_size(Pos2::ZERO, egui::vec2(0.0, 0.0)),
    );
    let report = run_audit(&ctx, &snapshot(vec![w]), None);
    assert!(issues_for(&report, "interactive.zero_size").is_empty());
    let stats = rule_stats(&report, "interactive.zero_size");
    assert_eq!(stats.skipped, 1);
    assert_eq!(stats.skip_reason, Some(SkipReason::Clipped));
}

#[test]
fn zero_size_ignores_non_interactive() {
    let ctx = egui::Context::default();
    let mut w = widget(1, 1, Rect::ZERO, Rect::ZERO);
    w.senses_click = false;
    w.senses_drag = false;
    w.focusable = false;
    let report = run_audit(&ctx, &snapshot(vec![w]), None);
    assert!(issues_for(&report, "interactive.zero_size").is_empty());
}

#[test]
fn zero_size_ignores_disabled() {
    let ctx = egui::Context::default();
    let mut w = widget(1, 1, Rect::ZERO, Rect::ZERO);
    w.enabled = false;
    let report = run_audit(&ctx, &snapshot(vec![w]), None);
    assert!(issues_for(&report, "interactive.zero_size").is_empty());
}

// ── overlap.suspicious ───────────────────────────────────────────

#[test]
fn overlap_flags_sibling_covering_sibling() {
    let ctx = egui::Context::default();
    // b overlaps x 5..105 over a's x 0..100 → 95×19 of a's 100×20 ≈ 90%,
    // no containment (b extends past a on x, a past b on y? no: y 1..20 ⊂
    // 0..20 — rect containment is on BOTH axes; b's x range (5..105)
    // exceeds a's (0..100) so neither contains the other).
    let a = widget(
        1,
        1,
        Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
        Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
    );
    let b = widget(
        2,
        1,
        Rect::from_min_size(Pos2::new(5.0, 1.0), egui::vec2(100.0, 19.0)),
        Rect::from_min_size(Pos2::new(5.0, 1.0), egui::vec2(100.0, 19.0)),
    );
    let report = run_audit(&ctx, &snapshot(vec![a, b]), None);
    let issues = issues_for(&report, "overlap.suspicious");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].severity, Severity::Warning);
    assert_eq!(issues[0].other_id, Some(Id::new(1u64)));
    assert!(issues[0].evidence.contains('%'));
}

#[test]
fn overlap_containment_is_layout_not_bug() {
    let ctx = egui::Context::default();
    // Checkbox inside a row's hitbox → containment → evaluated, no issue.
    let row = widget(
        1,
        1,
        Rect::from_min_size(Pos2::ZERO, egui::vec2(200.0, 30.0)),
        Rect::from_min_size(Pos2::ZERO, egui::vec2(200.0, 30.0)),
    );
    let cb = widget(
        2,
        1,
        Rect::from_min_size(Pos2::new(8.0, 8.0), egui::vec2(14.0, 14.0)),
        Rect::from_min_size(Pos2::new(8.0, 8.0), egui::vec2(14.0, 14.0)),
    );
    let report = run_audit(&ctx, &snapshot(vec![row, cb]), None);
    assert!(issues_for(&report, "overlap.suspicious").is_empty());
    assert_eq!(rule_stats(&report, "overlap.suspicious").evaluated, 1);
}

#[test]
fn overlap_different_areas_skip_as_intentional() {
    let ctx = egui::Context::default();
    // Same Order::Middle but different layer ids = two Areas (floating
    // palette over canvas) — intentional overlay, skip.
    let a = widget(
        1,
        1,
        Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 30.0)),
        Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 30.0)),
    );
    let b = widget(
        2,
        2,
        Rect::from_min_size(Pos2::new(10.0, 5.0), egui::vec2(80.0, 20.0)),
        Rect::from_min_size(Pos2::new(10.0, 5.0), egui::vec2(80.0, 20.0)),
    );
    let report = run_audit(&ctx, &snapshot(vec![a, b]), None);
    assert!(issues_for(&report, "overlap.suspicious").is_empty());
    let stats = rule_stats(&report, "overlap.suspicious");
    assert_eq!(stats.skipped, 1);
    assert_eq!(stats.skip_reason, Some(SkipReason::IntentionalOverlay));
}

#[test]
fn overlap_small_overlap_does_not_flag() {
    let ctx = egui::Context::default();
    // ~10% edge overlap — dense toolbar adjacency, below threshold.
    let a = widget(
        1,
        1,
        Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
        Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
    );
    let b = widget(
        2,
        1,
        Rect::from_min_size(Pos2::new(90.0, 0.0), egui::vec2(50.0, 20.0)),
        Rect::from_min_size(Pos2::new(90.0, 0.0), egui::vec2(50.0, 20.0)),
    );
    let report = run_audit(&ctx, &snapshot(vec![a, b]), None);
    assert!(issues_for(&report, "overlap.suspicious").is_empty());
}

#[test]
fn overlap_disabled_widget_not_flagged() {
    let ctx = egui::Context::default();
    let a = widget(
        1,
        1,
        Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
        Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
    );
    let mut b = widget(
        2,
        1,
        Rect::from_min_size(Pos2::new(50.0, 5.0), egui::vec2(100.0, 20.0)),
        Rect::from_min_size(Pos2::new(50.0, 5.0), egui::vec2(100.0, 20.0)),
    );
    b.enabled = false;
    let report = run_audit(&ctx, &snapshot(vec![a, b]), None);
    assert!(issues_for(&report, "overlap.suspicious").is_empty());
}

#[test]
fn empty_snapshot_is_valid_and_clean() {
    let ctx = egui::Context::default();
    let report = run_audit(&ctx, &GeometrySnapshot::default(), None);
    assert!(report.issues.is_empty());
    assert_eq!(report.widget_count, 0);
    assert_eq!(report.rules.len(), RULES.len());
}

// ── semantic.missing_name ───────────────────────────────────────

fn sem_tree(
    nodes: Vec<(egui::accesskit::NodeId, egui::accesskit::Node)>,
    focus: egui::accesskit::NodeId,
) -> SemanticSnapshot {
    use egui::accesskit::Tree;
    let root = egui::accesskit_root_id().accesskit_id();
    SemanticSnapshot::from_update(&egui::accesskit::TreeUpdate {
        nodes,
        tree: Some(Tree::new(root)),
        focus,
        tree_id: egui::accesskit::TreeId::ROOT,
    })
    .expect("fixture tree")
}

#[test]
fn missing_name_flags_unlabelled_button() {
    use egui::accesskit::{Action, Node, Role};
    let ctx = egui::Context::default();
    let nid = egui::Id::new("b").accesskit_id();
    let root = egui::accesskit_root_id().accesskit_id();
    let mut b = Node::new(Role::Button);
    b.add_action(Action::Click);
    b.add_action(Action::Focus);
    let mut r = Node::new(Role::Window);
    r.push_child(nid);
    let sem = sem_tree(vec![(root, r), (nid, b)], root);
    let report = run_audit(&ctx, &GeometrySnapshot::default(), Some(&sem));
    let issues = issues_for(&report, "semantic.missing_name");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].severity, Severity::Error);
    assert_eq!(issues[0].widget_id, egui::Id::new("b"));
}

#[test]
fn missing_name_passes_labelled_and_skips_hidden() {
    use egui::accesskit::{Action, Node, Role};
    let ctx = egui::Context::default();
    let labelled = egui::Id::new("ok").accesskit_id();
    let hidden = egui::Id::new("hid").accesskit_id();
    let root = egui::accesskit_root_id().accesskit_id();
    let mut b = Node::new(Role::Button);
    b.set_label("OK");
    b.add_action(Action::Click);
    let mut h = Node::new(Role::Button); // unnamed but hidden → SKIP
    h.add_action(Action::Click);
    h.set_hidden();
    let mut r = Node::new(Role::Window);
    r.push_child(labelled);
    r.push_child(hidden);
    let sem = sem_tree(vec![(root, r), (labelled, b), (hidden, h)], root);
    let report = run_audit(&ctx, &GeometrySnapshot::default(), Some(&sem));
    assert!(issues_for(&report, "semantic.missing_name").is_empty());
    let stats = rule_stats(&report, "semantic.missing_name");
    assert_eq!(stats.evaluated, 1); // only the labelled, visible button
    assert_eq!(stats.skipped, 1); // hidden node skipped
}

#[test]
fn missing_name_joins_clickable_unknown_role_with_widget() {
    use egui::accesskit::{Action, Node, Role};
    let ctx = egui::Context::default();
    // Unknown-role node advertising Click + widget that senses clicks →
    // flags; the same node without a clickable widget counterpart → skip.
    let w = widget(
        1,
        1,
        Rect::from_min_size(Pos2::ZERO, egui::vec2(30.0, 20.0)),
        Rect::from_min_size(Pos2::ZERO, egui::vec2(30.0, 20.0)),
    );
    let nid = w.id.accesskit_id();
    let root = egui::accesskit_root_id().accesskit_id();
    let mut n = Node::new(Role::Unknown);
    n.add_action(Action::Click);
    let mut r = Node::new(Role::Window);
    r.push_child(nid);
    let sem = sem_tree(vec![(root, r), (nid, n)], root);

    let report = run_audit(&ctx, &snapshot(vec![w.clone()]), Some(&sem));
    assert_eq!(
        issues_for(&report, "semantic.missing_name").len(),
        1,
        "clickable unnamed node must flag"
    );

    // Same node, no widget counterpart (or non-clicking widget) → not evaluated.
    let mut quiet = widget(
        2,
        1,
        Rect::from_min_size(Pos2::ZERO, egui::vec2(30.0, 20.0)),
        Rect::from_min_size(Pos2::ZERO, egui::vec2(30.0, 20.0)),
    );
    quiet.senses_click = false;
    quiet.senses_drag = false;
    quiet.focusable = false;
    quiet.id = w.id; // same egui id, but the widget doesn't sense clicks
    quiet.id_debug = w.id_debug.clone();
    let report = run_audit(&ctx, &snapshot(vec![quiet]), Some(&sem));
    assert!(
        issues_for(&report, "semantic.missing_name").is_empty(),
        "non-clicking widget must not flag"
    );
}

#[test]
fn missing_name_ignores_label_role_and_skips_without_tree() {
    let ctx = egui::Context::default();
    // No semantic data → the rule reports SKIP, never invents a verdict.
    let report = run_audit(&ctx, &GeometrySnapshot::default(), None);
    let stats = rule_stats(&report, "semantic.missing_name");
    assert_eq!(stats.evaluated, 0);
    assert_eq!(stats.skipped, 1);
    assert_eq!(stats.skip_reason, Some(SkipReason::NoSemanticTree));
    assert!(issues_for(&report, "semantic.missing_name").is_empty());
}

// ── semantic.disabled_mismatch ──────────────────────────────────

#[test]
fn disabled_mismatch_flags_and_skips_cleanly() {
    use egui::accesskit::{Action, Node, Role};
    let ctx = egui::Context::default();
    // Widget enabled, but its semantic node says disabled → mismatch.
    let w = widget(
        1,
        1,
        Rect::from_min_size(Pos2::ZERO, egui::vec2(50.0, 20.0)),
        Rect::from_min_size(Pos2::ZERO, egui::vec2(50.0, 20.0)),
    );
    let mut node = Node::new(Role::Button);
    node.set_label("X");
    node.set_disabled();
    node.add_action(Action::Click);
    let nid = w.id.accesskit_id();
    let root = egui::accesskit_root_id().accesskit_id();
    let mut r = Node::new(Role::Window);
    r.push_child(nid);
    let sem = sem_tree(vec![(root, r), (nid, node)], root);
    let report = run_audit(&ctx, &snapshot(vec![w]), Some(&sem));
    let issues = issues_for(&report, "semantic.disabled_mismatch");
    assert_eq!(issues.len(), 1);
    assert!(issues[0].evidence.contains("enabled=true"));
}

#[test]
fn disabled_mismatch_skips_widget_without_node() {
    let ctx = egui::Context::default();
    // Interactive widget with no semantic node at all → SKIP, not a flag.
    let w = widget(
        1,
        1,
        Rect::from_min_size(Pos2::ZERO, egui::vec2(50.0, 20.0)),
        Rect::from_min_size(Pos2::ZERO, egui::vec2(50.0, 20.0)),
    );
    let root = egui::accesskit_root_id().accesskit_id();
    let sem = sem_tree(
        vec![(root, egui::accesskit::Node::new(egui::accesskit::Role::Window))],
        root,
    );
    let report = run_audit(&ctx, &snapshot(vec![w]), Some(&sem));
    assert!(issues_for(&report, "semantic.disabled_mismatch").is_empty());
    assert_eq!(rule_stats(&report, "semantic.disabled_mismatch").skipped, 1);
}

// ── reliability: verdicts, coverage, baseline diff ─────────────

fn report_with(stats: RuleStats, n_issues: usize) -> RuleReport {
    RuleReport {
        rule_id: "test",
        stats,
        issues: n_issues,
    }
}

#[test]
fn verdict_never_passes_on_zero_evaluated() {
    // Evaluated something clean → Pass.
    assert_eq!(
        report_with(
            RuleStats {
                evaluated: 3,
                ..Default::default()
            },
            0
        )
        .verdict(),
        Verdict::Pass
    );
    // All skips, nothing evaluated → Skipped (not Pass).
    assert_eq!(
        report_with(
            RuleStats {
                skipped: 2,
                skip_reason: Some(SkipReason::Clipped),
                ..Default::default()
            },
            0
        )
        .verdict(),
        Verdict::Skipped
    );
    // Nothing in scope at all → NotApplicable.
    assert_eq!(report_with(RuleStats::default(), 0).verdict(), Verdict::NotApplicable);
    // Issues → Fail.
    assert_eq!(
        report_with(
            RuleStats {
                evaluated: 3,
                ..Default::default()
            },
            2
        )
        .verdict(),
        Verdict::Fail
    );
}

#[test]
fn diff_compares_identity_not_counts() {
    let ctx = egui::Context::default();
    let mk = |id: u64| {
        let mut w = widget(id, 1, Rect::ZERO, Rect::ZERO);
        w.clipped = false;
        w
    };
    let snap = snapshot(vec![mk(10)]);
    let report_a = run_audit(&ctx, &snap, None);
    let report_b = run_audit(&ctx, &snap, None);
    // Equivalent snapshots, identical issues → empty diff.
    assert!(report_b.equivalent_snapshot(&report_a));
    let (new, resolved) = report_b.diff(&report_a).unwrap();
    assert!(new.is_empty() && resolved.is_empty());

    // A different widget set → not equivalent → no diff at all.
    let other = snapshot(vec![mk(20)]);
    let report_c = run_audit(&ctx, &other, None);
    assert!(!report_c.equivalent_snapshot(&report_a));
    assert!(report_c.diff(&report_a).is_none());
}

#[test]
fn diff_reports_new_and_resolved_by_identity() {
    let ctx = egui::Context::default();
    let bad = |id: u64| {
        let mut w = widget(id, 1, Rect::ZERO, Rect::ZERO);
        w.clipped = false;
        w
    };
    let ok = |id: u64| {
        widget(
            id,
            1,
            Rect::from_min_size(Pos2::ZERO, egui::vec2(30.0, 20.0)),
            Rect::from_min_size(Pos2::ZERO, egui::vec2(30.0, 20.0)),
        )
    };
    // Frame A: widgets 1(bad) + 2(ok). Frame B: same set, but 1 fixed and
    // 2 broken — same *count*, different identities.
    let a = run_audit(&ctx, &snapshot(vec![bad(1), ok(2)]), None);
    let b = run_audit(&ctx, &snapshot(vec![ok(1), bad(2)]), None);
    assert!(a.equivalent_snapshot(&b));
    let (new, resolved) = b.diff(&a).unwrap();
    assert_eq!(new.len(), 1, "widget 2's new issue must appear");
    assert_eq!(resolved.len(), 1, "widget 1's fixed issue must resolve");
    assert!(new[0].contains(&egui::Id::new(2u64).short_debug_format()));
}

// ── visual.typography_hierarchy ─────────────────────────────────

#[test]
fn typography_flags_heading_smaller_than_body() {
    let ctx = egui::Context::default();
    ctx.global_style_mut(|style| {
        style
            .text_styles
            .insert(egui::TextStyle::Heading, egui::FontId::proportional(10.0));
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(14.0));
    });
    let report = run_audit(&ctx, &GeometrySnapshot::default(), None);
    let issues = issues_for(&report, "visual.typography_hierarchy");
    assert_eq!(issues.len(), 1);
    assert!(issues[0].evidence.contains("Heading"));
}

#[test]
fn typography_passes_egui_defaults() {
    let ctx = egui::Context::default();
    let report = run_audit(&ctx, &GeometrySnapshot::default(), None);
    assert!(issues_for(&report, "visual.typography_hierarchy").is_empty());
    assert!(rule_stats(&report, "visual.typography_hierarchy").evaluated > 0);
}

// ── visual.text_contrast ────────────────────────────────────────

#[test]
fn contrast_flags_same_fg_bg_and_skips_transparent() {
    let ctx = egui::Context::default();
    ctx.global_style_mut(|style| {
        style.visuals.widgets.inactive.fg_stroke.color = egui::Color32::DARK_GRAY;
        style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::DARK_GRAY;
        // A non-opaque bg makes the pair undecidable → skip, not a guess.
        style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::TRANSPARENT;
    });
    let report = run_audit(&ctx, &GeometrySnapshot::default(), None);
    let issues = issues_for(&report, "visual.text_contrast");
    assert_eq!(issues.len(), 1);
    assert!(issues[0].evidence.contains("buttons at rest"));
    let stats = rule_stats(&report, "visual.text_contrast");
    assert_eq!(stats.skipped, 1);
    assert_eq!(stats.skip_reason, Some(SkipReason::NonOpaqueColors));
}

#[test]
fn contrast_passes_egui_default_visuals() {
    let ctx = egui::Context::default();
    let report = run_audit(&ctx, &GeometrySnapshot::default(), None);
    assert!(issues_for(&report, "visual.text_contrast").is_empty());
}

#[test]
fn contrast_ratio_is_wcag() {
    // Black on white is the WCAG reference point: 21:1.
    let r = contrast_ratio(egui::Color32::BLACK, egui::Color32::WHITE).unwrap();
    assert!((r - 21.0).abs() < 0.1, "expected ~21:1, got {r}");
    assert!(contrast_ratio(egui::Color32::WHITE, egui::Color32::TRANSPARENT).is_none());
}

// ── visual.touching_controls ────────────────────────────────────

#[test]
fn touching_flags_gapless_stacked_siblings() {
    let ctx = egui::Context::default();
    let a = widget(
        1,
        1,
        Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
        Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
    );
    let b = widget(
        2,
        1,
        Rect::from_min_size(Pos2::new(0.0, 20.0), egui::vec2(100.0, 20.0)),
        Rect::from_min_size(Pos2::new(0.0, 20.0), egui::vec2(100.0, 20.0)),
    );
    let report = run_audit(&ctx, &snapshot(vec![a, b]), None);
    let issues = issues_for(&report, "visual.touching_controls");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].confidence, Confidence::Medium);
}

#[test]
fn touching_ignores_normal_gaps_and_rows() {
    let ctx = egui::Context::default();
    // 8px gap — fine.
    let a = widget(
        1,
        1,
        Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
        Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
    );
    let b = widget(
        2,
        1,
        Rect::from_min_size(Pos2::new(0.0, 28.0), egui::vec2(100.0, 20.0)),
        Rect::from_min_size(Pos2::new(0.0, 28.0), egui::vec2(100.0, 20.0)),
    );
    // Side-by-side on one row — not a stack, not evaluated.
    let c = widget(
        3,
        1,
        Rect::from_min_size(Pos2::new(120.0, 0.0), egui::vec2(50.0, 20.0)),
        Rect::from_min_size(Pos2::new(120.0, 0.0), egui::vec2(50.0, 20.0)),
    );
    let report = run_audit(&ctx, &snapshot(vec![a, b, c]), None);
    assert!(issues_for(&report, "visual.touching_controls").is_empty());
}

#[test]
fn touching_skips_foreground_layers() {
    let ctx = egui::Context::default();
    // Menu/popup rows live on Foreground — gapless stacking is the pattern.
    let mut a = widget(
        1,
        1,
        Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
        Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
    );
    let mut b = widget(
        2,
        1,
        Rect::from_min_size(Pos2::new(0.0, 20.0), egui::vec2(100.0, 20.0)),
        Rect::from_min_size(Pos2::new(0.0, 20.0), egui::vec2(100.0, 20.0)),
    );
    a.layer.order = egui::Order::Foreground;
    b.layer.order = egui::Order::Foreground;
    let report = run_audit(&ctx, &snapshot(vec![a, b]), None);
    assert!(issues_for(&report, "visual.touching_controls").is_empty());
    let stats = rule_stats(&report, "visual.touching_controls");
    assert_eq!(stats.skipped, 1);
    assert_eq!(stats.skip_reason, Some(SkipReason::IntentionalOverlay));
}

// ── visual.alignment_drift ──────────────────────────────────────

#[test]
fn drift_flags_off_by_few_px_sibling() {
    let ctx = egui::Context::default();
    // Column of 4, one row 2px off the shared left edge → drift.
    let mk = |id, y, x| {
        widget(
            id,
            1,
            Rect::from_min_size(Pos2::new(x, y), egui::vec2(100.0, 20.0)),
            Rect::from_min_size(Pos2::new(x, y), egui::vec2(100.0, 20.0)),
        )
    };
    let widgets = vec![
        mk(1, 0.0, 10.0),
        mk(2, 30.0, 10.0),
        mk(3, 60.0, 10.0),
        mk(4, 90.0, 12.0),
    ];
    let report = run_audit(&ctx, &snapshot(widgets), None);
    let issues = issues_for(&report, "visual.alignment_drift");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].widget_id, egui::Id::new(4u64));
    assert!(issues[0].evidence.contains("2.0px"));
}

#[test]
fn drift_ignores_aligned_columns_and_real_indents() {
    let ctx = egui::Context::default();
    let mk = |id, y, x| {
        widget(
            id,
            1,
            Rect::from_min_size(Pos2::new(x, y), egui::vec2(100.0, 20.0)),
            Rect::from_min_size(Pos2::new(x, y), egui::vec2(100.0, 20.0)),
        )
    };
    // Aligned column.
    let aligned = vec![mk(1, 0.0, 10.0), mk(2, 30.0, 10.0), mk(3, 60.0, 10.0)];
    let report = run_audit(&ctx, &snapshot(aligned), None);
    assert!(issues_for(&report, "visual.alignment_drift").is_empty());
    // 20px offset = deliberate indent, outside the drift band.
    let indented = vec![
        mk(1, 0.0, 10.0),
        mk(2, 30.0, 10.0),
        mk(3, 60.0, 10.0),
        mk(4, 90.0, 30.0),
    ];
    let report = run_audit(&ctx, &snapshot(indented), None);
    assert!(issues_for(&report, "visual.alignment_drift").is_empty());
    // Two members only — no majority edge exists → skip whole group.
    let pair = vec![mk(1, 0.0, 10.0), mk(2, 30.0, 12.0)];
    let report = run_audit(&ctx, &snapshot(pair), None);
    assert!(issues_for(&report, "visual.alignment_drift").is_empty());
}

// ── semantic.orphan_node ────────────────────────────────────────

#[test]
fn orphan_node_flags_parentless_node() {
    use egui::accesskit::{Node, Role};
    let ctx = egui::Context::default();
    let root = egui::accesskit_root_id().accesskit_id();
    // Node emitted but never listed as anyone's child → orphan.
    let stray = egui::Id::new("stray").accesskit_id();
    let mut n = Node::new(Role::Button);
    n.set_label("Stray");
    let sem = sem_tree(vec![(root, Node::new(Role::Window)), (stray, n)], root);
    let report = run_audit(&ctx, &GeometrySnapshot::default(), Some(&sem));
    let issues = issues_for(&report, "semantic.orphan_node");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].widget_id, egui::Id::new("stray"));
}

#[test]
fn orphan_node_passes_root_and_children() {
    use egui::accesskit::{Node, Role};
    let ctx = egui::Context::default();
    let root = egui::accesskit_root_id().accesskit_id();
    let child = egui::Id::new("c").accesskit_id();
    let mut r = Node::new(Role::Window);
    r.push_child(child);
    let mut c = Node::new(Role::Button);
    c.set_label("C");
    let sem = sem_tree(vec![(root, r), (child, c)], root);
    let report = run_audit(&ctx, &GeometrySnapshot::default(), Some(&sem));
    assert!(issues_for(&report, "semantic.orphan_node").is_empty());
}

// ── focus.unfocusable ───────────────────────────────────────────

#[test]
fn focus_unfocusable_flags_focus_on_non_focusable_node() {
    use egui::accesskit::{Node, Role};
    let ctx = egui::Context::default();
    let root = egui::accesskit_root_id().accesskit_id();
    let id = egui::Id::new("lbl").accesskit_id();
    let mut r = Node::new(Role::Window);
    r.push_child(id);
    let mut n = Node::new(Role::Label); // no Focus action
    n.set_value("some label");
    let sem = sem_tree(vec![(root, r), (id, n)], id);
    let report = run_audit(&ctx, &GeometrySnapshot::default(), Some(&sem));
    let issues = issues_for(&report, "focus.unfocusable");
    assert_eq!(issues.len(), 1);
    assert!(issues[0].evidence.contains("without Action::Focus"));
}

#[test]
fn focus_unfocusable_passes_on_root_and_focusable() {
    use egui::accesskit::{Action, Node, Role};
    let ctx = egui::Context::default();
    let root = egui::accesskit_root_id().accesskit_id();
    let id = egui::Id::new("b").accesskit_id();
    let mut r = Node::new(Role::Window);
    r.push_child(id);
    let mut n = Node::new(Role::Button);
    n.set_label("B");
    n.add_action(Action::Focus);
    // Case 1: focus on the focusable button.
    let sem = sem_tree(vec![(root, r.clone()), (id, n)], id);
    let report = run_audit(&ctx, &GeometrySnapshot::default(), Some(&sem));
    assert!(issues_for(&report, "focus.unfocusable").is_empty());
    // Case 2: focus on the root = "nothing focused" — not a violation.
    let mut r2 = Node::new(Role::Window);
    r2.push_child(id);
    let sem2 = sem_tree(vec![(root, r2), (id, Node::new(Role::Button))], root);
    let report = run_audit(&ctx, &GeometrySnapshot::default(), Some(&sem2));
    assert!(issues_for(&report, "focus.unfocusable").is_empty());
}

// ── focus.orphaned ──────────────────────────────────────────────

#[test]
fn focus_orphaned_flags_dangling_focus() {
    use egui::accesskit::{Node, Role};
    let ctx = egui::Context::default();
    let root = egui::accesskit_root_id().accesskit_id();
    // focus points at a node that was never emitted.
    let sem = sem_tree(vec![(root, Node::new(Role::Window))], egui::accesskit::NodeId(424242));
    let report = run_audit(&ctx, &GeometrySnapshot::default(), Some(&sem));
    let issues = issues_for(&report, "focus.orphaned");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].severity, Severity::Error);
    assert!(issues[0].evidence.contains("424242"));
}

#[test]
fn focus_orphaned_passes_on_root_focus() {
    use egui::accesskit::{Node, Role};
    let ctx = egui::Context::default();
    let root = egui::accesskit_root_id().accesskit_id();
    let sem = sem_tree(vec![(root, Node::new(Role::Window))], root);
    let report = run_audit(&ctx, &GeometrySnapshot::default(), Some(&sem));
    assert!(issues_for(&report, "focus.orphaned").is_empty());
}

#[test]
fn issue_id_resolves_then_survives_widget_gone() {
    // Issue → snapshot.find round-trips while the widget lives; a stale
    // id after the widget vanishes resolves to None, not a panic.
    let ctx = egui::Context::default();
    let mut w = widget(7, 1, Rect::from_min_size(Pos2::ZERO, egui::vec2(0.0, 0.0)), Rect::ZERO);
    w.clipped = false;
    let snap = snapshot(vec![w]);
    let report = run_audit(&ctx, &snap, None);
    let issue = report
        .issues
        .iter()
        .find(|i| i.rule_id == "interactive.zero_size")
        .unwrap();
    assert!(snap.find(issue.widget_id).is_some(), "issue must resolve to its widget");
    let empty = GeometrySnapshot::default();
    assert!(empty.find(issue.widget_id).is_none());
}
