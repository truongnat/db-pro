//! Workspace and Application Shell components.
//!
//! This module preserves the public workspace component API while keeping egui
//! rendering in [`ui`], state/event types and deterministic layout decisions in
//! [`handler`], and component-specific values in [`config`].

mod config;
mod handler;
mod ui;

pub use handler::{ActivityBarItemKind, ConnectionHealth, StatusBarItem};
pub use ui::{ActivityBar, ConnectionIndicator, StatusBar};
