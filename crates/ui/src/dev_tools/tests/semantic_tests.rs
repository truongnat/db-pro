//! Tests for `../semantic.rs` (moved out of the source file).
use egui::accesskit::{Node, Tree};

use super::*;

fn root_with(children: &[NodeId]) -> (NodeId, Node) {
    let root_id = egui::accesskit_root_id().accesskit_id();
    let mut root = Node::new(Role::Window);
    for c in children {
        root.push_child(*c);
    }
    (root_id, root)
}

#[test]
fn snapshot_maps_role_label_disabled_focus_parent() {
    let btn_id = Id::new("btn").accesskit_id();
    let (root_id, root) = root_with(&[btn_id]);
    let mut btn = Node::new(Role::Button);
    btn.set_label("Open");
    btn.set_disabled();
    btn.add_action(Action::Click);
    btn.add_action(Action::Focus);
    let update = accesskit::TreeUpdate {
        nodes: vec![(root_id, root), (btn_id, btn)],
        tree: Some(Tree::new(root_id)),
        focus: btn_id,
        tree_id: accesskit::TreeId::ROOT,
    };
    let snap = SemanticSnapshot::from_update(&update).expect("tree present");
    let node = snap.find_widget(Id::new("btn")).expect("node by egui id");
    assert_eq!(node.role, Role::Button);
    assert_eq!(node.name.as_deref(), Some("Open"));
    assert!(node.disabled);
    assert!(node.supports_click && node.supports_focus);
    assert_eq!(node.parent, Some(root_id));
    assert_eq!(snap.focus, Some(btn_id));
}

#[test]
fn empty_update_yields_none_not_empty_snapshot() {
    let update = accesskit::TreeUpdate {
        nodes: vec![],
        tree: None,
        focus: accesskit::NodeId(0),
        tree_id: accesskit::TreeId::ROOT,
    };
    assert!(SemanticSnapshot::from_update(&update).is_none());
}

#[test]
fn unlabelled_node_is_detectable() {
    let id = Id::new("x").accesskit_id();
    let (root_id, root) = root_with(&[id]);
    let node = Node::new(Role::Button); // no label
    let update = accesskit::TreeUpdate {
        nodes: vec![(root_id, root), (id, node)],
        tree: Some(Tree::new(root_id)),
        focus: root_id,
        tree_id: accesskit::TreeId::ROOT,
    };
    let snap = SemanticSnapshot::from_update(&update).unwrap();
    assert!(snap.find_widget(Id::new("x")).unwrap().name.is_none());
}
