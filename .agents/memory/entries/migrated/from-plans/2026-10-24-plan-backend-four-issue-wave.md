---
schema_version: 1
id: "MEM-PLAN-backend-four-issue-wave-5bcc8ede49ec"
title: "Plan archive: Backend Four Issue Wave"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, backend-four-issue-wave]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "5bcc8ede49ece7866ee0ea67a3839c9713f2d59bd37e61a9bdae0483b290f0d7"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/backend-four-issue-wave/PLAN.md"
  - "docs/plans/active/backend-four-issue-wave/FINDINGS.md"
  - "docs/plans/active/backend-four-issue-wave/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/backend-four-issue-wave/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Backend four-issue wave

State: PLANNING. Baseline: bceb735727f81180dad678defee1000bc547d97b. Owner explicitly authorizes four more issues, commit/push/individual closure; expected144→140 if no concurrent backlog changes. Work directly on main; preserve open PR ownership.

## Analysis (L2; L3 for credential/DDL boundaries)

[verified] #499 accepts malformed function references and pastes bare function names without parentheses. #508 compares dependency names with LIKE and contains a suspect regclass catalog-field access; actual failure belongs to a real PostgreSQL probe. #452 interpolates credentials/database into a URL before SQLx parsing. [unknown] #486 may already be handled by csv-core BOM recognition: reproduce with the public source/transfer path before editing. No selected issue overlaps the thirteen open PR titles/bodies inspected.

Constraints: no dependencies, no raw SQL identifier/literal concatenation, no changes to TLS policy/pool limits/timeout, no UI redesign or external native state. Consumers are existing core builder→Postgres service, dependency API→Postgres connector, MySQL connect/test connection, and streaming delimited transfer.

## Alternatives / decision

Correctness/security (weight5), minimal blast radius(4), maintainability(3). Choose existing identifier validation/quoting plus a no-argument function grammar; exact namespace/relation catalog predicates with bound names; native MySqlConnectOptions field setters; leave BOM production code unchanged if actual public transfer already succeeds. Their alternatives—raw suffix concatenation, escaped LIKE, percent-encoded credential URLs, unconditional second BOM stripper—are superficially smaller but retain invalid grammar/identity/parse ambiguity or duplicate parser work. Doing nothing is acceptable only after runtime non-reproduction, not source speculation. No generic parser/options abstraction.

L3 pre-mortem: quoting can change function case semantics, dependency display can vary with search_path, TLS settings could silently weaken, and a BOM stripper could remove actual data. Mitigation: preserve PostgreSQL unquoted-name folding; reject arguments/malformed components; compare catalog identity independent of display; retain all four SSL mappings; exercise initial/embedded BOM and no-header CSV/TSV cases. The options-builder recommendation changes only if installed SQLx lacks field setters or changes TLS defaults; verify installed source. Schema/role fixtures are disposable only, no production mutation. Bias audit: do not accept issue-proposed fixes or close to hit a number without reproduction/evidence.

## MUST / scope

- #499: bare/name() and schema.name/schema.name() produce valid zero-argument event-trigger invocation; malformed references rejected. Actual PG create, fire and introspect prove it.
- #508: exact requested relation dependencies across active/other schemas, wildcard/quoted identifiers, same-named decoys and missing relation. Actual PG query/API, not SQL-string assertion.
- #452: exact reserved/Unicode credential/database fields reach actual MySQL connect and test_connection; wrong password still fails, TLS mode mapping unchanged. Audit analogous PG/SQLServer setup without unrelated edits.
- #486: actual BOM CSV/TSV source headers, preview, headerless first row and streamed transfer preserve intended data. Fix only a reproduced defect; otherwise document not-planned/non-reproduction.

SHOULD: isolated consumer-visible regressions. COULD: shared fixture support only if existing patterns suffice. NOT NOW: #400/#441, unrelated PRs, UI capture changes, broader credential policies or routine overload resolution. Any new business/security posture decision is escalated rather than silently chosen.

## Verification / ownership

Parent: baseline export/probes, #486, disposable PG/MySQL fixtures, all gates/runtime, shared changelog/status, review/publish/close. PgBuilderDependencies: #499/#508 source/adjacent tests and issue findings. MysqlOptions: #452 source/tests and findings. Children skip builds/lint/tests/formatters mid-flight. Parent runs baseline probes against exact export and candidate probes after stable edits, meaningful permanent regressions, fmt/check/owned Clippy/workspace tests and locked release build. Provider matrix: PG-only objects unsupported on SQLite, MySQL options do not change PG/SQLite, CSV transfer verified independently using real files plus SQLite import. No UI rendering source changes intended.


## Source: `FINDINGS.md`

# Findings

Source baseline bceb735727f81180dad678defee1000bc547d97b; source alone is not runtime evidence. Initial contracts/risks: PLAN.md. Issue-specific findings are appended by the assigned implementer and runtime outcomes by parent.


## Source: `VERIFICATION.md`

# Verification

Baseline bceb735727f81180dad678defee1000bc547d97b. Initial144 open; selected #499/#508/#452/#486 have no matching open implementation PR. No test/provider pass claimed yet; append actual commands/outcomes, distinguishing baseline and candidate.

## Parent baseline execution

Exact source export: bceb735727f81180dad678defee1000bc547d97b (no worktree). Identical standalone Rust public-API probe projects use only path dependencies switched between export/main; `cargo run --offline --manifest-path /tmp/db-pro-wave140-probes/baseline/Cargo.toml -- <case>`.

| Case | Observed baseline result | Exit |
|---|---|---|
| mysql / #452 | ConnectionFailed: MySQL connect failed: error with configuration: empty host; fixture credential delimiters were parsed as URL structure | 1 |
| event / #499 | Confirmed public service create fails QuerySyntaxAt: syntax error near semicolon, position122 | 1 |
| deps / #508 | Public connector Internal: column notation .relnamespace applied to type regclass, not a composite type | 1 |
| csv / #486 | Public preview/stream/SQLite import PASS8 cases, exact header/name mapping and data semantics | 0 |

The first throwaway compilation used SqliteConnector instead of actual SQLiteConnector (E0432); corrected both probes before execution. This harness typo is not a product RED result. The dependency oracle queries the provider's native identity for the known enum type rather than pinning its numeric OID/display spelling.

## Fixture evidence

- PostgreSQL18.6, loopback25432, disposable dbpro_smoke; current_user dbpro, default search_path `$user, public`. No existing application service used.
- MySQL8.4.11, loopback25433. Initial attempted `--skip-ssl` fixture exited because MySQL8.4 removed that flag; corrected via documented `--tls-version=`. Server tls_version and session Ssl_cipher observed empty. Source: https://dev.mysql.com/doc/refman/8.4/en/encrypted-connection-protocols-ciphers.html .
- Provisioned local-only reserved/Unicode fixture user `u@:#/?%Việt`, database `db#@:?%Việt`, account host `%`; no credential URL is used for candidate tests. Passwords are disposable fixture-only, not production secrets.
- Adjacent BOM import consumer test ran on unchanged production code:1 passed/0 failed/0 ignored, exit0. No duplicate BOM implementation added.

