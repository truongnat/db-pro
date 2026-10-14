#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "en");

mod agent;
mod app;
mod chart;
pub mod components;
#[cfg(debug_assertions)]
mod dev_tools;
pub mod diagram;
pub mod editor;
pub mod i18n;
mod policy;
pub mod query;
mod result_grid;
mod runtime;
#[cfg(test)]
#[path = "app/test_frame.rs"]
pub(crate) mod test_frame;
mod theme;

// `app::*` also surfaces `app::result_grid_view::GridSelectionLookup`, pinned for
// `benches/result_grid_benchmarks.rs` (a bench is a separate crate and can only
// reach the lookup through the crate root).
pub use agent::*;
pub use app::*;
pub use chart::*;
pub use components::*;
pub use i18n::*;
pub use policy::*;
pub use result_grid::*;
pub use runtime::*;
pub use theme::*;
