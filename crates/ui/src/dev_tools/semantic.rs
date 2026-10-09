// cc-scan:allow LONG_FUNCTION — linear mapping + linear rule bodies.
//! AccessKit semantic-tree capture for the UI inspector.
//!
//! egui 0.36 emits an [`accesskit::TreeUpdate`] into
//! `PlatformOutput::accesskit_update` at end of pass **only while accesskit
//! is enabled** (`Context::enable_accesskit`, sticky). [`AccessKitCapture`]
//! is an egui [`Plugin`] whose `output_hook` clones that update each frame so
//! the inspector can audit role/label/disabled/parent-child/focus without
//! touching eframe's output pipeline.
//!
//! Semantic nodes are *not* widget geometry: a `WidgetRect` and a semantic
//! node share the same [`egui::Id`] (egui builds node ids via
//! `Id::accesskit_id`), which is the only join between the two snapshots.
//! Rules never infer parents or focus — a missing `TreeUpdate` or absent
//! parent entry means the rule skips, it doesn't guess.

use std::collections::HashMap;
use std::sync::Mutex;

use egui::accesskit::{self, Action, NodeId, Role};
use egui::Id;

/// egui [`Plugin`] that clones each pass's AccessKit [`accesskit::TreeUpdate`]
/// into a slot the inspector reads on the next frame. Registered lazily the
/// first time the inspector opens; clone (not take) keeps real assistive-tech
/// delivery intact.
#[derive(Default)]
pub struct AccessKitCapture {
    latest: Mutex<Option<accesskit::TreeUpdate>>,
}

impl egui::plugin::Plugin for AccessKitCapture {
    fn debug_name(&self) -> &'static str {
        "db-pro accesskit capture"
    }

    fn output_hook(&mut self, _ctx: &egui::Context, output: &mut egui::FullOutput) {
        if let Some(update) = output.platform_output.accesskit_update.clone() {
            // Poisoned mutex → drop this frame's update; rules will skip on
            // missing data rather than guess.
            if let Ok(mut latest) = self.latest.lock() {
                *latest = Some(update);
            }
        }
    }
}

impl AccessKitCapture {
    /// Ensure the plugin is registered and accesskit is generating updates.
    /// Both are idempotent (`add_plugin` dedupes by type; `enable_accesskit`
    /// just sets a flag).
    pub fn ensure_installed(ctx: &egui::Context) {
        ctx.enable_accesskit();
        ctx.add_plugin(Self::default());
    }

    /// The most recent captured tree, if the plugin was registered and at
    /// least one accesskit-enabled pass has completed.
    pub fn latest(ctx: &egui::Context) -> Option<accesskit::TreeUpdate> {
        ctx.with_plugin(|p: &mut Self| p.latest.lock().ok().and_then(|g| g.clone()))
            .flatten()
    }
}

/// One node from the semantic tree, flattened to owned data.
#[derive(Clone, Debug)]
pub struct SemanticNode {
    /// The `egui::Id` this node was built from (inverse of `Id::accesskit_id`).
    /// `None` for the root window node, which has no widget counterpart.
    pub widget_id: Option<Id>,
    pub node_id: NodeId,
    pub role: Role,
    /// `Role` debug string for display.
    pub role_debug: String,
    /// Accessible name: `label`, `value` (labels store text in `value`), or
    /// resolved through `labelled_by`. Empty if the node is unnamed.
    pub name: Option<String>,
    pub disabled: bool,
    pub hidden: bool,
    pub supports_click: bool,
    pub supports_focus: bool,
    /// Parent in the semantic tree, when the update lists this node as a
    /// child of another node.
    pub parent: Option<NodeId>,
}

/// Frozen view of one `TreeUpdate`.
#[derive(Clone, Debug, Default)]
pub struct SemanticSnapshot {
    /// All nodes present in the update, keyed by `NodeId`. `TreeUpdate` is
    /// incremental — nodes not re-emitted keep prior state, so this merge is
    /// the best available view; absent nodes simply mean "no data".
    pub nodes: HashMap<NodeId, SemanticNode>,
    /// Node egui says has keyboard focus (or the tree root when none does).
    pub focus: Option<NodeId>,
}

impl SemanticSnapshot {
    /// Flatten a `TreeUpdate` into inspectable nodes.
    /// Returns `None` when the update carries no tree/nodes — i.e. there is
    /// nothing valid to audit.
    pub fn from_update(update: &accesskit::TreeUpdate) -> Option<Self> {
        if update.nodes.is_empty() {
            return None;
        }
        let mut nodes: HashMap<NodeId, SemanticNode> = HashMap::new();
        // Parent map from children lists — the only structural truth in the
        // update; `Node` has no `parent` property.
        let mut parent_of: HashMap<NodeId, NodeId> = HashMap::new();
        for (id, node) in &update.nodes {
            for child in node.children() {
                parent_of.insert(*child, *id);
            }
        }
        let labelled_by: HashMap<NodeId, Vec<NodeId>> = update
            .nodes
            .iter()
            .map(|(id, node)| (*id, node.labelled_by().to_vec()))
            .collect();
        for (node_id, node) in &update.nodes {
            // Resolve the accessible name: direct label → labelled_by targets
            // → `value` (Label-role text lives there).
            let mut name = node.label().map(str::to_owned);
            if name.is_none() {
                for src in labelled_by.get(node_id).into_iter().flatten() {
                    if let Some((_, src_node)) = update.nodes.iter().find(|(id, _)| id == src) {
                        name = src_node
                            .label()
                            .map(str::to_owned)
                            .or_else(|| src_node.value().map(str::to_owned));
                        if name.is_some() {
                            break;
                        }
                    }
                }
            }
            if name.is_none() && matches!(node.role(), Role::Label) {
                name = node.value().map(str::to_owned);
            }
            if name.as_deref().is_some_and(|n| n.trim().is_empty()) {
                name = None; // whitespace-only is not a name
            }
            let widget_id = if u64::from(*node_id) == egui::accesskit_root_id().value() {
                None
            } else {
                // NodeId wraps the egui Id's u64 verbatim (`Id::accesskit_id`
                // is `self.value().into()`); the inverse is by construction.
                u64_to_id(u64::from(*node_id))
            };
            nodes.insert(
                *node_id,
                SemanticNode {
                    widget_id,
                    node_id: *node_id,
                    role: node.role(),
                    role_debug: format!("{:?}", node.role()),
                    name,
                    disabled: node.is_disabled(),
                    hidden: node.is_hidden(),
                    supports_click: node.supports_action(Action::Click),
                    supports_focus: node.supports_action(Action::Focus),
                    parent: parent_of.get(node_id).copied(),
                },
            );
        }
        Some(Self {
            nodes,
            focus: Some(update.focus),
        })
    }

    /// Look a node up by its egui widget id.
    pub fn find_widget(&self, id: Id) -> Option<&SemanticNode> {
        self.nodes.get(&id.accesskit_id())
    }
}

/// Convert an accesskit `NodeId` payload back to the `egui::Id` it came from.
/// Valid because egui constructs node ids as `id.value().into()` — the bits
/// are the Id's own high-entropy hash. `None` for zero (not a valid `Id`).
fn u64_to_id(bits: u64) -> Option<Id> {
    if bits == 0 {
        None
    } else {
        // SAFETY-contract: `from_high_entropy_bits` requires non-zero, which
        // is checked above; the value is an Id's own bits so entropy holds.
        Some(unsafe { Id::from_high_entropy_bits(bits) })
    }
}

/// Roles that demand an accessible name — pointing/keyboard users and screen
/// readers alike have no way to understand an unnamed interactive control.
pub(crate) fn is_name_required_role(role: Role) -> bool {
    matches!(
        role,
        Role::Button
            | Role::CheckBox
            | Role::RadioButton
            | Role::ComboBox
            | Role::Slider
            | Role::SpinButton
            | Role::TextInput
            | Role::MultilineTextInput
            | Role::Link
            | Role::MenuItem
            | Role::MenuItemCheckBox
            | Role::MenuItemRadio
            | Role::Tab
            | Role::TreeItem
            | Role::ListBoxOption
            | Role::Switch
            | Role::SearchInput
            | Role::ColorWell
            | Role::Splitter
            | Role::ScrollBar
            | Role::Cell
            | Role::RowHeader
            | Role::ColumnHeader
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::accesskit::{Node, Tree};

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
}
