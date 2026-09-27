pub mod config;
pub mod frame;
mod handler;
pub mod layout;
pub(crate) mod modal_guard;
pub mod sheet;
mod ui;

#[cfg(test)]
mod tests;

/// Compatibility path for the pre-layered modal API.
pub mod modal {
    pub use super::ui::{close_icon_button, dialog_actions, Dialog, DialogActionLabels};
}

pub use frame::DialogFrame;
pub use modal::{close_icon_button, dialog_actions, Dialog, DialogActionLabels};
pub use sheet::Sheet;
