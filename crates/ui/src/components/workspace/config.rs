use lucide_icons::Icon;

use super::handler::ActivityBarItemKind;

// Fixed icon advance used by status-bar text measurement and rendering, in egui points.
pub(crate) const STATUS_BAR_ICON_SLOT_WIDTH: f32 = 16.0;
// Galley placement uses the same center factor for every status-bar label.
pub(crate) const STATUS_TEXT_VERTICAL_CENTER_FACTOR: f32 = 0.5;
// Horizontal inset that centers each activity control in the rail, in egui points.
pub(crate) const ACTIVITY_ITEM_LEFT_INSET: f32 = 6.0;
// Square hit-target and background dimensions for activity items, in egui points.
pub(crate) const ACTIVITY_ITEM_SIZE: f32 = 36.0;
// Vertical pitch between activity items, including their separation, in egui points.
pub(crate) const ACTIVITY_ITEM_STEP: f32 = 42.0;
// Width of the selected activity marker, in egui points.
pub(crate) const ACTIVITY_ACCENT_WIDTH: f32 = 2.5;
// Top inset of the selected marker within an activity item, in egui points.
pub(crate) const ACTIVITY_ACCENT_TOP_INSET: f32 = 8.0;
// Height of the selected marker, in egui points.
pub(crate) const ACTIVITY_ACCENT_HEIGHT: f32 = 20.0;
// Corner radius of the selected activity marker, in egui points.
pub(crate) const ACTIVITY_ACCENT_RADIUS: f32 = 1.0;
// Square allocation reserved for the connection-state indicator, in egui points.
pub(crate) const CONNECTION_DOT_BOX_SIZE: f32 = 8.0;
// Painted connection-state dot radius, in egui points.
pub(crate) const CONNECTION_DOT_RADIUS: f32 = 3.5;
// Latency above this threshold is shown as a warning; units are milliseconds.
pub(crate) const LATENCY_WARNING_THRESHOLD_MS: u32 = 200;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ActivityBarDefinition {
    pub kind: ActivityBarItemKind,
    pub icon: Icon,
    pub tooltip: &'static str,
}

// Canonical activity order and discoverability labels shared by the rail renderer.
pub(crate) const ACTIVITY_BAR_ITEMS: [ActivityBarDefinition; 4] = [
    ActivityBarDefinition {
        kind: ActivityBarItemKind::Explorer,
        icon: Icon::FolderTree,
        tooltip: "Database Explorer",
    },
    ActivityBarDefinition {
        kind: ActivityBarItemKind::QueryEditor,
        icon: Icon::CodeXml,
        tooltip: "SQL Editor",
    },
    ActivityBarDefinition {
        kind: ActivityBarItemKind::Agent,
        icon: Icon::Sparkles,
        tooltip: "AI Copilot",
    },
    ActivityBarDefinition {
        kind: ActivityBarItemKind::Diagram,
        icon: Icon::Network,
        tooltip: "Schema ER Diagram",
    },
];
