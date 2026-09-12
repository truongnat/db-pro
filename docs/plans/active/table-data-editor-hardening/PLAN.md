# Table Data Editor Hardening

## Objective

Make staged table mutations safe under primary-key edits and concurrent row
changes. The first implementation slice is limited to stable row identity,
composed ChangeSet mutations, zero/multiple affected-row handling, and the
native UI failure state. Filtering, sorting, layout persistence, and metadata
expansion remain follow-up slices unless they are required by these invariants.

## Scope

- Preserve original primary-key values for every update and delete predicate.
- Support composite primary keys and primary-key edits (`WHERE` original,
  `SET` final values).
- Keep no-primary-key tables read-only for row edit/delete.
- Merge staged edits by stable row identity into one update per row.
- Drop no-op/reverted cells and make insert/delete staging reversible.
- Map zero affected rows to a conflict while retaining staged state.
- Treat more than one affected row as an invariant violation.
- Keep failure targeting and retry/reload/discard actions connected to the
  staged row.

## Out of scope for this slice

- Full grid layout persistence and virtualization redesign.
- New metadata fields not required for mutation identity.
- Binary editing.
- Broad UI redesign or archived frontend changes.

## Provider matrix

| Provider | Mutation support | Automated evidence | Runtime evidence | Capability gate |
|---|---|---|---|---|
| PostgreSQL | parameterized PK update/delete/insert | pending | pending | existing connector transaction path |
| SQLite | parameterized PK update/delete/insert | pending | pending | existing connector transaction path |

## Completion gates

- Focused UI/core tests cover the required identity and ChangeSet cases.
- `cargo fmt --all -- --check`, `cargo check --workspace`,
  `cargo clippy --workspace --all-targets -- -D warnings`, and
  `cargo test --workspace` pass.
- Native release build passes.
- Provider and native runtime evidence is recorded before moving the plan out
  of `active/`.
