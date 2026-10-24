---
schema_version: 1
id: "MEM-SRC-f8892e0836ac03c1"
title: "Source mirror: docs/plans/active/postgres-typed-query-description/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/postgres-typed-query-description/CHECKLIST.md"
source_sha256: "f8892e0836ac03c12ac48c21e7b40bb609d507b9a7464d12d1e06e2f426dbba1"
mirror_body_sha256: "f8892e0836ac03c12ac48c21e7b40bb609d507b9a7464d12d1e06e2f426dbba1"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #533 checklist

Baseline: 76b120724a7542de8b587a002811ddacad28156a. Published: 6f94372dd5caba22e525fbc1aff918b936e0491d. State: REVIEW due inherited global gate, not missing provider proof.

- [x] Preserve parent-observed inherited P1 command/result separately from candidate evidence.
- [x] Inspect installed SQLx 0.8.6 argument encoding, SQL-only cache, prepare_with and describe behavior.
- [x] Keep one parameter conversion policy and ordinary statement caching.
- [x] Capture actual converted argument types opt-in; prepare_with before describe and streaming; retain nullable and empty-result metadata without post-stream draining.
- [x] Keep bounded streaming, backend guard, search_path restoration and transaction lease scopes.
- [x] Inspect all three describe callsites: the other two execute non-parameterized transaction statements; no typed arguments are supplied there.
- [x] Inspect execute / parameterized transaction / explain: bind arguments before preparation, no pre-describe; unchanged.
- [x] Add five permanent real-PostgreSQL integration tests, including real TableDataService INTEGER composite-PK lookup, repeated matching/missing values, BIGINT precision, empty metadata, execute/explain, row limits 0/2, connection reuse, fast capped rows before a slow tail and first-execution NULL contextual inference.
- [x] Add binder unit coverage for integer/date/timestamp/time/offset-time wire types and unspecified OID 0 for domain NULL.
- [x] Parent reran original public DbProRuntime INTEGER composite-PK probe unchanged; PASS.
- [x] Final fmt, 5 actual-PG regressions, 16 mapper cases, owned Clippy and full workspace tests PASS; inherited global lint failure reported.
- [x] Independent reviewer final ACCEPT after bounded-tail and NULL-type corrections; no introduced open findings.
- [x] Published exact candidate SHA above; GitHub close #533 completed succeeded with real provider/public-service evidence.

No build/test/lint/formatter/smoke was executed by this worker; this is an implementation checklist, not a passing test report. Different binary parameter types for identical SQL remain the inherited SQLx cache limitation.
