# Component UI Layer Structure

## State
- **State:** IMPLEMENTING
- **Branch:** `main`
- **Baseline SHA:** `2fb54db0`
- **Surface:** `crates/ui/src/components/` native egui component library.

## Goal
Organize every public UI component into a predictable folder with a single `mod.rs` entry/exporter, `ui.rs` presentation layer, `handler.rs` behavior/decision layer, `config.rs` component-local configuration, and `README.md` usage guide. Shared design tokens and common UI primitives stay centralized and are reused by components.

## Required component shape
```text
<component>/
  mod.rs       # public entry point; declares modules and re-exports public API
  ui.rs        # egui drawing/layout; delegates decisions to handler
  handler.rs   # component behavior, state transitions, validation and pure calculations
  config.rs    # component-local defaults only; shared tokens/theme remain common
  README.md    # concise usage/API/behavior guide with example
```

Each file is required for migrated components. Keep `handler.rs` meaningful: move behavior/decision code out of UI; do not create placeholder boilerplate. Keep configuration local only when component-specific; reference shared settings from `tokens.rs`/`DbProTheme` rather than duplicate them.

## Scope
- Migrate the complete public component surface exported by `crates/ui/src/components/mod.rs`, including already-multi-file components (`dialog`, `form`, `input`, `select`, `tabs`).
- Keep shared infrastructure (`animation`, `interact`, shared tokens/helpers, legacy compatibility facade) centralized rather than pretending they are standalone UI components.
- Preserve current public exports and runtime behavior during each migration.
- Add a `README.md` for every public component folder; document purpose, public API, key behavior, and a concise Rust usage example.
- Migrate in coherent batches on this feature branch, with compile/test checks after each batch. Select is the first completed family in the full-scope migration, not a scope limit.

## Non-goals
- UI redesign, changes to behavior/API beyond necessary module path updates, and changes to database/runtime layers.
- A generic component framework, empty handler/config placeholders, or duplicated design tokens.

## Implementation guide

Use the durable [component authoring guide](../../../crates/ui/src/components/README.md) and migrated `Select` component as references when creating or migrating a component:

1. Inspect the current public API, callers, tests and shared UI patterns before moving files. Record the observable interaction contract (including focus, keyboard/mouse behavior, dismissal and accessibility identity where applicable).
2. Choose a stable caller-facing API, then make the component's `mod.rs` its public entry point. Preserve the expected `components/mod.rs` re-exports and existing behavior; update callers only when module paths require it.
3. Keep egui widget construction, layout, geometry acquisition and painting in `ui.rs` (private `ui/` submodules are allowed). Move meaningful decisions, state transitions, validation and UI-independent calculations into typed `handler.rs` functions. Pass measurements/signals explicitly and have UI apply handler outcomes; do not move painting into handlers.
4. Put only component-owned defaults/constants in `config.rs`. Reuse shared `DbProTheme` and `tokens.rs` values; do not duplicate shared tokens. Give non-obvious factors/numbers semantic names. If a layer has no meaningful responsibility, record the exception rather than adding placeholder code.
5. Add focused tests for handler boundaries/state transitions and a `README.md` documenting purpose, public API, important behavior/constraints and a concise usage example.
6. Migrate one coherent component/family at a time. Review exports, callers, behavior and diff for drift; run formatting and the smallest relevant crate test/check, then applicable broader gates. Record only commands actually completed and distinguish automated evidence from runtime evidence in `VERIFICATION.md`.

Example layering: `select/ui/option.rs` obtains the text bounds and galley size from egui, then calls `option_galley_pos(left, vertical_center, galley_height)`; `select/handler.rs` performs the pure position calculation using `VERTICAL_CENTER_FACTOR` from `config.rs`.

## Acceptance criteria
- Every public component has the required folder/layers and README.
- `mod.rs` is the only public component entry/exporter and public API remains compatible through `components` re-exports.
- UI files focus on egui rendering/layout and call typed handler functions for behavior/calculation.
- Component-specific settings are kept in config; shared spacing/colors use canonical tokens/theme.
- Rust component refactors include detailed English comments for non-obvious logic and UI flow, explaining behavior step by step (inputs/events → handlers/state/outcomes → rendering). Comments focus on intent and rationale, not merely restating code.
- Rust formatting, workspace checks, clippy, tests and native release build are executed and recorded.

## Decisions / risks
- Do not move pure UI rendering to handlers; egui paint/layout remains in `ui.rs`. Handler owns decisions and state transitions, even if UI passes egui input signals and applies returned actions.
- Large folders may require additional private submodules below the named layer files, but only `mod.rs` is the public entry.
- Keep migrations staged so a broad filesystem refactor does not hide behavior regressions.

## Provider impact
N/A — UI architecture only; no PostgreSQL/SQLite behavior changes.

## Tổng kết bằng tiếng Việt
Phạm vi là toàn bộ component công khai, không dừng ở Select. Mỗi component có `mod.rs`, `ui.rs`, `handler.rs`, `config.rs`, `README.md`; Select là nhóm đầu tiên. Phần hướng dẫn triển khai component mới dùng Select làm ví dụ, nhấn mạnh giữ API, tách phép tính thuần khỏi egui, tái sử dụng token/theme và kiểm chứng đúng mức.
