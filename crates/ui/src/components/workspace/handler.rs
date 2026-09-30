use lucide_icons::Icon;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_bar_item_builder_preserves_public_state() {
        let item = StatusBarItem::new("Ready")
            .icon(Icon::Check)
            .tooltip("All systems operational")
            .accent(true);

        assert_eq!(item.text, "Ready");
        assert!(matches!(item.icon, Some(Icon::Check)));
        assert_eq!(item.tooltip.as_deref(), Some("All systems operational"));
        assert!(item.is_accent);
    }

    #[test]
    fn connection_health_labels_match_workspace_copy() {
        assert_eq!(connection_health_label(ConnectionHealth::Healthy), "Connected");
        assert_eq!(connection_health_label(ConnectionHealth::Degraded), "High Latency");
        assert_eq!(connection_health_label(ConnectionHealth::Disconnected), "Disconnected");
    }
}
