use super::*;

#[test]
fn shell_regions_preserve_sidebar_and_split_remaining_space() {
    let regions = layout_shell(Size::new(1440.0, 900.0), 280.0).unwrap();

    assert_eq!(regions.sidebar.width(), 280.0);
    assert_eq!(regions.main.min.x, 280.0);
    assert_eq!(regions.main.width(), 1160.0);
    assert_eq!(regions.tabs.height(), TABS_HEIGHT);
    assert_eq!(regions.workspace.height(), 824.0);
}

#[test]
fn shell_layout_rejects_viewports_that_cannot_fit_the_main_surface() {
    assert!(layout_shell(Size::new(690.0, 700.0), 280.0).is_none());
    assert!(layout_shell(Size::new(1440.0, 70.0), 280.0).is_none());
}

#[test]
fn sidebar_resizer_uses_runtime_drag_delta_and_limits() {
    let mut runtime = RsUiShellRuntime::default();
    runtime
        .begin_sidebar_resize(280.0, 240.0, 360.0, rs_ui_core::Point::new(280.0, 30.0))
        .unwrap();

    assert_eq!(
        runtime
            .update_sidebar_resize(rs_ui_core::Point::new(330.0, 30.0))
            .unwrap(),
        330.0
    );
    assert_eq!(
        runtime
            .update_sidebar_resize(rs_ui_core::Point::new(400.0, 30.0))
            .unwrap(),
        360.0
    );
    runtime.end_sidebar_resize().unwrap();
}

#[test]
fn focused_sidebar_resizer_accepts_normalized_keyboard_commands() {
    let mut runtime = RsUiShellRuntime::default();

    assert_eq!(
        runtime
            .adjust_sidebar_width(280.0, 240.0, 360.0, BehaviorCommand::MoveLeft)
            .unwrap(),
        272.0
    );
}

#[test]
fn output_splitters_apply_resizable_behavior_in_database_dock_direction() {
    let mut runtime = RsUiShellRuntime::default();
    runtime
        .begin_output_resize(true, 480.0, 240.0, 1200.0, rs_ui_core::Point::new(100.0, 100.0))
        .unwrap();
    assert_eq!(
        runtime
            .update_output_resize(true, rs_ui_core::Point::new(80.0, 100.0))
            .unwrap(),
        500.0
    );
    runtime.end_output_resize(true).unwrap();

    runtime
        .begin_output_resize(false, 180.0, 120.0, 640.0, rs_ui_core::Point::new(100.0, 100.0))
        .unwrap();
    assert_eq!(
        runtime
            .update_output_resize(false, rs_ui_core::Point::new(100.0, 80.0))
            .unwrap(),
        200.0
    );
    runtime.end_output_resize(false).unwrap();
}

#[test]
fn sidebar_scroll_state_clamps_wheel_input_to_content_extent() {
    let mut runtime = RsUiShellRuntime::default();
    runtime
        .sync_sidebar_scroll(
            Size::new(200.0, 100.0),
            Size::new(200.0, 500.0),
            rs_ui_core::Point::ZERO,
        )
        .unwrap();

    assert_eq!(
        runtime
            .scroll_sidebar(Size::new(200.0, 100.0), rs_ui_core::Point::new(0.0, -80.0))
            .unwrap(),
        80.0
    );
    assert_eq!(
        runtime
            .scroll_sidebar(Size::new(200.0, 100.0), rs_ui_core::Point::new(0.0, 900.0))
            .unwrap(),
        0.0
    );
}

#[test]
fn workspace_tabs_register_selected_tab_semantics_and_normalized_activation() {
    let mut runtime = RsUiShellRuntime::default();
    runtime.begin_workspace_tabs();
    let bounds = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(120.0, 28.0));

    assert!(runtime
        .register_workspace_tab("query:doc-1", "Query 1", true, bounds, true, true)
        .unwrap());
    let root = runtime.tabs_root.unwrap();
    let node = runtime.tab_nodes["query:doc-1"];
    let mut semantic_tree = rs_ui_runtime::SemanticTree::default();
    semantic_tree.update(&mut runtime.tree, root).unwrap();
    let semantic_id = semantic_tree.accessibility_id(node).unwrap();
    let semantic = semantic_tree.nodes().get(&semantic_id).unwrap();

    assert_eq!(semantic.role, AccessibilityRole::Tab);
    assert_eq!(semantic.label.as_deref(), Some("Query 1"));
    assert_eq!(semantic.state.selected, Some(true));
    assert_eq!(semantic.state.focused, Some(true));
    assert_eq!(semantic.bounds.unwrap().width(), 120.0);
}

#[test]
fn workspace_tab_cleanup_removes_closed_tab_nodes() {
    let mut runtime = RsUiShellRuntime::default();
    let bounds = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(100.0, 28.0));
    runtime.begin_workspace_tabs();
    runtime
        .register_workspace_tab("query:closed", "Closed", false, bounds, false, false)
        .unwrap();
    runtime.end_workspace_tabs().unwrap();
    runtime.begin_workspace_tabs();
    runtime.end_workspace_tabs().unwrap();

    assert!(runtime.tab_nodes.is_empty());
    assert_eq!(runtime.tree.node_count(), 1);
}

#[test]
fn workspace_close_button_is_a_focused_pressable_child_of_its_tab() {
    let mut runtime = RsUiShellRuntime::default();
    let tab_bounds = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(120.0, 28.0));
    let close_bounds = egui::Rect::from_min_size(egui::pos2(100.0, 6.0), egui::vec2(16.0, 16.0));
    runtime.begin_workspace_tabs();
    runtime
        .register_workspace_tab("query:doc-1", "Query 1", true, tab_bounds, false, false)
        .unwrap();
    assert!(runtime
        .register_workspace_close_button(
            "query:doc-1",
            "close:query:doc-1",
            "Close Query 1",
            close_bounds,
            true,
            true,
        )
        .unwrap());

    let root = runtime.tabs_root.unwrap();
    let tab = runtime.tab_nodes["query:doc-1"];
    let close_button = runtime.tab_nodes["close:query:doc-1"];
    let mut semantic_tree = rs_ui_runtime::SemanticTree::default();
    semantic_tree.update(&mut runtime.tree, root).unwrap();
    let tab_id = semantic_tree.accessibility_id(tab).unwrap();
    let close_id = semantic_tree.accessibility_id(close_button).unwrap();
    let tab_semantic = semantic_tree.nodes().get(&tab_id).unwrap();
    let close_semantic = semantic_tree.nodes().get(&close_id).unwrap();

    assert!(tab_semantic.children.contains(&close_id));
    assert_eq!(close_semantic.role, AccessibilityRole::Button);
    assert_eq!(close_semantic.label.as_deref(), Some("Close Query 1"));
    assert_eq!(close_semantic.state.focused, Some(true));
}
