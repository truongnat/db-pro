---
schema_version: 1
id: "MEM-SRC-3271cfaca601497c"
title: "Source mirror: docs/plans/active/data-diff-typed-values/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/data-diff-typed-values/FINDINGS.md"
source_sha256: "3271cfaca601497c5cedad4409e0cb7c7cf02344f4f1660e82cae9bf0aa0eecb"
mirror_body_sha256: "3271cfaca601497c5cedad4409e0cb7c7cf02344f4f1660e82cae9bf0aa0eecb"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — #457 / #485

All baseline source assertions below refer to `76b120724a7542de8b587a002811ddacad28156a`; revised behavior describes the uncommitted implementation based on that SHA, not observed execution.

| Severity | Baseline evidence | Consequence / decision |
|---|---|---|
| P1 | `crates/core/src/application/data_diff.rs:275-290,310-320,331-345` converts NULL and text `∅` to identical strings before comparison. | A NULL/text change can be reported equal; key samples can collapse or be rejected as duplicates. Retain typed values, map only actual NULL to DTO `None`. |
| P1 | `crates/core/src/application/data_diff.rs:266-279,310-314` joins/splits U+001F keys, emits `= NULL`, and interprets text `∅` as SQL NULL. | Distinct composite tuples can collide, WHERE can lose tuple boundaries, and literal text becomes NULL. Use structural tuples for identity and predicates; NULL uses IS NULL. |
| P2 | `crates/core/src/application/data_diff.rs:283-290` uses Debug for decimal/temporal/JSON values. | Precision-sensitive strings must not be parsed as f64 or reconstructed from Debug. The replacement retains exact scalar payloads and uses them for comparison/SQL. |

## Callers and compatibility
At baseline `76b120724a7542de8b587a002811ddacad28156a`, `crates/runtime/src/api.rs:1151-1180` and `crates/runtime/src/worker.rs:2612-2619` delegate to the public service and forward its DTO. `crates/app/src/schema/schema_compare_view.rs:498-544` only displays row keys and SQL, replacing U+001F for presentation; it does not decode keys for mutation. `crates/core/src/domain/cross_connection.rs:49-95` defines string-valued DTO fields. No public field/type change is needed. Keys become escaped typed display strings; column-change NULL is `None`, text `∅` remains `Some("∅")`. No caller migration is necessary.

## Deliberate boundaries
- Baseline `76b120724a7542de8b587a002811ddacad28156a`, `crates/core/src/ports/dialect.rs:1-6`, offers no provider-specific literal/binary API. Main explicitly authorized retaining legacy binary preview spelling, with binary literal correctness **not addressed/not verified**. No claim of provider-correct binary synchronization is made.
- Provider-specific MySQL backslash modes, unsupported literal classes, sampling/pagination, capabilities, SQL builders, schema DDL and ports remain unchanged/out of scope. Quoted scalar SQL preserves the existing convention, doubles apostrophes and never reparses decimal/integer values through floating point.
- Float values retain their exact f64 bits for identity; strings/bytes borrow provider values. JSON uses its serialized payload, not a SQL sentinel or composite-key delimiter.
- No source-text/wiring tests, dependencies, inline test modules, commits, pushes or issue closures were added.

## Tooling and evidence
Semantic `find` failed because its judge backend rejected the account; literal symbol searches were used instead. Parent reported LSP unavailable, so literal navigation was retained. Tests and runtime gates were deliberately not executed in this lane; their status remains NOT VERIFIED until Main records actual results.
