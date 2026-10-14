//! Tests for `../handler.rs` (moved out of the source file).
use super::*;

#[test]
fn status_bar_item_builder_preserves_public_state() {
    let item = StatusBarItem::new("Ready")
        .icon(Icon::Check)
        .tooltip("All systems operational")
        .accent(true);

    assert_eq!(item.text, "Ready");
    assert!(matches!(item.icon, Some(Icon::Check)));
    assert_eq!(item.tooltip.as_deref(), Some("All systems operational"));
    assert!(item.is_accent);
}

#[test]
fn connection_health_labels_match_workspace_copy() {
    assert_eq!(connection_health_label(ConnectionHealth::Healthy), "Connected");
    assert_eq!(connection_health_label(ConnectionHealth::Degraded), "High Latency");
    assert_eq!(connection_health_label(ConnectionHealth::Disconnected), "Disconnected");
}

#[test]
fn workspace_geometry_preserves_status_and_activity_layout() {
    assert_eq!(status_item_width(24.0, false), 24.0);
    assert_eq!(status_item_width(24.0, true), 40.0);
    assert_eq!(next_status_item_cursor(10.0, 40.0), 10.0 + 40.0 + SPACE_MD);
    assert_eq!(right_status_item_start(100.0, 40.0), 60.0);
    assert_eq!(status_text_top(20.0, 10.0), 15.0);

    assert_eq!(right_status_block_width(&[]), 0.0);
    assert_eq!(right_status_block_width(&[40.0]), 40.0);
    assert_eq!(right_status_block_width(&[40.0, 20.0]), 60.0 + SPACE_MD);
    assert_eq!(left_status_items_limit(200.0, 0.0, false), 200.0);
    assert_eq!(left_status_items_limit(200.0, 80.0, true), 200.0 - 80.0 - SPACE_MD);

    let y = activity_item_start_y(3.0);
    let rect = activity_item_rect(10.0, y);
    assert_eq!(rect.min, Pos2::new(16.0, 3.0 + SPACE_MD));
    assert_eq!(rect.size(), Vec2::splat(36.0));
    assert_eq!(next_activity_item_y(y), y + ACTIVITY_ITEM_STEP);
}

#[test]
fn activity_bar_order_and_latency_threshold_remain_stable() {
    let kinds: Vec<ActivityBarItemKind> = super::super::config::ACTIVITY_BAR_ITEMS
        .iter()
        .map(|item| item.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            ActivityBarItemKind::Explorer,
            ActivityBarItemKind::QueryEditor,
            ActivityBarItemKind::Agent,
            ActivityBarItemKind::Diagram,
        ]
    );
    assert!(!is_latency_warning(200));
    assert!(is_latency_warning(201));
}
