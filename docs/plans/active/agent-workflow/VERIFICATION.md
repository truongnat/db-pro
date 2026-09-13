# Verification

## Initial state

- Branch: `main`
- Base: `main@e5b035d`
- Implementation verification: core contracts, executor mapping, and runtime
  command/event boundary are covered by automated checks.

## Automated evidence

- `cargo fmt --all` — PASS.
- `cargo test -p db-pro-core --quiet` — PASS (290 tests).
- `cargo clippy -p db-pro-core --all-targets -- -D warnings` — PASS.
- `cargo fmt --all -- --check` — PASS.
- `cargo check --workspace` — PASS.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo build --release --locked -p db-pro-native` — PASS.
- `cargo test --workspace --quiet` — PASS (290 core, 62 infrastructure,
  32 infrastructure integration, 12 runtime, 231 UI tests; provider-dependent
  PostgreSQL/SSH cases remain ignored).
- `cc-scan.py` / `arch-scan.py` — NOT AVAILABLE in this checkout or installed
  skill paths; no clean-code/architecture scan result is claimed.

The tests cover stale document/version rejection, UTF-8 patch boundaries,
confirmation-gated mutation/unknown SQL, bounded schema retrieval, bounded
result samples, serialized context limits, document snapshot routing, and
executor UTF-8-safe output handling.

The workflow tests additionally cover mode/tool permissions, run-id validation,
stale tool requests, patch preview/confirmation, mutation confirmation,
rejection without ending the run, cancellation cleanup, and the invariant that
no synthetic run ID is created after a run finishes.

## Required evidence

- Core unit tests for patch safety, bounded context, result summaries, and
  execution permissions.
- Workspace format/check/clippy/test gates after implementation.
- PostgreSQL and SQLite runtime verification for agent inspection/execution.
- Native UI evidence for patch preview, confirmation, cancellation, and
  background-tab routing.
- Provider tool-call streaming and multi-step continuation.
