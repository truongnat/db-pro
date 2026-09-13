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
- `cargo test --workspace`: PASS, including core 274 tests, UI 138 tests, and
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
- Continuation UI regression suite: `cargo test -p db-pro-ui`: 138 passed.
- `cargo fmt --all -- --check`: PASS.
- `cargo check --workspace`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace`: PASS; all workspace suites passed, with 18
  PostgreSQL fixture tests ignored because no live fixture was configured.
- `cargo build --release --locked -p db-pro-native`: PASS.
- `bash .skills/perf-audit/scripts/perf-scan.sh`: PASS; release binary 21.0MB,
  workspace check/clippy passed, zero audit warnings.
- `bash .skills/clean-code/scripts/clean-code-scan.sh --diff`: completed with
  existing large-grid-file heuristic findings; no unwrap/println/swallowed
  Result findings in changed production paths.
- Continuation source evidence: typed operator gating, multiple editable filter
  chips, unknown-total Next behavior, pending-change review, expanded
  JSON/long-text editor, header Add Filter, separator auto-size, and persisted
  layout normalization are implemented and covered by focused tests where
  applicable.
- Conflict reload source evidence: Reload Row dispatches one-row equality
  filters from the original RowIdentity; returned values merge only into the
  matching visible row, while staged values and conflict state remain intact.
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
- Runtime continuation: the `db-pro-native` window was present at 1838×1049,
  but `orca computer get-app-state --restore-window` and its no-screenshot
  variant both timed out; no UI screenshot or interaction is claimed from this
  run.

## Latest continuation

- Stable named layout persistence: removed columns are dropped, added columns
  append with defaults, and renamed columns do not inherit old layout state;
  legacy index layouts are retained only when the schema shape matches exactly.
- Row identity cache is built once per loaded table result and reused by the
  grid; targeted row reload matches directly against a PK column-index map
  without cloning `UiQueryResult`, its columns, or candidate rows.
- Conflict retry now filters the Apply batch to the related mutation target;
  unrelated staged rows remain untouched.
- Pending Changes is grouped by original RowIdentity and temporary insert ID.
- PostgreSQL introspection now reads catalog index metadata (method, primary,
  unique, key/include columns, predicate, definition), column identity/
  generated/collation metadata, and FK action/deferrability metadata. SQLite
  exposes equivalent supported metadata and PRAGMA FK actions.
- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace`: PASS; PostgreSQL fixture tests remain ignored
  without a configured live fixture.
- `cargo build --release --locked -p db-pro-native`: PASS.
- `cargo check -p db-pro-ui --benches`: PASS.
- `bash .skills/perf-audit/scripts/perf-scan.sh`: PASS; release binary 21.1MB.
- `git diff --check`: PASS.
- PostgreSQL runtime mutation flow with a concurrent delete/update.
- SQLite runtime mutation flow with a concurrent delete/update.
- Native UI screenshots or recording at 1280x800, 1440x900, and 1920x1080
  covering normal, loading, error, and empty states.
- Full Rust quality gates and `cargo build --release --locked -p db-pro-native`.
