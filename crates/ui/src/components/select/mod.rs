mod config;
mod handler;
mod ui;

#[cfg(test)]
mod tests;

pub use handler::dropdown_should_open_above;
pub use ui::{paint_option, Select, SelectOption, SelectSize, SelectVariant};
