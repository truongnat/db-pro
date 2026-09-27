use crate::tokens::*;
use egui::{Pos2, Rect, Vec2};
use lucide_icons::Icon;

use super::handler::ActivityBarItemKind;

// Fixed icon advance used by status-bar text measurement and rendering, in egui points.
pub(crate) const STATUS_BAR_ICON_SLOT_WIDTH: f32 = 16.0;
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

pub(crate) fn status_item_icon_width(has_icon: bool) -> f32 {
    if has_icon {
        STATUS_BAR_ICON_SLOT_WIDTH
    } else {
        0.0
    }
}

pub(crate) fn status_item_width(text_width: f32, has_icon: bool) -> f32 {
    status_item_icon_width(has_icon) + text_width
}

pub(crate) fn next_status_item_cursor(cursor: f32, item_width: f32) -> f32 {
    cursor + item_width + SPACE_MD
}

pub(crate) fn right_status_item_start(cursor: f32, item_width: f32) -> f32 {
    cursor - item_width
}

pub(crate) fn activity_item_start_y(container_top: f32) -> f32 {
    container_top + SPACE_MD
}

pub(crate) fn activity_item_rect(container_left: f32, y: f32) -> Rect {
    Rect::from_min_size(
        Pos2::new(container_left + ACTIVITY_ITEM_LEFT_INSET, y),
        Vec2::new(ACTIVITY_ITEM_SIZE, ACTIVITY_ITEM_SIZE),
    )
}

pub(crate) fn activity_accent_rect(container_left: f32, item_top: f32) -> Rect {
    Rect::from_min_size(
        Pos2::new(container_left, item_top + ACTIVITY_ACCENT_TOP_INSET),
        Vec2::new(ACTIVITY_ACCENT_WIDTH, ACTIVITY_ACCENT_HEIGHT),
    )
}

pub(crate) fn next_activity_item_y(current_y: f32) -> f32 {
    current_y + ACTIVITY_ITEM_STEP
}

pub(crate) fn is_latency_warning(latency_ms: u32) -> bool {
    latency_ms > LATENCY_WARNING_THRESHOLD_MS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_bar_cursor_math_matches_existing_spacing() {
        assert_eq!(status_item_width(24.0, false), 24.0);
        assert_eq!(status_item_width(24.0, true), 40.0);
        assert_eq!(next_status_item_cursor(10.0, 40.0), 10.0 + 40.0 + SPACE_MD);
        assert_eq!(right_status_item_start(100.0, 40.0), 60.0);
    }

    #[test]
    fn activity_bar_config_keeps_existing_order_and_layout() {
        let kinds: Vec<ActivityBarItemKind> = ACTIVITY_BAR_ITEMS.iter().map(|item| item.kind).collect();
        assert_eq!(
            kinds,
            vec![
                ActivityBarItemKind::Explorer,
                ActivityBarItemKind::QueryEditor,
                ActivityBarItemKind::Agent,
                ActivityBarItemKind::Diagram,
            ]
        );

        let y = activity_item_start_y(3.0);
        let rect = activity_item_rect(10.0, y);
        assert_eq!(rect.min, Pos2::new(16.0, 3.0 + SPACE_MD));
        assert_eq!(rect.size(), Vec2::new(36.0, 36.0));
        assert_eq!(next_activity_item_y(y), y + ACTIVITY_ITEM_STEP);
    }

    #[test]
    fn latency_warning_boundary_preserves_previous_threshold() {
        assert!(!is_latency_warning(200));
        assert!(is_latency_warning(201));
    }
}
