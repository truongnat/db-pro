use eframe::egui;
use rs_ui_core::{Point, Rect};
use rs_ui_runtime::{
    AccessibilityRole, AccessibilitySemantics, BehaviorCommand, HitTestState, LayoutStyle, NodeId, PaintState,
    PointerEvents, RuntimeError, UiTree,
};
use std::{collections::HashMap, collections::HashSet, time::Duration};

#[derive(Clone, Copy)]
pub(crate) struct ExplorerTreeItem<'a> {
    pub(crate) key: &'a str,
    pub(crate) parent_key: Option<&'a str>,
    pub(crate) label: &'a str,
    pub(crate) expanded: bool,
    pub(crate) selected: bool,
    pub(crate) bounds: egui::Rect,
    pub(crate) focused: bool,
    pub(crate) clicked: bool,
}

pub(crate) struct ExplorerTreeRuntime {
    tree: UiTree,
    root: Option<NodeId>,
    nodes: HashMap<String, NodeId>,
    parents: HashMap<String, Option<String>>,
    active_keys: HashSet<String>,
    focused_item_seen: bool,
    pending_navigation: Option<BehaviorCommand>,
}

impl Default for ExplorerTreeRuntime {
    fn default() -> Self {
        Self {
            tree: UiTree::new(),
            root: None,
            nodes: HashMap::new(),
            parents: HashMap::new(),
            active_keys: HashSet::new(),
            focused_item_seen: false,
            pending_navigation: None,
        }
    }
}

impl ExplorerTreeRuntime {
    pub(crate) fn begin(&mut self) {
        self.active_keys.clear();
        self.focused_item_seen = false;
    }

    pub(crate) fn queue_navigation(&mut self, command: BehaviorCommand) {
        if matches!(command, BehaviorCommand::MoveNext | BehaviorCommand::MovePrevious) {
            self.pending_navigation = Some(command);
        }
    }

    pub(crate) fn item_has_focus(&self, key: &str) -> bool {
        self.nodes
            .get(key)
            .is_some_and(|node| self.tree.focus_manager().focused() == Some(*node))
    }

    pub(crate) fn register_item(&mut self, item: ExplorerTreeItem<'_>) -> Result<bool, RuntimeError> {
        let root = self.ensure_root()?;
        let parent = match item.parent_key {
            Some(parent_key) => self
                .nodes
                .get(parent_key)
                .copied()
                .ok_or(RuntimeError::UnknownNode(root))?,
            None => root,
        };
        let node = self.ensure_node(parent, item)?;
        self.update_item_state(node, item)?;
        self.active_keys.insert(item.key.to_owned());
        self.parents
            .insert(item.key.to_owned(), item.parent_key.map(str::to_owned));
        if item.focused {
            self.focused_item_seen = true;
            let _ = self.tree.request_focus(node)?;
        }
        if !item.clicked {
            return Ok(false);
        }
        let _ = self.tree.request_focus(node)?;
        let event = self
            .tree
            .dispatch_behavior_command(root, BehaviorCommand::Activate, Duration::ZERO)?;
        Ok(event.default_prevented())
    }

    pub(crate) fn end(&mut self) -> Result<(), RuntimeError> {
        let root = self.ensure_root()?;
        let focused_item = self
            .tree
            .focus_manager()
            .focused()
            .is_some_and(|focused| self.nodes.values().any(|node| *node == focused));
        if focused_item && !self.focused_item_seen {
            self.tree.clear_focus();
        }
        self.remove_stale_subtrees()?;
        self.active_keys.clear();
        if let Some(command) = self.pending_navigation.take() {
            self.tree.dispatch_behavior_command(root, command, Duration::ZERO)?;
        }
        if self.tree.node(root).is_none() {
            return Err(RuntimeError::UnknownNode(root));
        }
        Ok(())
    }

    fn ensure_root(&mut self) -> Result<NodeId, RuntimeError> {
        if let Some(root) = self.root {
            return Ok(root);
        }
        let root = self
            .tree
            .create_node(None, LayoutStyle::default(), PaintState::default())?;
        self.tree
            .set_accessibility_semantics(root, AccessibilitySemantics::new(AccessibilityRole::Tree))?;
        self.root = Some(root);
        Ok(root)
    }

    fn ensure_node(&mut self, parent: NodeId, item: ExplorerTreeItem<'_>) -> Result<NodeId, RuntimeError> {
        if let Some(node) = self.nodes.get(item.key).copied() {
            return Ok(node);
        }
        let node = self
            .tree
            .create_node(Some(parent), LayoutStyle::default(), PaintState::default())?;
        self.tree.register_pressable(node, Some(item.label.to_owned()), false)?;
        self.nodes.insert(item.key.to_owned(), node);
        Ok(node)
    }

    fn update_item_state(&mut self, node: NodeId, item: ExplorerTreeItem<'_>) -> Result<(), RuntimeError> {
        let mut semantics = AccessibilitySemantics::new(AccessibilityRole::TreeItem);
        semantics.label = Some(item.label.to_owned());
        semantics.state.expanded = Some(item.expanded);
        self.tree.set_accessibility_semantics(node, semantics)?;
        self.tree.apply_selection_state(node, item.selected)?;
        self.tree.set_hit_test_state(
            node,
            HitTestState {
                bounds: Rect::from_min_max(
                    Point::new(item.bounds.min.x, item.bounds.min.y),
                    Point::new(item.bounds.max.x, item.bounds.max.y),
                ),
                pointer_events: PointerEvents::Auto,
                ..HitTestState::default()
            },
        )?;
        Ok(())
    }

    fn remove_stale_subtrees(&mut self) -> Result<(), RuntimeError> {
        let stale = self
            .nodes
            .keys()
            .filter(|key| !self.active_keys.contains(*key))
            .cloned()
            .collect::<HashSet<_>>();
        let stale_roots = stale
            .iter()
            .filter(|key| {
                self.parents
                    .get(*key)
                    .and_then(Option::as_ref)
                    .is_none_or(|parent| !stale.contains(parent))
            })
            .filter_map(|key| self.nodes.get(key).copied())
            .collect::<Vec<_>>();
        for node in stale_roots {
            self.tree.remove_subtree(node)?;
        }
        self.nodes.retain(|key, _| self.active_keys.contains(key));
        self.parents.retain(|key, _| self.active_keys.contains(key));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item<'a>(key: &'a str, parent_key: Option<&'a str>, focused: bool) -> ExplorerTreeItem<'a> {
        ExplorerTreeItem {
            key,
            parent_key,
            label: key,
            expanded: true,
            selected: false,
            bounds: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(160.0, 26.0)),
            focused,
            clicked: false,
        }
    }

    #[test]
    fn tree_items_keep_parentage_and_selected_expanded_state() {
        let mut runtime = ExplorerTreeRuntime::default();
        runtime.begin();
        runtime.register_item(item("connection", None, false)).unwrap();
        runtime
            .register_item(item("database", Some("connection"), false))
            .unwrap();
        let mut schema = item("schema", Some("database"), false);
        schema.selected = true;
        runtime.register_item(schema).unwrap();
        runtime.register_item(item("tables", Some("schema"), false)).unwrap();
        let mut table = item("table", Some("tables"), true);
        table.label = "users";
        table.expanded = false;
        table.selected = true;
        runtime.register_item(table).unwrap();

        let root = runtime.root.unwrap();
        let schema_node = runtime.nodes["schema"];
        let tables_node = runtime.nodes["tables"];
        let table_node = runtime.nodes["table"];
        let mut semantic_tree = rs_ui_runtime::SemanticTree::default();
        semantic_tree.update(&mut runtime.tree, root).unwrap();
        let schema_semantic = semantic_tree
            .nodes()
            .get(&semantic_tree.accessibility_id(schema_node).unwrap())
            .unwrap();
        let tables_semantic = semantic_tree
            .nodes()
            .get(&semantic_tree.accessibility_id(tables_node).unwrap())
            .unwrap();
        let table_semantic = semantic_tree
            .nodes()
            .get(&semantic_tree.accessibility_id(table_node).unwrap())
            .unwrap();
        assert_eq!(schema_semantic.role, AccessibilityRole::TreeItem);
        assert_eq!(schema_semantic.state.selected, Some(true));
        assert!(schema_semantic
            .children
            .contains(&semantic_tree.accessibility_id(tables_node).unwrap()));
        assert!(tables_semantic
            .children
            .contains(&semantic_tree.accessibility_id(table_node).unwrap()));
        assert_eq!(table_semantic.label.as_deref(), Some("users"));
        assert_eq!(table_semantic.state.selected, Some(true));
        assert_eq!(table_semantic.state.focused, Some(true));
    }

    #[test]
    fn stale_descendants_are_removed_without_removing_live_parent() {
        let mut runtime = ExplorerTreeRuntime::default();
        runtime.begin();
        runtime.register_item(item("connection", None, false)).unwrap();
        runtime
            .register_item(item("database", Some("connection"), false))
            .unwrap();
        runtime.register_item(item("schema", Some("database"), false)).unwrap();
        runtime.end().unwrap();

        runtime.begin();
        runtime.register_item(item("connection", None, false)).unwrap();
        runtime.end().unwrap();

        assert_eq!(runtime.nodes.len(), 1);
        assert_eq!(runtime.tree.node_count(), 2);
    }

    #[test]
    fn navigation_uses_rendered_focus_order() {
        let mut runtime = ExplorerTreeRuntime::default();
        runtime.begin();
        runtime.register_item(item("first", None, true)).unwrap();
        runtime.register_item(item("second", None, false)).unwrap();
        runtime.queue_navigation(BehaviorCommand::MoveNext);
        runtime.end().unwrap();
        assert!(runtime.item_has_focus("second"));
    }
}
