//! Presentation layer for the input family.
//!
//! The concrete widgets stay in focused private modules: they own egui layout and painting,
//! while pure choices (labels and counters) are delegated to [`super::handler`].

#[path = "password.rs"]
mod password;
#[path = "search.rs"]
mod search;
#[path = "text.rs"]
mod text;
#[path = "textarea.rs"]
mod textarea;

pub use password::PasswordInput;
pub use search::SearchInput;
pub use text::Input;
pub use textarea::Textarea;
