# Component UI Layer Structure

## State
- **State:** IMPLEMENTING
- **Branch:** `main`
- **Baseline SHA:** `2fb54db0`
- **Surface:** `crates/ui/src/components/` native egui component library.

## Goal
Organize every public UI component into a predictable folder with a single `mod.rs` entry/exporter, UI and handler layers, component-local `config.rs`, and separate design, usage, and API documentation. Shared design tokens and common UI primitives stay centralized and are reused by components.

## Required component shape
```text
<component>/
  mod.rs       # public entry point; declares modules and re-exports public API
  ui.rs        # egui drawing/layout; use ui/ for distinct UI parts
  handler.rs   # behavior and calculations; use handlers/ for distinct decision groups
  config.rs    # component-local defaults only; no core imports or mirrored core constants
  DESIGN.md    # UI design rationale and logic/event/performance design
  README.md    # short human-facing introduction and example
  API.md       # detailed developer/AI usage contract
```

`mod.rs`, `config.rs`, `DESIGN.md`, `README.md`, and `API.md` are required. The UI and handler layers may each be a single file or a directory with private submodules. Keep handler code meaningful: move behavior/decision code out of UI; do not create placeholder boilerplate. `config.rs` holds only component-owned values; consumers reference `tokens.rs`/`DbProTheme` directly where needed. Existing components migrate incrementally, starting with Button.

## Scope
- Migrate the complete public component surface exported by `crates/ui/src/components/mod.rs`, including already-multi-file components (`dialog`, `form`, `input`, `select`, `tabs`).
- Keep shared infrastructure (`animation`, `interact`, shared tokens/helpers, legacy compatibility facade) centralized rather than pretending they are standalone UI components.
- Preserve current public exports and runtime behavior during each migration.
- Add `README.md`, `DESIGN.md`, and `API.md` for every public component folder as each component is revisited. Keep the README introductory, the design rationale in DESIGN, and the detailed usage contract in API.
- Migrate in coherent batches on `main` under the owner workflow override, with compile/test checks when validation is requested. Select was the first completed family in the original migration; Button is the first family under this revised documentation contract.

## Non-goals
- UI redesign, changes to behavior/API beyond necessary module path updates, and changes to database/runtime layers.
- A generic component framework, empty handler/config placeholders, or duplicated design tokens.

## Implementation guide

Use the durable [component authoring guide](../../../crates/ui/src/components/README.md) and migrated `Select` component as references when creating or migrating a component:

1. Inspect the current public API, callers, tests and shared UI patterns before moving files. Record the observable interaction contract (including focus, keyboard/mouse behavior, dismissal and accessibility identity where applicable).
2. Choose a stable caller-facing API, then make the component's `mod.rs` its public entry point. Preserve the expected `components/mod.rs` re-exports and existing behavior; update callers only when module paths require it.
3. Keep egui widget construction, layout, geometry acquisition and painting in `ui.rs` or `ui/`. Move meaningful decisions, state transitions, validation and UI-independent calculations into typed `handler.rs` or `handlers/` functions. Pass measurements/signals explicitly and have UI apply handler outcomes; do not move painting into handlers.
4. Put only component-owned defaults/constants in `config.rs`; it has no core imports. UI and handlers use shared `DbProTheme` and `tokens.rs` values directly. Give non-obvious factors/numbers semantic names. If a layer has no meaningful responsibility, record the exception rather than adding placeholder code.
5. Add focused tests for handler boundaries/state transitions. Write `README.md` for basic use, `DESIGN.md` for UI and logic rationale, and `API.md` for the full developer/AI usage contract.
   Comment non-obvious Rust flow in English beside the code, tracing caller inputs and egui events through handler decisions/state to UI rendering and returned actions. Explain precedence and intent, not individual statements.
6. Migrate one coherent component/family at a time. Review exports, callers, behavior and diff for drift; run formatting and the smallest relevant crate test/check, then applicable broader gates. Record only commands actually completed and distinguish automated evidence from runtime evidence in `VERIFICATION.md`.

Example layering: `select/ui/option.rs` obtains the text bounds and galley size from egui, then calls `option_galley_pos(left, vertical_center, galley_height)`; `select/handler.rs` performs the pure position calculation using `VERTICAL_CENTER_FACTOR` from `config.rs`.

## Acceptance criteria
- Every public component gains the required folder/layers and three documents as it is revisited; Button is the first component under the revised contract.
- `mod.rs` is the only public component entry/exporter and public API remains compatible through `components` re-exports.
- UI files focus on egui rendering/layout and call typed handler functions for behavior/calculation.
- Component-specific settings are kept in config; shared spacing/colors use canonical tokens/theme.
- Light and dark appearances, component states, typography, spacing, and semantic colors remain consistent with the core theme.
- Interactive controls expose a meaningful name, role, state, keyboard activation, and visible focus; check contrast, target size, reduced motion, and accessibility-tree output.
- User-facing actions and loading/disabled/error/empty/narrow states remain understandable without color or motion alone.
- Shared visual changes come from theme/core tokens, while supported component variants remain easy to configure without duplicated values.
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
