mod config;
mod handler;
mod ui;

pub use handler::{day_of_week, days_in_month, is_leap_year, next_month, previous_month};
pub use ui::{Calendar, DatePicker, SimpleDate};

#[cfg(test)]
#[path = "tests/mod_tests.rs"]
mod tests;
