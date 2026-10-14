//! Segmented and underline tab tracks for native shell surfaces.

mod config;
mod document_tab;
mod handler;
mod layout;
mod style;
mod track;
mod ui;

#[cfg(test)]
mod tests;

pub use document_tab::{draw_document_tab, DocumentTabAction, DocumentTabItem, DOCUMENT_TAB_HEIGHT};
pub use ui::{SegmentedTabs, UnderlineTabs};
