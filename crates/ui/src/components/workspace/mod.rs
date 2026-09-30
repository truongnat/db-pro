//! Workspace and Application Shell components.
//!
//! This module preserves the public workspace component API while keeping egui
//! rendering in [`ui`], state/event types in [`handler`], and deterministic
//! layout/configuration helpers in [`config`].

mod config;
mod handler;
mod ui;

pub use handler::{ActivityBarItemKind, ConnectionHealth, StatusBarItem};
pub use ui::{ActivityBar, ConnectionIndicator, StatusBar};
