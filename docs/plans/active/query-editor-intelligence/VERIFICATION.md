# Verification

Source evidence recorded on 2026-09-13:

- Baseline: `1d3a278 feat(query): harden SQL prediction quality and UX`.
- Native translator now routes `RequestSqlPrediction` and `CancelSqlPrediction` to the runtime worker.
- Query execution, explain state, prediction request state, and Query Workspace output selection are
  stored per `QueryDocument`; the global query request slot was removed from UI routing.
- Prediction overlap and partial acceptance only operate at UTF-8 character boundaries. Manual
  prediction can replace the current partial token atomically and undo restores the original text.
- Completion now covers explicit/simple CTE output columns, UPDATE target columns, and simple
  subquery aliases with deterministic context ordering.

Source evidence is not runtime evidence.

Automated evidence recorded on 2026-09-13:

- `cargo test --workspace` — PASS, including 203 UI tests and the workspace crate suites.
- `cargo fmt --all -- --check` — PASS.
- `cargo check --workspace` — PASS.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo build --release --locked -p db-pro-native` — PASS.
- `bash .skills/perf-audit/scripts/perf-scan.sh` — PASS; native binary 21.5 MB, no warnings.
- `git diff --check` — PASS.

Prediction quality coverage now includes deterministic fingerprints with sorted alias maps,
bounded before/after/CTE context, short-lived document-local cache, code-fence/explanation
normalization, prefix/suffix overlap removal, and non-empty replacement ranges for manual token
replacement. The native runtime worker logs provider latency using request/document metadata only.

- Focused tests cover translator routing, stale document version rejection, debounce/deduplication,
  current-statement context, UTF-8 overlap, atomic replacement acceptance, cooldown handling,
  completion context, and concurrent per-document query/output state.

Runtime evidence collected in this turn:

- Release binary started through the native runtime and connected to the configured PostgreSQL
  connection; schema introspection completed with 68 tables.
- Orca captured the DB Pro Query Workspace at a 1838×1049 window (2× screenshot scale), including
  the editor, diagnostics, empty Results state, and query actions menu.
- Text injection through the available accessibility/synthetic path was unverified by egui, so this
  is a native smoke check, not a prediction acceptance E2E pass.

Live provider evidence remains pending. The shell had no `GROQ_API_KEY` or `OPENAI_API_KEY`, and no
credential was read from Keychain. Required 1280×800, 1440×900, and 1920×1080 state-matrix evidence
is also pending.
