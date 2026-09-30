# Core Component Style System — Checklist

## Discovery
- [x] Confirm target audience, use cases, tone and screenshot reference.
- [x] Choose dark database workstation direction with compact shadcn-inspired controls.
- [x] Define first batch boundary: theme, tokens, Button, Input, Card, focus/interaction.

## Implementation
- [x] Recalibrate dark `DbProTheme` surfaces, borders, text and accent.
- [x] Align shared radius, spacing, control-height and typography tokens.
- [x] Restyle Button variants without changing public API or click semantics.
- [x] Restyle Input family with clear idle/hover/focus/error states through shared theme tokens.
- [x] Restyle Card and shared focus treatment through shared surface/border tokens.
- [x] Add or update characterization tests for visual-token contracts.
- [x] Fix gallery primitive overflow: wrap Checkbox/Radio copy to local cell width.
- [x] Make unavailable controls explicit and visually legible in disabled state.
- [x] Recompose Disclosure & Choice into three compact panels without nested outer-card chrome.
- [x] Show Calendar inline, persist gallery toggle/calendar state, and fix Accordion icon/title spacing.

## Verification
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] `cargo build --release --locked -p db-pro-native`
- [x] Runtime evidence at 1280×800, 1440×900, 1920×1080 for Disclosure & Choice; captures are in `evidence/disclosure-choice-controls-*.png`.
- [ ] Review loading, error and empty states where the batch is visible.

## Open follow-up
- [ ] Apply calibrated core primitives to shell/data-grid/sidebar in the next batch.
- [ ] Capture independent accessibility/focus evidence for keyboard interactions.

## Tổng kết bằng tiếng Việt
Checklist tách rõ việc chuẩn hóa core components khỏi việc lan style ra toàn bộ shell/data-grid/sidebar. Runtime evidence vẫn là gate riêng, không suy diễn từ source/test.
