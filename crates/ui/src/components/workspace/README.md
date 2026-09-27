# Workspace component split

`workspace` keeps the existing public API from the former `workspace.rs` module while separating responsibilities:

- `mod.rs` — module boundary and public re-exports.
- `ui.rs` — egui painting and widget builders for `StatusBar`, `ActivityBar`, and `ConnectionIndicator`.
- `handler.rs` — public state types and small state/copy helpers.
- `config.rs` — deterministic layout constants and pure calculations covered by unit tests.

Behavior is intentionally preserved: activity bar item order, spacing, status item measurement, connection latency threshold, and exported type names remain unchanged.
