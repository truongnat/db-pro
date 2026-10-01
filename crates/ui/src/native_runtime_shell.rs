//! Layout adapter between DB Pro's egui host and the rs-ui retained layout runtime.
//!
//! egui remains responsible for windowing, input, and painting during the incremental
//! migration. This adapter lets rs-ui own shell geometry without making DB Pro state
//! or domain types depend on rs-ui.

use rs_ui_core::{Rect, Size};
use rs_ui_runtime::{
    Align, BehaviorCommand, Constraints, Dimension, LayoutMode, LayoutStyle, NodeId, PaintState, ResizeAxis,
    ResizeConfig, RuntimeError, ScrollState, UiTree,
};
use std::fmt;
use std::time::Duration;

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
    sidebar_scroll: Option<NodeId>,
    sidebar_width: f32,
    resizing_sidebar: bool,
}

impl Default for RsUiShellRuntime {
    fn default() -> Self {
        let tree = UiTree::new();
        Self {
            tree,
            sidebar_splitter: None,
            sidebar_scroll: None,
            sidebar_width: 260.0,
            resizing_sidebar: false,
        }
    }
}

impl RsUiShellRuntime {
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

#[cfg(test)]
mod tests {
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
}
