---
schema_version: 1
id: "MEM-PLAN-query-editor-large-schema-audit-489d49d5de71"
title: "Plan archive: Query Editor Large Schema Audit"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, query-editor-large-schema-audit]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "489d49d5de710654c3e0613b21ba456fed4b16de67badfc5d7d8f3c95d4fbe99"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/query-editor-large-schema-audit/PLAN.md"
  - "docs/plans/active/query-editor-large-schema-audit/FINDINGS.md"
  - "docs/plans/active/query-editor-large-schema-audit/VERIFICATION.md"
  - "docs/plans/active/query-editor-large-schema-audit/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/query-editor-large-schema-audit/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Query Editor large-schema audit

State: PLANNING. Audit lane Done; implementation not requested in this audit. Date 2026-10-06. Baseline/current HEAD: `3dd988e2ff06ac71942c7ec331acf66389d3c5cb`, main. Existing dirty Grid, Select, Welcome, runtime invalidation and macOS menu changes preserved.

Owner reports PostgreSQL company DB with over 500 tables: load crash and lag across Query features, including column suggestions. Scope: inspect connect/schema summary/UI reducer/completion/popup/result render path; collect executable synthetic performance and panic evidence. No company DB access, no credential retrieval, no production code changes.

Architecture: PostgreSQL metadata queries run in runtime/infrastructure. Native UiEvent reducer builds symbol index synchronously. Query Editor resolves completion on UI thread and paints floating popup; output pane receives a cloned UiQueryResult. Provider-neutral UiSchemaSummary feeds editor.

Deliverable: severity findings, exact source SHA/anchors, fixture measurements, repro script, supported/pending provider matrix, prioritized minimal remediation. A full company crash cause requires actual crash/backtrace and reproduction scenario; synthetic failure is not equivalent to reproducing the reported crash.


## Source: `FINDINGS.md`

# Findings — Query Editor at large schema sizes

Exact source SHA for every anchor below: `3dd988e2ff06ac71942c7ec331acf66389d3c5cb`. Audited implementation files are unchanged from that HEAD, verified with git diff --quiet. Source hashes and optimized library fingerprint: evidence/manifest.json. Other checkout modifications are not attributed to this audit.

## [P1] Completion panics on UTF-8 identifiers

`crates/ui/src/query/schema_completion.rs:1094`: extract_word_prefix slices text[end-1..end] and decrements byte offsets. `SELECT tên` panics inside ê. The executable probe calls actual SchemaCompletionProvider::provide and catches the panic; two runs reproduced it. `query_editor_surface_view.rs:302` calls the provider synchronously during completion; typing/IME while popup is open refreshes it, and manual completion is reachable too. Not tied to PostgreSQL or 500 tables. This is a proven crash-capable code path, NOT confirmation of the owner's actual startup crash.

Minimal fix: walk char_indices backwards and slice only character boundaries; check qualifier and cursor/replacement byte-offset boundaries. Add regressions for Vietnamese, accented identifiers, multibyte separators and IME edits. No such fix is included here.

## [P1] Completion pipeline expands/renders an unbounded suggestion list

`schema_completion.rs:555`: fallback pushes every matching column across all table_details (the comment says active schema, but code does not filter it). No maximum item count. `schema_completion.rs:1076`: full sorting repeatedly lowercases labels inside comparator. `query_completion_popup_view.rs:98-103`: ScrollArea max_height only constrains viewport; it still runs the row closure for every item, with badge/label/detail widgets. No show_rows or equivalent virtualization.

Actual optimized provider: 500 × 40 = 20,000 column suggestions, median 26.365 ms, nearest-rank p95 27.731 ms across 15 calls. At 1000 tables: 40,000 items, median 54.445 ms, p95 57.430 ms. These generation times alone exceed a 16.7 ms frame budget, before popup layout. Popup frame-time itself was not measured: source confirms O(total suggestions) widget work, not a measured FPS claim.

Counter-evidence: qualified `t.col` with one referenced table returns 40 items at 0.028 ms median for 500 tables. Raw lookup of a scoped column is not slow in this fixture. Reported one-table lag may instead involve unscoped fallback, popup paint, document size or output-pane work; an actual trace is required to isolate it.

Minimal fix: semantic table/schema scope first; deduplicate by qualified identity, precompute normalized labels, bounded top-k ranking and virtualized popup. Preserve hidden-column access/search and keyboard selection; do not merely truncate correctness away.

## [P2] Whole query result cloned on every output-pane frame

`crates/ui/src/query_view.rs:269`: active_result().cloned() precedes draw_output_pane each frame. This deep-copies strings/cells, independent of which Query interaction caused repaint. Real UiQueryResult::clone fixture: 500 rows × 20 columns × 8KiB text = 78.125 MiB payload; upper median of 10 clones 8.445 ms, max 17.963 ms. This is clone time only; allocation/destruction/render total is higher. Probe peak RSS 266,010,624 bytes covers the combined harness, not the app or this clone alone. No OOM reproduced.

Minimal fix: borrow/split disjoint state or share immutable result storage; clone only small view state. Verify Save/refresh/selection and old-result lifetime before changing ownership.

## [P2] Schema index built synchronously on UI load transition

`crates/ui/src/schema_events.rs:57` calls SchemaSymbolIndex::build before state replacement. `query/intelligence.rs:120-135` stores column help under bare, table-qualified and schema-qualified keys, cloning strings into separate records. Single-build fixture timings: 24.771 ms for 500 × 40; 53.117 ms for 1000 × 40. First probe had 27.582 / 89.260 ms, indicating cold/scheduling variation; do not read single values as stable percentile budgets. This can visibly pause load; no load crash reproduced.

Minimal fix: build once per schema generation off the UI thread, publish atomically under connection/request generation checks; share symbol records rather than duplicating strings. Avoid stale indices from a different connection.

## [P2] Runtime schema summary performs repeated global metadata scans

`crates/runtime/src/api.rs:470-484`: for each table, filter all columns and all foreign keys. Complexity O(T×C + T×FK), instead of one partition pass. 500 tables × 20,000 columns implies 10 million column predicate checks (algorithmic count, not measured latency). Runs on worker side; therefore it cannot alone establish an egui-thread crash.

PostgreSQL provider `crates/infrastructure/src/postgres/introspect.rs:54` already runs ten bulk metadata queries with tokio::join!. No evidence that it sends one metadata SQL query per table. Live PostgreSQL latency, permissions, schema-change races, provider error output and cache behavior remain unmeasured.

Minimal fix: group metadata by (schema, table) once, preserve ordering and qualified identity. Benchmark summary assembly separately from database query time.

## Additional evidence and limits

- 66,027-byte SQL buffer: qualified completion still yields 40 items but takes 9.404 ms in one run. `schema_completion.rs:80-85` scans/parses full document before qualified dispatch; cache semantic context by buffer version and avoid unrelated statements. This is a contributor, not another asserted P1.
- Main editor renderer already virtualizes visible lines and uses CachedSqlTokens; `editor/renderer.rs:784` is not a full-line rendering loop. Do not replace that correctly scoped path on speculation.
- PostgreSQL is owner's affected provider; engine/UTF-8/result-copy findings are shared with SQLite, but live SQLite/PostgreSQL tests were not run.
- No matching DB Pro crash report was found in this host's macOS DiagnosticReports. Exit-vs-hang and exact startup failing frame remain unresolved; no credential or company data was read.
- Host free disk around 144 MiB during probe. Existing optimized rlib was reused to avoid a large build; benchmarks are local CPU microprobes, not clean-room frame profiling.

## Remediation order and learning pass

1. Eliminate proven UTF-8 panic with a regression capable of going red.
2. Bound producer and virtualize popup together; keep scoped completion correct.
3. Remove result-copy work from paint path.
4. Move shared schema-index construction off UI and partition metadata once.
5. Collect actual company crash/backtrace and native frame/memory traces before claiming the load crash fixed.

Reusable lessons: max_height is not virtualization; schema-wide completion budgets should cover wide column counts, not just table names; rendering must not deep-copy payloads; scoped lookup can be cheap while the surrounding frame remains expensive. Recorded here; no global memory write authorized.


## Source: `VERIFICATION.md`

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


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — Query Editor large-schema audit

## 1. Claim

Agent Codex root; research/performance audit lane, task Done. Feature state PLANNING for proposed remediation. Baseline exact SHA 3dd988e2ff06ac71942c7ec331acf66389d3c5cb. Main; PR n/a. Scope: owner's PostgreSQL >500-table crash/Query lag audit. Production source changes: none. Existing dirty checkout preserved.

## 2. Progress checkpoint

Completed source trace, public-path synthetic measurements, twice reproduced Unicode panic, evidence fingerprints and recommendations. Actual company startup crash, native popup frame-time, provider latency and OOM evidence pending. Task Done means audit report delivered, not bug fixed.

## 3. Implementation handoff / review request

No implementation. Five findings: 2 P1 (UTF-8 panic and unbounded completion pipeline), 3 P2 (result-frame clone, UI index build, repeated metadata scans). Proposed minimal fixes and acceptance boundaries in FINDINGS.md. Keep provider-qualified identities, request generations and editor semantics. No credentials, schema dump or company access collected.

## 4. Review outcome

Self-review verdict BLOCK for claiming large-schema Query readiness or crash resolution until P1 paths are fixed. Introduced findings by this audit: P0=0 / P1=0 / P2=0 (no product patch). Audited baseline defects/risks: P0=0 / P1=2 / P2=3. Exact committed behavior is asserted only on audited unchanged paths at SHA above; current cached library/source fingerprints in manifest. Skipped gates are listed in VERIFICATION.md, not marked passed. Independent review/CI n/a.

## 5. Research / audit handoff

Primary evidence: current local code, optimized public-path probe, source equality to exact SHA, measurements.log and manifest.json. Source date 2026-10-06. PostgreSQL user-reported affected; shared frontend findings also apply to SQLite paths, live tests pending independently. No remote PR coverage check or external web research needed for this local audit. Lessons recorded locally; global memory unchanged.

## 6. Tổng kết (Vietnamese summary)

Đã tái hiện panic UTF-8 và đo autocomplete không scope vượt frame budget ở 500–1000 bảng. Popup chưa virtualize, Query clone result mỗi frame, schema index xây trên UI thread và mapper quét metadata nhiều lần. Completion có alias trên SQL ngắn vẫn nhanh; chưa có bằng chứng chốt nguyên nhân sập lúc load DB công ty. Audit chưa sửa product code; ưu tiên sửa panic và pipeline completion trước, rồi ownership/index/metadata và runtime trace thực tế.

