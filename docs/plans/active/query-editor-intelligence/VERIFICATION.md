# Verification

Source evidence recorded on 2026-09-13:

- Baseline: `8c688d9 feat(query): add editor delimiter pairing UX`, with the completion and
  delimiter hardening recorded below.
- Native translator now routes `RequestSqlPrediction` and `CancelSqlPrediction` to the runtime worker.
- Query execution, explain state, prediction request state, and Query Workspace output selection are
  stored per `QueryDocument`; the global query request slot was removed from UI routing.
- Prediction overlap and partial acceptance only operate at UTF-8 character boundaries. Manual
  prediction can replace the current partial token atomically and undo restores the original text.
- Completion now covers explicit/simple CTE output columns, UPDATE target columns, and simple
  subquery aliases with deterministic context ordering.
- Completion now recognizes `JOIN ... ON`, ranks matching aliased FK predicates (including
  composite keys), ranks SELECT aliases/non-aggregate projections for ORDER BY/GROUP BY, and
  omits INSERT columns already present in the target column list.
- SQL formatting is conservative and safe for incomplete input: it preserves comments, quoted
  text, identifiers, and PostgreSQL dollar-quoted bodies while normalizing common clause
  boundaries. Selection/document formatting uses a single editor undo snapshot and is available
  from Query actions and Cmd/Ctrl+Shift+F.
- The editor now performs SQL-aware `()`, `[]`, `'`, and `"` pairing, wraps selections atomically,
  skips an already-present closing delimiter, and highlights the delimiter pair near the caret.
  Unmatched square brackets are surfaced as ranged diagnostics while literals/comments/dollar
  quotes are ignored by the structural matcher.
- Mixed structural delimiters now report the closing delimiter and expected pair (for example,
  `([)]` reports `): expected ]`) instead of silently treating the mismatch as a generic unmatched
  character. Escaped PostgreSQL E-strings and quoted identifiers are covered by the matcher tests.
- GROUP BY projection analysis now excludes only known aggregate calls, including nested aggregate
  calls, while retaining scalar function expressions such as `LOWER(name)` as groupable output.
- Diagnostic sources are now typed as Parser, Delimiter, or Database. Query documents retain the
  executing SQL/range/version and last executed range; database failures attach a separate
  database diagnostic without allowing the next parser refresh to erase it. PostgreSQL driver
  positions travel as structured metadata through infrastructure, runtime, and native UI event
  translation, then map from one-based character positions to UTF-8-safe editor byte ranges;
  failures without a position use the statement range.
- GROUP BY aggregate detection is dialect-aware: PostgreSQL includes ARRAY_AGG, STRING_AGG,
  BOOL_AND, BOOL_OR, JSON_AGG, JSONB_AGG, and EVERY; SQLite includes GROUP_CONCAT and TOTAL.

Source evidence is not runtime evidence.

Automated evidence recorded on 2026-09-13:

- `cargo test --workspace` — PASS, including 224 UI tests and the workspace crate suites.
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

- Focused tests cover translator routing, structured database failure position/code propagation,
  stale document version rejection, debounce/deduplication,
  current-statement context, UTF-8 overlap, atomic replacement acceptance, cooldown handling,
  completion context, FK JOIN suggestions, INSERT/ORDER/GROUP completion ranking, conservative
  SQL formatting, bracket matching/diagnostics (including mixed mismatches and auto-pair deletion),
  aggregate-aware GROUP BY ranking, PostgreSQL error-position mapping over UTF-8, database failure
  diagnostic routing, and concurrent per-document query/output state.

The clean-code scan still reports legacy oversized renderer/query functions and existing clone/cast
heuristics; this focused change introduces no new unwrap/expect or clippy warning. Live provider and
the required multi-viewport native UI evidence remain the release-phase gaps.

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
