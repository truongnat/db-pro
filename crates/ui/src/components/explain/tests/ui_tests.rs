//! Tests for `../ui.rs` (moved out of the source file).
use super::*;

#[test]
fn explain_tree_truncates_manually_built_deep_plans() {
    let mut root = PlanNode::new("Seq Scan", 1.0, 1.0, 1);
    for _ in 0..db_pro_core::domain::explain_plan::MAX_EXPLAIN_PLAN_DEPTH {
        root = PlanNode::new("Nested Loop", 1.0, 1.0, 1).with_child(root);
    }
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);

    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ExplainPlanTree::new(&root, 1.0, DbProTheme::light()).show(ui);
        });
    });
}

#[test]
fn explain_tree_renders_runtime_badges_without_panicking() {
    let theme = DbProTheme::light();
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let child = PlanNode::new("Seq Scan", 30.0, 1.0, 10)
        .relation("users")
        .bottleneck(true);
    let root = PlanNode::new("Nested Loop", 40.0, 2.0, 10)
        .with_child(child.clone())
        .with_child(child);

    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let response = ExplainPlanTree::new(&root, 2.0, theme).has_runtime_stats(true).show(ui);
            assert!(response.rect.width() > 0.0);
            assert!(response.rect.height() > 0.0);
        });
    });
}
