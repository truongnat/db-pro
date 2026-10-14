//! Tests for `../workspace_shell.rs` (moved out of the source file).
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