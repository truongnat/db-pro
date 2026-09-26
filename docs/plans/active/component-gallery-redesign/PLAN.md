# Component Gallery Redesign

- **State:** RUNTIME_VERIFY
- **Branch:** `feature/component-gallery-redesign`
- **Baseline SHA:** `7bbfb08e289fa2b77867d40feb0d886dbc097d31`
- **Surface:** native `egui` Component Gallery only

## Problem

The current gallery uses a single unscrollable 14-item segmented tab row, renders every complex sample in the default `All` view, and presents the component system as rounded shadow cards. This clips navigation at normal workspace widths, creates an unnecessarily heavy immediate-mode page, and contradicts the DB Pro workstation direction in `docs/DESIGN.md`.

## Decision

Reshape the gallery as a restrained workstation browser:

- persistent vertical category rail;
- one selected category rendered at a time;
- compact 36px utility header with canonical naming and actions;
- flat, token-driven planes and separators;
- shared typography/spacing/radius tokens;
- deterministic native capture entry point.

Keep all existing interactive component demos. Do not create a second theme or component implementation.

## Scope

1. Replace horizontal category tabs and `All` mode with reachable vertical navigation.
2. Standardize gallery title, category naming, section hierarchy and spacing.
3. Bring the shared Card primitive into the documented 4px, shadow-free workstation contract.
4. Add a Component Gallery capture hook.
5. Verify through the existing UI suite and native captures; do not add tautological wiring tests.
6. Capture the affected surface at 1280×800, 1440×900 and 1920×1080.

## Non-goals

- Rewriting individual component behavior.
- Adding new database/provider operations.
- Replacing the result grid or Explorer implementations with gallery-only samples.
- Redesigning archived frontend assets.
- Claiming PostgreSQL or SQLite runtime behavior; this surface is provider-independent.

## Architecture

`WorkspaceTab::ComponentGallery` remains the routing contract. `ComponentGalleryState::category` owns selection. A fixed navigation rail emits category changes; the detail pane owns vertical scrolling and renders only the selected section. `DbProTheme` and `tokens.rs` remain the only visual-token sources.

## Acceptance criteria

- Every gallery category is reachable at 1280px viewport width without horizontal clipping.
- Only the selected category's component section is rendered.
- Header and navigation use canonical `Component Gallery` naming and Lucide icons; no decorative emoji.
- Gallery layout uses theme surfaces, 1px separators, spacing tokens and 4px dense-surface radius.
- Shared Card has no decorative shadow and uses semantic card radius/padding.
- Capture harness can open the gallery deterministically.
- Focused tests, workspace check/clippy and native release build outcomes are recorded honestly.
- Native screenshots exist for 1280×800, 1440×900 and 1920×1080, or the exact runtime blocker is recorded.
