# Native IDE Redesign

## Status

`COMPLETED`

## Source requirement

Root requirement: [`goal-1.md`](../../../../goal-1.md).

## Goal

Evolve the Rust + egui native workspace into a professional database IDE with a stable workbench shell, compact database navigation, keyboard-first query work, staged data editing, contextual Agent actions, and provider-safe advanced tooling.

## Scope

- P0: shell layout, resizable/persistent panels, design tokens and keyboard routing.
- P1: compact hierarchical explorer and object context actions.
- P2: multi-document SQL workspace and result/message/explain output surface.
- P3: staged editable result grid with apply/discard and safe row identity.
- P4: table/object workspace metadata views.
- P5: command palette and Quick Open parity.
- P6: contextual Agent actions and reviewable SQL drafts.
- P7: provider-aware advanced tooling already exposed by the shared runtime.

## Non-goals

- Rewriting the Rust domain/application services that already provide the database contract.
- Replacing egui with a web renderer or adding a second UI stack.
- Silent destructive database operations.
- Claiming runtime completion without a visible native walkthrough for PostgreSQL and SQLite.

## Completion gates

- Every P0–P7 item has source and automated evidence, with unsupported provider paths explicit.
- `cargo fmt --all -- --check`, workspace check/clippy/tests pass.
- PostgreSQL and SQLite are independently exercised where the UI reaches database behavior.
- Native runtime walkthrough covers connection, schema, query, result grid, object workspace, Agent draft and recovery/error states.
- `goal-1.md` acceptance criteria are mapped to evidence in `VERIFICATION.md`.
