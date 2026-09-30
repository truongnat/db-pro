# Checklist

## Planning

- [x] Read the project design context and clean-code guidance.
- [x] Inspect both components, their handlers/configs, and gallery usage.
- [x] Keep the change on local `main` per the owner workflow override.

## Implementation

- [x] Add one shared disclosure visual/animation layer.
- [x] Remove per-component duplicate header and animation constants.
- [x] Use semantic transparent/hover/active surfaces.
- [x] Use egui's clipped body animation for both components.
- [x] Preserve public APIs and state transitions.

## Verification

- [x] Capture dark component-gallery disclosure surface at 1280×800.
- [x] Run component tests, check, clippy, fmt, and clean-code scan.
- [x] Run full workspace test/release gates after the final source change.
- [ ] Independent review remains pending.
