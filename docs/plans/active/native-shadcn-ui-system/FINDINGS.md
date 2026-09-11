# Findings: Native Shadcn-like Common UI System

## Initial Findings & Baseline Analysis

### Finding 1 (Design Consistency)
- **Status**: Identified.
- **Evidence**: `crates/ui/src/components.rs` currently contains only primitive button helpers (`primary_button`, `secondary_button`, `ghost_button`, `compact_button`) and rudimentary inputs without explicit variants, size scales, loading states, error boundaries, or helper text.
- **Severity**: P2 (Design System & Polish).
- **Resolution**: Build a dedicated modular component system mirroring the standard API conventions of `shadcn/ui` (Variants, Sizes, Compound parts).

### Finding 2 (Component Discoverability & Developer Experience)
- **Status**: Identified.
- **Evidence**: Previously there was no visual gallery or Storybook-like environment in the native app to inspect and test UI components in isolation across Light and Dark themes.
- **Severity**: P2 (Developer Experience & Visual Quality).
- **Resolution**: Provide a dedicated `ComponentShowcase` workspace tab in `DbProApp` that acts as a full-featured living style guide and component playground.
