# Component Layer Structure — Findings

## Baseline
- Baseline: `2fb54db0` (`main` before this feature branch).
- Select previously mixed egui rendering with popup keyboard/state decisions, accessible-label construction, and width calculations.
- Most exported components are standalone `.rs` files; complex families already use subdirectories but inconsistent file boundaries.

## Decisions
- `mod.rs` is the public component entry/exporter.
- Every public component is in scope and must have the named layers plus a usage README.
- UI owns egui rendering/layout and applies egui side effects; handler owns decision logic and calculations called by UI.
- Config owns component-specific defaults; shared design tokens/theme stay centralized.
- Migrate in batches to keep behavior and API stable; empty handler/config files are not accepted, so implementation must move meaningful responsibilities into them.

## Findings
- P2 — Select mixed UI and logic. Its keyboard navigation, trigger/accessibility labels, widths, option selection, dismissal, load-more decisions and dropdown geometry now live in typed handler functions.
- No provider/database impact; this is native UI architecture work.

## Unresolved
- Inventory each public component's meaningful handler/config responsibilities during its batch; do not duplicate shared settings or fabricate placeholder logic.