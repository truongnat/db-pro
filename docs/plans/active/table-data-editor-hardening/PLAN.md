# Table Data Editor Hardening

## Objective

Make staged table mutations safe under primary-key edits and concurrent row
changes, while keeping typed filtering/sorting and grid state independent from
the renderer. The current implementation prioritizes row identity, ChangeSet
correctness, conflict recovery, and render-coordinate performance.

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
- Keep table sort clauses typed and ordered, including Shift+click multi-sort.
- Persist grid width/order/hidden-column preferences per connection, schema,
  and table.
- Validate typed table filters before dispatching a query; support multiple AND
  predicates, editable filter chips, pagination reset, and server-side sorts.
- Provide compact staged-change review, cell diff/revert actions, expanded
  JSON/long-text editing, NULL/binary-safe rendering, and pagination controls.
- Avoid linear coordinate lookup in visible-cell rendering.

## Out of scope for this slice

- Binary editing.
- Full metadata expansion for provider-specific indexes and foreign-key
  actions (the existing structure surface remains unchanged in this slice).
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
