//! Segmented and underline tab tracks for native shell surfaces.

mod config;
mod layout;
mod segmented;
mod style;
mod track;
mod underline;

#[cfg(test)]
mod tests;

pub use segmented::SegmentedTabs;
pub use underline::UnderlineTabs;
