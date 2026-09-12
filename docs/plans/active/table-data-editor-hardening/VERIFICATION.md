# Verification

## Baseline

- `cargo test -p db-pro-core table_data_service --lib -- --nocapture`: 20
  passed at baseline; focused suite later passed 21.
- `cargo test -p db-pro-ui --lib`: 132 passed after stable identity, sort,
  conflict recovery, and grid-layout changes.

## Required evidence before completion

- Focused unit tests for original and composite PK identity, PK mutation,
  ChangeSet composition, conflicts, and invariant violations: PASS.
- `cargo check --workspace`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace`: PASS, including core 274 tests, UI 127 tests, and
  infrastructure integration 32 tests; 18 PostgreSQL tests ignored because no
  live PG fixture was configured.
- `cargo build --release --locked -p db-pro-native`: PASS.
- `cargo fmt --all -- --check`: PASS; `git diff --check`: PASS.
- Apply validation guard test: `cargo test -p db-pro-ui --lib
  apply_is_blocked_while_a_validation_error_exists -- --nocapture`: PASS.
- `bash .skills/perf-audit/scripts/perf-scan.sh`: PASS with one warning from
  the audit's non-`-D warnings` clippy invocation; the required strict clippy
  gate is recorded separately below.
- `cargo test -p db-pro-core table_data_service --lib`: 21 passed.
- SQLite mutation integration test: PASS.
- Live SQLite integration mutation test:
  `cargo test -p db-pro-infrastructure --test integration
  sqlite_parameterized_table_mutations_roll_back_and_reject_zero_rows
  -- --nocapture`: PASS.
- Native app smoke observation: release `db-pro-native` is running and the
  connected PostgreSQL Data Editor loaded `public.t_driver_group` (20 rows,
  primary-key column visible). This is normal-state evidence only; the
  computer-use session could not focus the custom egui grid/editor for a live
  cell mutation.
- Current completion build smoke: the freshly built release process started
  with the configured PostgreSQL connection and completed schema introspection
  for 68 tables. The desktop accessibility snapshot timed out, so no new
  screenshot or live cell-edit pass is claimed for this continuation.
- PostgreSQL runtime mutation flow with a concurrent delete/update.
- SQLite runtime mutation flow with a concurrent delete/update.
- Native UI screenshots or recording at 1280x800, 1440x900, and 1920x1080
  covering normal, loading, error, and empty states.
- Full Rust quality gates and `cargo build --release --locked -p db-pro-native`.
