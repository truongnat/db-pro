# S3 — Schema Relations Runtime Verification

State: REVIEW

## Commands actually executed

None recorded yet for this branch.

The branch contains new tests, but their existence is source evidence only until a runner executes them.

## Tests added

### Rust

- `crates/infrastructure/tests/schema_relations_runtime_verification.rs`
  - creates a composite parent key
  - creates a composite child FK
  - introspects mapping identity/order
  - inserts a valid relation row
  - verifies an invalid relation row is rejected by SQLite FK enforcement

- `crates/core/src/application/schema_service.rs`
  - unit test verifies two mapping rows reconstruct as one ordered composite FK constraint

### Frontend

- `frontend/src/modules/schema/__tests__/foreign-key-groups.test.ts`
  - composite grouping/order
  - distinct constraint identity
  - cross-schema separation

## Provider matrix

| Provider | Automated | Live/runtime | Notes |
|---|---|---|---|
| PostgreSQL | PENDING | PENDING | source query preserves real name and ordinality; execution proof still required |
| SQLite | PENDING EXECUTION | PENDING | runtime integration test has been added |

## UI lifecycle

```text
DB foreign key
→ introspection
→ tableInfo
→ groupForeignKeys
→ Foreign Keys tab
→ target-table navigation
```

Status: PENDING runtime evidence.

## Quality gates required

```text
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets
cargo test --workspace

cd frontend
npm run typecheck
npm run lint
npm run format:check
npm run test
npm run build
```

No PASS claim is made until these commands actually run.

## Completion decision

Not completed. S3 is ready for automated/independent review after the branch is published as a PR, then moves to `RUNTIME_VERIFY` only after P0/P1 review closure.
