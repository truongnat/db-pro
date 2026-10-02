//! Welcome-only renderer experiment; no TaskBridge, database, or production host.

use db_pro_ui::{DbProTheme, UiConnectionSummary};
use rs_ui_core::{Color, Point, Radius, Size, Srgb8, Stroke};
use rs_ui_runtime::{Constraints, Dimension, LayoutStyle, NodeId, PaintState, Position, UiTree};
use rs_ui_text::{FontFamily, FontWeight, TextStyle, TextSystem};

type SpikeResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Default)]
struct WelcomeSnapshot {
    active_connection: Option<UiConnectionSummary>,
    draft: String,
}

impl WelcomeSnapshot {
    fn subtitle(&self) -> String {
        match &self.active_connection {
            Some(connection) => format!("Connected to {}", connection.name),
            None => "Write SQL. Connect when the query needs a database.".to_owned(),
        }
    }
}

pub struct WelcomeScene {
    pub tree: UiTree,
    pub text: TextSystem,
    pub root: NodeId,
    content: NodeId,
    card: NodeId,
    viewport: Size,
}

impl WelcomeScene {
    pub fn new() -> SpikeResult<Self> {
        let theme = DbProTheme::light();
        let snapshot = WelcomeSnapshot::default();
        let mut tree = UiTree::new();
        let root = tree.create_node(None, LayoutStyle::default(), fill(theme.surface_panel.to_array()))?;
        let content = tree.create_node(Some(root), LayoutStyle::default(), PaintState::default())?;
        let card = tree.create_node(
            Some(content),
            layout(Size::new(980.0, 212.0), Point::new(0.0, 84.0)),
            PaintState {
                background: Some(color(theme.surface_elevated.to_array())),
                radius: Radius::all(12.0),
                border: Some(Stroke::new(1.0, color(theme.border_subtle.to_array()))),
            },
        )?;
        let mut scene = Self {
            tree,
            text: TextSystem::new(),
            root,
            content,
            card,
            viewport: Size::ZERO,
        };
        scene.label(
            content,
            Point::ZERO,
            "DB Pro",
            style(32.0, FontWeight::Medium, theme.text_primary.to_array()),
        )?;
        scene.node(
            content,
            layout(Size::new(36.0, 3.0), Point::new(0.0, 40.0)),
            fill(theme.accent.to_array()),
        )?;
        scene.label(
            content,
            Point::new(0.0, 50.0),
            &snapshot.subtitle(),
            style(15.0, FontWeight::Regular, theme.text_secondary.to_array()),
        )?;
        let prompt = if snapshot.draft.is_empty() {
            "Ask in SQL. Ctrl+Enter opens the editor."
        } else {
            &snapshot.draft
        };
        scene.label(
            card,
            Point::new(12.0, 12.0),
            prompt,
            TextStyle {
                family: FontFamily::Monospace,
                ..style(13.0, FontWeight::Regular, theme.text_primary.to_array())
            },
        )?;
        let buttons = [
            ("Open in editor", 115.0, theme.accent, theme.accent_foreground, None),
            (
                "New connection",
                129.0,
                theme.surface_elevated,
                theme.text_primary,
                Some(theme.border_default),
            ),
        ];
        let mut x = 12.0;
        for (label, minimum_width, background, foreground, border) in buttons {
            let button = scene.node(
                card,
                layout(Size::new(minimum_width, 28.0), Point::new(x, 54.0)),
                PaintState {
                    background: Some(color(background.to_array())),
                    radius: Radius::all(4.0),
                    border: border.map(|border| Stroke::new(1.0, color(border.to_array()))),
                },
            )?;
            scene.tree.register_pressable(
                button,
                Some(label.to_owned()),
                label == "Open in editor" && snapshot.draft.is_empty(),
            )?;
            scene.node(
                button,
                layout(Size::new(12.0, 12.0), Point::new(8.0, 8.0)),
                PaintState {
                    radius: Radius::all(2.0),
                    border: Some(Stroke::new(1.0, color(foreground.to_array()))),
                    ..PaintState::default()
                },
            )?;
            let label_size = scene.label(
                button,
                Point::new(27.0, 5.0),
                label,
                style(13.0, FontWeight::Medium, foreground.to_array()),
            )?;
            let width = minimum_width.max(label_size.width + 35.0);
            scene
                .tree
                .set_layout(button, layout(Size::new(width, 28.0), Point::new(x, 54.0)))?;
            x += width + 4.0;
        }
        for (index, (title, detail)) in [
            ("Orders by status", "Group the lab order book"),
            ("Top customers", "Highest spend in lab.customers"),
            ("Event payloads", "Recent jsonb events"),
        ]
        .into_iter()
        .enumerate()
        {
            let y = 98.0 + index as f32 * 36.0;
            scene.label(
                card,
                Point::new(40.0, y),
                title,
                style(13.0, FontWeight::Medium, theme.text_primary.to_array()),
            )?;
            scene.label(
                card,
                Point::new(40.0, y + 16.0),
                detail,
                style(12.0, FontWeight::Regular, theme.text_muted.to_array()),
            )?;
            scene.node(
                card,
                layout(Size::new(12.0, 1.0), Point::new(15.0, y + 11.0)),
                fill(theme.accent.to_array()),
            )?;
            scene.node(
                card,
                layout(Size::new(1.0, 12.0), Point::new(20.0, y + 6.0)),
                fill(theme.accent.to_array()),
            )?;
        }
        Ok(scene)
    }

    pub fn layout(&mut self, viewport: Size) -> SpikeResult<()> {
        if self.viewport == viewport {
            return Ok(());
        }
        self.viewport = viewport;
        // Reserve the production shell's footprint without rendering its surfaces.
        let body_width = (viewport.width - 315.0).max(80.0);
        let width = (body_width - 40.0).min(980.0);
        let left = 315.0 + ((body_width - width) * 0.5).max(20.0);
        let top = 70.0 + ((viewport.height - 100.0 - 460.0).max(0.0) * 0.18).clamp(24.0, 96.0);
        self.tree.set_layout(self.root, layout(viewport, Point::ZERO))?;
        self.tree
            .set_layout(self.content, layout(Size::new(width, 320.0), Point::new(left, top)))?;
        self.tree
            .set_layout(self.card, layout(Size::new(width, 212.0), Point::new(0.0, 84.0)))?;
        self.tree.layout(self.root, Constraints::loose(viewport))?;
        Ok(())
    }

    fn node(&mut self, parent: NodeId, layout: LayoutStyle, paint: PaintState) -> SpikeResult<NodeId> {
        Ok(self.tree.create_node(Some(parent), layout, paint)?)
    }

    fn label(&mut self, parent: NodeId, origin: Point, value: &str, style: TextStyle) -> SpikeResult<Size> {
        let run = self.text.shape(value, style, None);
        let metrics = self.text.run_metrics(run).ok_or("shaped text run is missing")?;
        let node = self.node(parent, layout(metrics.size, origin), PaintState::default())?;
        self.tree.set_text(node, run, metrics)?;
        Ok(metrics.size)
    }
}

fn layout(size: Size, origin: Point) -> LayoutStyle {
    LayoutStyle {
        width: Dimension::Points(size.width),
        height: Dimension::Points(size.height),
        position: Position::Absolute(origin),
        ..LayoutStyle::default()
    }
}

fn color([r, g, b, a]: [u8; 4]) -> Color {
    Color::from_srgba8(Srgb8 { r, g, b, a })
}
fn fill(rgba: [u8; 4]) -> PaintState {
    PaintState {
        background: Some(color(rgba)),
        ..PaintState::default()
    }
}
fn style(size_px: f32, weight: FontWeight, rgba: [u8; 4]) -> TextStyle {
    TextStyle {
        size_px,
        line_height_px: size_px + 5.0,
        weight,
        color: color(rgba),
        ..TextStyle::default()
    }
}
