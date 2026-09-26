//! Feature-owned state for the native workspace shell and navigation surface.
//!
//! This aggregate owns shell layout and navigation selection. Query documents,
//! connection lifecycle and feature data remain in their own aggregates.

use super::*;

#[derive(Debug)]
pub(crate) struct WorkspaceShellState {
    pub(crate) activity: Activity,
    pub(crate) welcome_open: bool,
    pub(crate) active_tab: WorkspaceTab,
    pub(crate) sidebar_open: bool,
    pub(crate) sidebar_width: f32,
    pub(crate) agent_open: bool,
    pub(crate) agent_width: f32,
    pub(crate) bottom_panel_open: bool,
    pub(crate) bottom_panel_height: f32,
    pub(crate) output_dock_position: OutputDockPosition,
    pub(crate) right_dock_width: f32,
    pub(crate) sidebar_open_before_agent: Option<bool>,
    pub(crate) pending_navigation_action: Option<PendingNavigationAction>,
    pub(crate) split_editor_secondary: Option<usize>,
}

impl Default for WorkspaceShellState {
    fn default() -> Self {
        Self {
            activity: Activity::Explorer,
            welcome_open: true,
            active_tab: WorkspaceTab::Welcome,
            sidebar_open: true,
            sidebar_width: 260.0,
            agent_open: false,
            agent_width: 360.0,
            bottom_panel_open: false,
            bottom_panel_height: 180.0,
            output_dock_position: OutputDockPosition::Bottom,
            right_dock_width: 480.0,
            sidebar_open_before_agent: None,
            pending_navigation_action: None,
            split_editor_secondary: None,
        }
    }
}

impl WorkspaceShellState {
    pub(crate) fn set_sidebar_width(&mut self, width: f32) {
        self.sidebar_width = width.clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
    }

    pub(crate) fn set_agent_width(&mut self, width: f32) {
        self.agent_width = width.clamp(AGENT_MIN_WIDTH, AGENT_MAX_WIDTH);
    }

    pub(crate) fn set_bottom_panel_height(&mut self, height: f32) {
        self.bottom_panel_height = height.clamp(OUTPUT_MIN_HEIGHT, OUTPUT_MAX_HEIGHT);
    }

    pub(crate) fn set_right_dock_width(&mut self, width: f32) {
        self.right_dock_width = width.clamp(240.0, 1200.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_shell_state_starts_at_welcome_with_stable_panel_sizes() {
        let state = WorkspaceShellState::default();

        assert_eq!(state.activity, Activity::Explorer);
        assert_eq!(state.active_tab, WorkspaceTab::Welcome);
        assert!(state.welcome_open);
        assert!(state.sidebar_open);
        assert!(!state.agent_open);
        assert_eq!(state.sidebar_width, 260.0);
        assert_eq!(state.agent_width, 360.0);
        assert_eq!(state.bottom_panel_height, 180.0);
    }

    #[test]
    fn shell_panel_sizes_are_clamped_at_the_state_boundary() {
        let mut state = WorkspaceShellState::default();

        state.set_sidebar_width(f32::MAX);
        state.set_agent_width(f32::MIN);
        state.set_bottom_panel_height(f32::MAX);

        assert_eq!(state.sidebar_width, SIDEBAR_MAX_WIDTH);
        assert_eq!(state.agent_width, AGENT_MIN_WIDTH);
        assert_eq!(state.bottom_panel_height, OUTPUT_MAX_HEIGHT);
    }
}
