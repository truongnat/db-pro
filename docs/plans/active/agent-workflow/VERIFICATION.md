# Verification

## Initial state

- Branch: `feature/agent-workflow`
- Base: `main@739f79c`
- Implementation verification: core foundation checks pass.

## Automated evidence

- `cargo fmt --all` — PASS.
- `cargo test -p db-pro-core --quiet` — PASS (283 tests).
- `cargo clippy -p db-pro-core --all-targets -- -D warnings` — PASS.
- `cargo fmt --all -- --check` — PASS.
- `cargo check --workspace` — PASS.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo build --release --locked -p db-pro-native` — PASS.
- `cargo test --workspace --quiet` — PASS (283 core, 62 infrastructure,
  32 infrastructure integration, 9 native, 7 runtime, 21 legacy-host, and
  231 UI tests; provider-dependent PostgreSQL/SSH cases remain ignored).
- `cc-scan.py` / `arch-scan.py` — NOT AVAILABLE in this checkout or installed
  skill paths; no clean-code/architecture scan result is claimed.

The tests cover stale document/version rejection, UTF-8 patch boundaries,
confirmation-gated mutation/unknown SQL, bounded schema retrieval, bounded
result samples, and serialized context limits.

## Required evidence

- Core unit tests for patch safety, bounded context, result summaries, and
  execution permissions.
- Workspace format/check/clippy/test gates after implementation.
- PostgreSQL and SQLite runtime verification for agent inspection/execution.
- Native UI evidence for patch preview, confirmation, cancellation, and
  background-tab routing.
