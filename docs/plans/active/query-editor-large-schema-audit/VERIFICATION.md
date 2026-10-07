# Verification

Baseline SHA: 3dd988e2ff06ac71942c7ec331acf66389d3c5cb. Audited paths verified unchanged with git diff --quiet HEAD -- <manifest source files>, exit 0. Manifest fingerprints source, cached optimized rlib, Rust probe, binary and results. No product code edited by this audit.

Executable loop ran twice using actual public SchemaCompletionProvider, SchemaSymbolIndex and UiQueryResult clone. Synthetic ASCII fixtures: 50/500/1000 tables, 40 columns each, short qualified/unscoped/table-list SQL. Final run adds 66KB document and 78.125MiB result payload. Provider timings: 15 iterations, median and nearest-rank p95; clone: 10 iterations upper median and maximum. Index/long-doc are single timings. Process /usr/bin/time -l records peak RSS of combined probe.

Reproduce from repository root (existing optimized library required):

```sh
probe_lib=$(ls -t target/release/deps/libdb_pro_ui-*.rlib | head -n 1)
rustc --edition 2021 -O docs/plans/active/query-editor-large-schema-audit/evidence/probe.rs -L dependency=target/release/deps --extern "db_pro_ui=$probe_lib" -o /tmp/db-pro-query-audit-probe
/usr/bin/time -l /tmp/db-pro-query-audit-probe
```

Panic is caught so all measurements complete; stdout utf8_completion_panics=true is the red-capable failure signal. Stderr pinpoints schema_completion.rs:1096 and invalid UTF-8 boundary. This reproduces a real completion panic, not the company startup incident. No native screenshot/frame-time/mouse/keyboard proof, no provider SQL run, no OOM proof.

Audit-only: fmt/workspace test/check/clippy/native rebuild and generic performance scan skipped; no product changes and no generic scan answers these focused questions. Existing workspace failure history is outside this audit. One direct optimized probe compiled and executed; raw command output retained. Review is self-review, independent approval n/a. No commit/PR/merge/push or CI run.
