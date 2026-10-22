mod config;
pub mod field;
mod handler;
pub mod rules;
pub mod state;
mod ui;

#[cfg(test)]
#[path = "tests/mod_tests.rs"]
mod tests;

pub use field::{FormField, Label};
pub use rules::{CustomValidator, FieldRule, ValidationMode};
pub use state::FormState;
