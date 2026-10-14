mod config;
mod handler;
mod ui;

pub use ui::{Alert, AlertDialog, AlertDialogAction, AlertVariant};

#[cfg(test)]
#[path = "tests/mod_tests.rs"]
mod tests;
