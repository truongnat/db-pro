# Sprint 2 - S2-01 Connection Controller Extraction

## Goal

Move connection CRUD/reset/cache orchestration out of `App.tsx` into a dedicated controller hook while preserving behavior.

## Delivered

- New controller hook:
  - `src/features/connections/useConnectionController.ts`
- Moved from `App.tsx` into controller:
  - connection list loading and preferred selection restore
  - add/edit modal open/close/reset flows
  - connection save/delete flows
  - selected-connection cache persistence
  - create-draft cache persistence
- Kept reset-data orchestration in `App.tsx`, but delegated connection state reset via:
  - `applyResetConnectionState(...)`

## Result

- `App.tsx` reduced from 1130 LOC to 957 LOC.
- Connection business logic is now isolated and reusable.

## Validation

- `npm run -s build` => PASS
- `cargo check --manifest-path src-tauri/Cargo.toml` => PASS
- `cargo run --manifest-path /Users/truongdq/Documents/GitHub/agentic-sdlc/Cargo.toml -- workflow check` => PASS

Validated on 2026-03-05.
