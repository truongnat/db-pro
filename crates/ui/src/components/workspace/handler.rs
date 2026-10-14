use egui::{Pos2, Rect, Vec2};
use lucide_icons::Icon;

use super::config::{
    ACTIVITY_ACCENT_HEIGHT, ACTIVITY_ACCENT_TOP_INSET, ACTIVITY_ACCENT_WIDTH, ACTIVITY_ITEM_LEFT_INSET,
    ACTIVITY_ITEM_SIZE, ACTIVITY_ITEM_STEP, LATENCY_WARNING_THRESHOLD_MS, STATUS_BAR_ICON_SLOT_WIDTH,
    STATUS_TEXT_VERTICAL_CENTER_FACTOR,
};
use crate::tokens::SPACE_MD;

/// Top-level workspace destinations shown by the activity bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityBarItemKind {
    Explorer,
    QueryEditor,
    Agent,
    Diagram,
    Settings,
}

/// High-level connection status used by the workspace connection indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionHealth {
    Healthy,
    Degraded,
    Disconnected,
}

/// Display model for a status bar entry.
#[derive(Debug, Clone)]
pub struct StatusBarItem {
    pub text: String,
    pub icon: Option<Icon>,
    pub tooltip: Option<String>,
    pub is_accent: bool,
}

impl StatusBarItem {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            tooltip: None,
            is_accent: false,
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn accent(mut self, accent: bool) -> Self {
        self.is_accent = accent;
        self
    }
}

pub(crate) fn connection_health_label(health: ConnectionHealth) -> &'static str {
    match health {
        ConnectionHealth::Healthy => "Connected",
        ConnectionHealth::Degraded => "High Latency",
        ConnectionHealth::Disconnected => "Disconnected",
    }
}

pub(crate) fn status_item_width(text_width: f32, has_icon: bool) -> f32 {
    if has_icon {
        STATUS_BAR_ICON_SLOT_WIDTH + text_width
    } else {
        text_width
    }
}

pub(crate) fn next_status_item_cursor(cursor: f32, item_width: f32) -> f32 {
    cursor + item_width + SPACE_MD
}

pub(crate) fn right_status_item_start(cursor: f32, item_width: f32) -> f32 {
    cursor - item_width
}

/// Total width of the right-anchored status block including inter-item gaps.
pub(crate) fn right_status_block_width(item_widths: &[f32]) -> f32 {
    match item_widths.len() {
        0 => 0.0,
        count => item_widths.iter().sum::<f32>() + SPACE_MD * (count - 1) as f32,
    }
}

/// Right edge that left-side status items may paint up to: the right block's left edge minus one
/// gap, or the inner right edge of the bar when no right items exist.
pub(crate) fn left_status_items_limit(bar_inner_right: f32, right_block_width: f32, has_right_items: bool) -> f32 {
    if has_right_items {
        bar_inner_right - right_block_width - SPACE_MD
    } else {
        bar_inner_right
    }
}

pub(crate) fn status_text_top(center_y: f32, text_height: f32) -> f32 {
    center_y - text_height * STATUS_TEXT_VERTICAL_CENTER_FACTOR
}

pub(crate) fn activity_item_start_y(container_top: f32) -> f32 {
    container_top + SPACE_MD
}

pub(crate) fn activity_item_rect(container_left: f32, y: f32) -> Rect {
    Rect::from_min_size(
        Pos2::new(container_left + ACTIVITY_ITEM_LEFT_INSET, y),
        Vec2::splat(ACTIVITY_ITEM_SIZE),
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
#[path = "tests/handler_tests.rs"]
mod tests;
