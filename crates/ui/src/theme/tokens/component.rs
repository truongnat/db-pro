//! Component token contracts — layer 3 of `Primitive → Semantic → Component`.
//!
//! Each contract states, for one representative component: which semantic roles it
//! consumes, which states it actually supports, the mandatory interaction-state
//! precedence, and which values it owns. Component rendering must not introduce raw
//! color/radius/spacing/stroke values: colors arrive through `DbProTheme` (the
//! semantic facade), shared scales through the `crate::tokens` facade, and
//! component-owned sizes through the contracts below. Genuinely local geometry
//! stays in the component's `config.rs`, labelled there as implementation geometry.

pub mod button;
pub mod dialog;
pub mod feedback;
pub mod input;
pub mod shell;
pub mod table;

/// Mandatory interaction-state precedence, strongest first, for every contract in
/// this module. When several states apply at once, the earlier entry wins.
///
/// Component-specific status states slot in as documented by their contract:
/// input inserts `error` directly after `disabled` (before `focus`), and table
/// resolves `selected` above `hover`. `error`/`success` are display statuses, not
/// interaction states, so they are not part of this chain.
pub const STATE_PRECEDENCE: [&str; 6] = ["disabled", "loading", "active", "focus", "hover", "default"];

#[cfg(test)]
#[path = "tests/component_tests.rs"]
mod tests;
