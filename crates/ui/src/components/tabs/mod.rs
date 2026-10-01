//! Segmented and underline tab tracks for native shell surfaces.

mod config;
mod handler;
mod layout;
mod style;
mod track;
mod ui;

#[cfg(test)]
mod tests;

pub use ui::{SegmentedTabs, UnderlineTabs};
