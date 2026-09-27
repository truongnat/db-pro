mod config;
mod handler;
mod ui;

pub use config::*;
pub use handler::{count_diff_changes, diff_line_visual, format_diff_stats, DiffLine, DiffLineType, DiffLineVisual};
pub use ui::DiffViewer;
