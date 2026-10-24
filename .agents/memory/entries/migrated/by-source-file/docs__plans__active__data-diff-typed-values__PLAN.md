---
schema_version: 1
id: "MEM-SRC-27d18863aa3939b8"
title: "Source mirror: docs/plans/active/data-diff-typed-values/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/data-diff-typed-values/PLAN.md"
source_sha256: "27d18863aa3939b80cd26f802a7e4bd1c37c651b1975e525e0dd519d60a3b233"
mirror_body_sha256: "27d18863aa3939b80cd26f802a7e4bd1c37c651b1975e525e0dd519d60a3b233"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Typed data-diff values — #457 / #485

State: REVIEW. Baseline: `76b120724a7542de8b587a002811ddacad28156a`. Parent independently executed real SQLite/PostgreSQL round trips and integrated regression gates; source review ACCEPT. Publication belongs to parent.

## Requirements and context
At the baseline, `crates/core/src/application/data_diff.rs` converts SQL NULL and text `∅` to the same string, joins composite keys with U+001F, splits those keys for WHERE, and decodes `∅` as NULL when generating literals. This can hide changes, reject distinct keys, and generate predicates that target the wrong row or no row. Severity: P1 (incorrect synchronization preview).

## Design and scope
Keep borrowed typed values and structural key tuples inside DataDiffService. Convert only at the existing public string DTO boundary. INSERT and key predicates use typed values; UPDATE uses correctly nullable exact scalar change values (None for real NULL, Some for text), never sentinel/delimiter decoding. Preserve quoted identifiers and escaped literals, precise scalar text, sampling bounds, genuine duplicate rejection and public DTO field types. No SQL-builder/schema-DDL/port/dependency changes.

## Definition of done
- NULL differs from text `∅` in rows and keys; NULL predicates use IS NULL.
- Literal text `∅` survives INSERT and UPDATE.
- Distinct composite tuples containing U+001F compare independently and produce exact predicates.
- Existing added/removed/changed/equal behavior and genuine duplicate detection remain covered.
- Adjacent tests and independent real SQLite/PostgreSQL public-service smoke recipes are handed to Main. Execution and provider evidence must be recorded before closure.

## Tasks
1. Trace callers and existing tests, establish reproducible fixtures.
2. Add regression tests, then replace lossy internal representations and SQL inputs.
3. Record source findings, exact file inventory, selectors, independent provider recipes, and unexecuted gates.
