# Native Core Architecture — Verification

Source checkpoint: `0bde3a5f4091f2ce3e6ccdc3a12bb2f5a765c5db`.

## Current change

- Connection dialog state aggregate added under `crates/ui/src/connection/state.rs`.
- Connection dialog view, form, advanced panels, events and workspace actions
  now address the aggregate instead of individual `DbProApp` fields.
- Connection lifecycle state now owns active/pending/error/request state.
- `ConnectionCatalogState` now owns the saved-connection read model and its
  replacement/lookup operations.
- `WorkspaceShellState` now owns shell navigation, panel visibility/geometry,
  welcome lifecycle and pending navigation state; panel resize values are
  clamped through state setters.
- `QuerySessionState` now owns query documents, active selection, selected text,
  save/close request tracking and Save As lifecycle.
- Unit tests for the new aggregates: 7 passed, 0 failed.
- `cargo check -p db-pro-ui`: PASS.
- `cargo fmt --all`: executed.
- `cargo clippy -p db-pro-ui --all-targets -- -D warnings`: PASS.
- `cargo test -p db-pro-ui --lib`: 555 passed, 0 failed.
- `cargo fmt --all -- --check`: PASS.
- `cargo check --workspace`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace --no-fail-fast`: 1227 passed, 0 failed, 42 ignored.
- `cargo build --release --locked -p db-pro-native`: PASS.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`: 11 pass, 5 warnings, 0 failures; warnings are ratcheted size/cast/clone heuristics.

## Not yet proven

- Native screenshot/runtime evidence for all affected states.
- Completion of the remaining feature-state migrations.
