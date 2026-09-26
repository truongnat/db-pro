# Component Gallery Redesign Checklist

## Research

- [x] Trace all gallery entry points and renderer modules.
- [x] Compare the current surface with `docs/DESIGN.md`, `DbProTheme` and `tokens.rs`.
- [x] Identify clipping, monolithic rendering and card-style drift.

## Implementation

- [x] Replace the 14-item horizontal segmented track with vertical category navigation.
- [x] Remove the `All` rendering mode and render one category at a time.
- [x] Standardize title, subtitle and action hierarchy.
- [x] Apply spacing, typography, border and radius tokens.
- [x] Align the shared Card primitive with the workstation design contract.
- [x] Add deterministic Component Gallery capture routing.
- [x] Align Badges & Status Pills with the canonical Stitch execution-status specimen.
- [x] Use the existing UI suite plus native captures; no tautological gallery-wiring test added.

## Verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo test -p db-pro-ui --lib`
- [x] `cargo check --workspace`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo build --release --locked -p db-pro-native`
- [x] Focused badge tests — 5 passed / 0 failed / 0 ignored.
- [x] Native runtime capture of the corrected badge section at 1280×800.
- [x] Native runtime capture requested at 1280×800
- [x] Native runtime capture requested at 1440×900; host capped content height at 838px
- [x] Native runtime capture requested at 1920×1080; host capped content height at 838px
- [x] Record P0/P1/P2 review outcome and known limitations
