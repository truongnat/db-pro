//! Layout adapter between DB Pro's egui host and the rs-ui retained layout runtime.
//!
//! egui remains responsible for windowing, input, and painting during the incremental
//! migration. This adapter lets rs-ui own shell geometry without making DB Pro state
//! or domain types depend on rs-ui.

use rs_ui_core::{Rect, Size};
use rs_ui_runtime::{
    AccessibilityRole, AccessibilitySemantics, Align, BehaviorCommand, Constraints, Dimension, HitTestState,
    LayoutMode, LayoutStyle, NodeId, PaintState, PointerEvents, ResizeAxis, ResizeConfig, RuntimeError, ScrollState,
    UiTree,
};
use std::fmt;
use std::time::Duration;

#[path = "native_explorer_tree_runtime.rs"]
mod native_explorer_tree_runtime;
pub(crate) use native_explorer_tree_runtime::ExplorerTreeItem;
use native_explorer_tree_runtime::ExplorerTreeRuntime;

const TOOLBAR_HEIGHT: f32 = 42.0;
pub(crate) const TABS_HEIGHT: f32 = 34.0;
const MIN_MAIN_WIDTH: f32 = 420.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ShellRegions {
    pub(crate) sidebar: Rect,
    pub(crate) main: Rect,
    pub(crate) tabs: Rect,
    pub(crate) workspace: Rect,
}

pub(crate) fn layout_shell(viewport: Size, sidebar_width: f32) -> Option<ShellRegions> {
    match try_layout_shell(viewport, sidebar_width) {
        Ok(regions) => regions,
        Err(error) => {
            tracing::error!(%error, "rs-ui shell layout failed; keeping egui fallback geometry");
            None
        }
    }
}

#[derive(Debug)]
enum ShellLayoutError {
    Runtime(RuntimeError),
    MissingBounds(NodeId),
}

impl fmt::Display for ShellLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Runtime(error) => write!(formatter, "{error}"),
            Self::MissingBounds(node) => write!(formatter, "layout did not assign bounds to node {}", node.get()),
        }
    }
}

impl From<RuntimeError> for ShellLayoutError {
    fn from(error: RuntimeError) -> Self {
        Self::Runtime(error)
    }
}

fn try_layout_shell(viewport: Size, sidebar_width: f32) -> Result<Option<ShellRegions>, ShellLayoutError> {
    let main_width = viewport.width - sidebar_width;
    if !viewport.width.is_finite()
        || !viewport.height.is_finite()
        || !sidebar_width.is_finite()
        || viewport.width <= 0.0
        || viewport.height <= TOOLBAR_HEIGHT + TABS_HEIGHT
        || sidebar_width <= 0.0
        || main_width < MIN_MAIN_WIDTH
    {
        return Ok(None);
    }

    let layout = ShellLayoutSpec {
        viewport,
        sidebar_width,
        main_width,
    };
    let (mut tree, nodes) = build_shell_tree(layout)?;
    tree.layout(nodes.root, Constraints::loose(viewport))?;

    Ok(Some(ShellRegions {
        sidebar: rect(&tree, nodes.sidebar)?,
        main: rect(&tree, nodes.main)?,
        tabs: rect(&tree, nodes.tabs)?,
        workspace: rect(&tree, nodes.workspace)?,
    }))
}

struct ShellNodeIds {
    root: NodeId,
    sidebar: NodeId,
    main: NodeId,
    tabs: NodeId,
    workspace: NodeId,
}

struct ShellLayoutSpec {
    viewport: Size,
    sidebar_width: f32,
    main_width: f32,
}

fn build_shell_tree(layout: ShellLayoutSpec) -> Result<(UiTree, ShellNodeIds), ShellLayoutError> {
    let mut tree = UiTree::new();
    let (root, body) = create_shell_root(&mut tree, layout.viewport)?;
    let (sidebar, main, tabs, workspace) = create_main_regions(&mut tree, body, layout)?;

    Ok((
        tree,
        ShellNodeIds {
            root,
            sidebar,
            main,
            tabs,
            workspace,
        },
    ))
}

fn create_shell_root(tree: &mut UiTree, viewport: Size) -> Result<(NodeId, NodeId), ShellLayoutError> {
    let root = create_node(tree, None, style(LayoutMode::Column, viewport.width, viewport.height))?;
    create_node(tree, Some(root), style(LayoutMode::Row, viewport.width, TOOLBAR_HEIGHT))?;
    let body = create_node(
        tree,
        Some(root),
        LayoutStyle {
            mode: LayoutMode::Row,
            flex_grow: 1.0,
            align_x: Align::Stretch,
            align_y: Align::Stretch,
            ..LayoutStyle::default()
        },
    )?;
    Ok((root, body))
}

fn create_main_regions(
    tree: &mut UiTree,
    body: NodeId,
    layout: ShellLayoutSpec,
) -> Result<(NodeId, NodeId, NodeId, NodeId), ShellLayoutError> {
    let sidebar = create_node(
        tree,
        Some(body),
        style(
            LayoutMode::Column,
            layout.sidebar_width,
            layout.viewport.height - TOOLBAR_HEIGHT,
        ),
    )?;
    let main = create_node(
        tree,
        Some(body),
        LayoutStyle {
            mode: LayoutMode::Column,
            flex_grow: 1.0,
            align_x: Align::Stretch,
            align_y: Align::Stretch,
            ..LayoutStyle::default()
        },
    )?;
    let tabs = create_node(tree, Some(main), style(LayoutMode::Row, layout.main_width, TABS_HEIGHT))?;
    let workspace = create_node(
        tree,
        Some(main),
        LayoutStyle {
            mode: LayoutMode::Column,
            flex_grow: 1.0,
            align_x: Align::Stretch,
            align_y: Align::Stretch,
            ..LayoutStyle::default()
        },
    )?;
    Ok((sidebar, main, tabs, workspace))
}

fn create_node(tree: &mut UiTree, parent: Option<NodeId>, style: LayoutStyle) -> Result<NodeId, RuntimeError> {
    tree.create_node(parent, style, PaintState::default())
}

fn style(mode: LayoutMode, width: f32, height: f32) -> LayoutStyle {
    LayoutStyle {
        mode,
        width: Dimension::Points(width),
        height: Dimension::Points(height),
        align_x: Align::Stretch,
        align_y: Align::Stretch,
        ..LayoutStyle::default()
    }
}

fn rect(tree: &UiTree, id: NodeId) -> Result<Rect, ShellLayoutError> {
    tree.node(id)
        .and_then(|node| node.layout_rect())
        .ok_or(ShellLayoutError::MissingBounds(id))
}

/// Persistent rs-ui behavior state for the egui-hosted shell.
pub(crate) struct RsUiShellRuntime {
    tree: UiTree,
    sidebar_splitter: Option<NodeId>,
    output_splitters: [Option<NodeId>; 2],
    sidebar_scroll: Option<NodeId>,
    tabs_root: Option<NodeId>,
    tab_nodes: std::collections::HashMap<String, NodeId>,
    active_tab_keys: std::collections::HashSet<String>,
    focused_tab_seen: bool,
    explorer_tree_runtime: ExplorerTreeRuntime,
    sidebar_width: f32,
    resizing_sidebar: bool,
}

impl Default for RsUiShellRuntime {
    fn default() -> Self {
        let tree = UiTree::new();
        Self {
            tree,
            sidebar_splitter: None,
            output_splitters: [None, None],
            sidebar_scroll: None,
            tabs_root: None,
            tab_nodes: std::collections::HashMap::new(),
            active_tab_keys: std::collections::HashSet::new(),
            focused_tab_seen: false,
            explorer_tree_runtime: ExplorerTreeRuntime::default(),
            sidebar_width: 260.0,
            resizing_sidebar: false,
        }
    }
}

impl RsUiShellRuntime {
    pub(crate) fn begin_explorer_tree(&mut self) {
        self.explorer_tree_runtime.begin();
    }

    pub(crate) fn queue_explorer_tree_navigation(&mut self, command: BehaviorCommand) {
        self.explorer_tree_runtime.queue_navigation(command);
    }

    pub(crate) fn explorer_item_has_focus(&self, key: &str) -> bool {
        self.explorer_tree_runtime.item_has_focus(key)
    }

    pub(crate) fn register_explorer_tree_item(&mut self, item: ExplorerTreeItem<'_>) -> Result<bool, RuntimeError> {
        self.explorer_tree_runtime.register_item(item)
    }

    pub(crate) fn end_explorer_tree(&mut self) -> Result<(), RuntimeError> {
        self.explorer_tree_runtime.end()
    }

    fn ensure_output_splitter(&mut self, horizontal: bool) -> Result<NodeId, RuntimeError> {
        let index = usize::from(horizontal);
        if let Some(node) = self.output_splitters[index] {
            return Ok(node);
        }
        let root = self
            .tree
            .create_node(None, LayoutStyle::default(), PaintState::default())?;
        let splitter = self
            .tree
            .create_node(Some(root), LayoutStyle::default(), PaintState::default())?;
        self.output_splitters[index] = Some(splitter);
        Ok(splitter)
    }

    pub(crate) fn begin_output_resize(
        &mut self,
        horizontal: bool,
        value: f32,
        min: f32,
        max: f32,
        pointer: rs_ui_core::Point,
    ) -> Result<(), RuntimeError> {
        let splitter = self.ensure_output_splitter(horizontal)?;
        let axis = if horizontal {
            ResizeAxis::Horizontal
        } else {
            ResizeAxis::Vertical
        };
        self.tree.register_resizable(
            splitter,
            ResizeConfig {
                axis,
                min,
                max,
                step: 8.0,
                reset: value,
            },
            value,
            Some("Query output dock size".to_owned()),
        )?;
        self.tree
            .begin_resize(splitter, invert_splitter_pointer(horizontal, pointer))
    }

    pub(crate) fn update_output_resize(
        &mut self,
        horizontal: bool,
        pointer: rs_ui_core::Point,
    ) -> Result<f32, RuntimeError> {
        let splitter = self.ensure_output_splitter(horizontal)?;
        self.tree
            .update_resize(splitter, invert_splitter_pointer(horizontal, pointer))?;
        self.tree
            .resizable_value(splitter)
            .ok_or(RuntimeError::UnknownNode(splitter))
    }

    pub(crate) fn end_output_resize(&mut self, horizontal: bool) -> Result<(), RuntimeError> {
        let splitter = self.ensure_output_splitter(horizontal)?;
        self.tree.end_resize(splitter)
    }

    fn ensure_tabs_root(&mut self) -> Result<NodeId, RuntimeError> {
        if let Some(node) = self.tabs_root {
            return Ok(node);
        }
        let root = self
            .tree
            .create_node(None, LayoutStyle::default(), PaintState::default())?;
        self.tree
            .set_accessibility_semantics(root, AccessibilitySemantics::new(AccessibilityRole::TabList))?;
        self.tabs_root = Some(root);
        Ok(root)
    }

    pub(crate) fn begin_workspace_tabs(&mut self) {
        self.active_tab_keys.clear();
        self.focused_tab_seen = false;
    }

    pub(crate) fn register_workspace_tab(
        &mut self,
        key: &str,
        label: &str,
        selected: bool,
        bounds: egui::Rect,
        focused: bool,
        clicked: bool,
    ) -> Result<bool, RuntimeError> {
        let root = self.ensure_tabs_root()?;
        let node = if let Some(node) = self.tab_nodes.get(key).copied() {
            node
        } else {
            let node = self
                .tree
                .create_node(Some(root), LayoutStyle::default(), PaintState::default())?;
            self.tree.register_pressable(node, Some(label.to_owned()), false)?;
            let mut semantics = AccessibilitySemantics::new(AccessibilityRole::Tab);
            semantics.label = Some(label.to_owned());
            self.tree.set_accessibility_semantics(node, semantics)?;
            self.tab_nodes.insert(key.to_owned(), node);
            node
        };

        let mut semantics = AccessibilitySemantics::new(AccessibilityRole::Tab);
        semantics.label = Some(label.to_owned());
        self.tree.set_accessibility_semantics(node, semantics)?;
        self.tree.apply_selection_state(node, selected)?;
        self.tree.set_hit_test_state(
            node,
            HitTestState {
                bounds: rs_ui_core::Rect::from_min_max(
                    rs_ui_core::Point::new(bounds.min.x, bounds.min.y),
                    rs_ui_core::Point::new(bounds.max.x, bounds.max.y),
                ),
                pointer_events: PointerEvents::Auto,
                ..HitTestState::default()
            },
        )?;
        self.active_tab_keys.insert(key.to_owned());

        if focused {
            self.focused_tab_seen = true;
            let _ = self.tree.request_focus(node)?;
        }
        if !clicked {
            return Ok(false);
        }
        let _ = self.tree.request_focus(node)?;
        let event = self
            .tree
            .dispatch_behavior_command(root, BehaviorCommand::Activate, Duration::ZERO)?;
        Ok(event.default_prevented())
    }

    pub(crate) fn register_workspace_close_button(
        &mut self,
        parent_key: &str,
        key: &str,
        label: &str,
        bounds: egui::Rect,
        focused: bool,
        clicked: bool,
    ) -> Result<bool, RuntimeError> {
        let root = self.ensure_tabs_root()?;
        let parent = self
            .tab_nodes
            .get(parent_key)
            .copied()
            .ok_or(RuntimeError::UnknownNode(root))?;
        let node = if let Some(node) = self.tab_nodes.get(key).copied() {
            node
        } else {
            let node = self
                .tree
                .create_node(Some(parent), LayoutStyle::default(), PaintState::default())?;
            self.tree.register_pressable(node, Some(label.to_owned()), false)?;
            self.tab_nodes.insert(key.to_owned(), node);
            node
        };
        let mut semantics = AccessibilitySemantics::new(AccessibilityRole::Button);
        semantics.label = Some(label.to_owned());
        self.tree.set_accessibility_semantics(node, semantics)?;
        self.tree.set_hit_test_state(
            node,
            HitTestState {
                bounds: rs_ui_core::Rect::from_min_max(
                    rs_ui_core::Point::new(bounds.min.x, bounds.min.y),
                    rs_ui_core::Point::new(bounds.max.x, bounds.max.y),
                ),
                pointer_events: PointerEvents::Auto,
                ..HitTestState::default()
            },
        )?;
        self.active_tab_keys.insert(key.to_owned());
        if focused {
            self.focused_tab_seen = true;
            let _ = self.tree.request_focus(node)?;
        }
        if !clicked {
            return Ok(false);
        }
        let _ = self.tree.request_focus(node)?;
        let event = self
            .tree
            .dispatch_behavior_command(root, BehaviorCommand::Activate, Duration::ZERO)?;
        Ok(event.default_prevented())
    }

    pub(crate) fn activate_workspace_item(&mut self, key: &str) -> Result<bool, RuntimeError> {
        let root = self.ensure_tabs_root()?;
        let node = self
            .tab_nodes
            .get(key)
            .copied()
            .ok_or(RuntimeError::UnknownNode(root))?;
        let _ = self.tree.request_focus(node)?;
        let event = self
            .tree
            .dispatch_behavior_command(root, BehaviorCommand::Activate, Duration::ZERO)?;
        Ok(event.default_prevented())
    }

    pub(crate) fn workspace_item_has_focus(&self, key: &str) -> bool {
        self.tab_nodes
            .get(key)
            .is_some_and(|node| self.tree.focus_manager().focused() == Some(*node))
    }

    pub(crate) fn end_workspace_tabs(&mut self) -> Result<(), RuntimeError> {
        let focused_tab = self
            .tree
            .focus_manager()
            .focused()
            .is_some_and(|focused| self.tab_nodes.values().any(|node| *node == focused));
        if focused_tab && !self.focused_tab_seen {
            self.tree.clear_focus();
        }
        let removed = self
            .tab_nodes
            .iter()
            .filter(|(key, _)| !self.active_tab_keys.contains(*key))
            .map(|(key, node)| (key.clone(), *node))
            .collect::<Vec<_>>();
        for (key, node) in removed {
            self.tree.remove_subtree(node)?;
            self.tab_nodes.remove(&key);
        }
        self.active_tab_keys.clear();
        Ok(())
    }

    fn ensure_sidebar_scroll(&mut self) -> Result<NodeId, RuntimeError> {
        if let Some(node) = self.sidebar_scroll {
            return Ok(node);
        }
        let root = self
            .tree
            .create_node(None, LayoutStyle::default(), PaintState::default())?;
        let scroll = self
            .tree
            .create_node(Some(root), LayoutStyle::default(), PaintState::default())?;
        self.sidebar_scroll = Some(scroll);
        Ok(scroll)
    }

    pub(crate) fn scroll_sidebar(
        &mut self,
        viewport: Size,
        wheel_delta: rs_ui_core::Point,
    ) -> Result<f32, RuntimeError> {
        let node = self.ensure_sidebar_scroll()?;
        let mut state = self
            .tree
            .scroll_state(node)
            .unwrap_or_else(|| ScrollState::new(viewport, viewport));
        state.set_viewport(viewport);
        self.tree.set_scroll_state(node, state)?;
        self.tree
            .scroll_by(node, rs_ui_core::Point::new(-wheel_delta.x, -wheel_delta.y))?;
        Ok(self.tree.scroll_state(node).map_or(0.0, |state| state.offset.y))
    }

    pub(crate) fn sync_sidebar_scroll(
        &mut self,
        viewport: Size,
        content_size: Size,
        offset: rs_ui_core::Point,
    ) -> Result<(), RuntimeError> {
        let node = self.ensure_sidebar_scroll()?;
        let mut state = self
            .tree
            .scroll_state(node)
            .unwrap_or_else(|| ScrollState::new(viewport, content_size));
        state.set_viewport(viewport);
        state.set_content_size(content_size);
        state.set_offset(offset);
        self.tree.set_scroll_state(node, state)
    }

    pub(crate) fn sidebar_scroll_offset(&self) -> Option<f32> {
        let node = self.sidebar_scroll?;
        self.tree.scroll_state(node).map(|state| state.offset.y)
    }

    fn ensure_sidebar_splitter(&mut self) -> Result<NodeId, RuntimeError> {
        if let Some(node) = self.sidebar_splitter {
            return Ok(node);
        }
        let root = self
            .tree
            .create_node(None, LayoutStyle::default(), PaintState::default())?;
        let splitter = self
            .tree
            .create_node(Some(root), LayoutStyle::default(), PaintState::default())?;
        self.sidebar_splitter = Some(splitter);
        Ok(splitter)
    }

    pub(crate) fn begin_sidebar_resize(
        &mut self,
        width: f32,
        min_width: f32,
        max_width: f32,
        pointer: rs_ui_core::Point,
    ) -> Result<(), RuntimeError> {
        let splitter = self.ensure_sidebar_splitter()?;
        self.tree.register_resizable(
            splitter,
            ResizeConfig {
                axis: ResizeAxis::Horizontal,
                min: min_width,
                max: max_width,
                step: 8.0,
                reset: width,
            },
            width,
            Some("Sidebar width".to_owned()),
        )?;
        self.tree.begin_resize(splitter, pointer)?;
        self.resizing_sidebar = true;
        self.sidebar_width = width;
        Ok(())
    }

    pub(crate) fn update_sidebar_resize(&mut self, pointer: rs_ui_core::Point) -> Result<f32, RuntimeError> {
        if !self.resizing_sidebar {
            return Ok(self.sidebar_width);
        }
        let splitter = self.ensure_sidebar_splitter()?;
        self.tree.update_resize(splitter, pointer)?;
        let width = self
            .tree
            .resizable_value(splitter)
            .ok_or(RuntimeError::UnknownNode(splitter))?;
        self.sidebar_width = width;
        Ok(width)
    }

    pub(crate) fn end_sidebar_resize(&mut self) -> Result<(), RuntimeError> {
        if self.resizing_sidebar {
            let splitter = self.ensure_sidebar_splitter()?;
            self.tree.end_resize(splitter)?;
            self.resizing_sidebar = false;
        }
        Ok(())
    }

    pub(crate) fn adjust_sidebar_width(
        &mut self,
        width: f32,
        min_width: f32,
        max_width: f32,
        command: BehaviorCommand,
    ) -> Result<f32, RuntimeError> {
        let splitter = self.ensure_sidebar_splitter()?;
        self.tree.register_resizable(
            splitter,
            ResizeConfig {
                axis: ResizeAxis::Horizontal,
                min: min_width,
                max: max_width,
                step: 8.0,
                reset: width,
            },
            width,
            Some("Sidebar width".to_owned()),
        )?;
        self.tree.request_focus(splitter)?;
        self.tree.dispatch_behavior_command(splitter, command, Duration::ZERO)?;
        let width = self
            .tree
            .resizable_value(splitter)
            .ok_or(RuntimeError::UnknownNode(splitter))?;
        self.sidebar_width = width;
        Ok(width)
    }
}

fn invert_splitter_pointer(horizontal: bool, pointer: rs_ui_core::Point) -> rs_ui_core::Point {
    if horizontal {
        rs_ui_core::Point::new(-pointer.x, pointer.y)
    } else {
        rs_ui_core::Point::new(pointer.x, -pointer.y)
    }
}

#[cfg(test)]
#[path = "native_runtime_shell_tests.rs"]
mod tests;
