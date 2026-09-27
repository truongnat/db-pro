mod config;
mod handler;
mod ui;

pub use config::*;
pub use handler::{
    clamp_progress, mac_dot_colors, mac_dot_positions, progress_ring_arc_points, terminal_title, DEFAULT_TERMINAL_TITLE,
};
pub use ui::{ProgressRing, TerminalBlock};
