//! Feature: Select
//! Screen code: N/A (shared component; no canonical screen code)
//! Screen name: N/A (shared component; no canonical screen name)
//! Functionality: Render an accessible dropdown selector and coordinate its popup interaction.
//! Comment: Keep presentation here; selection and geometry decisions live in `handler.rs`.

mod option;
mod select;

pub use option::{paint_option, SelectOption};
pub use select::Select;
