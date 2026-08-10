# S7 — Full Schema Regression

## Goal

S7 is the closure program for S1–S6. It is NOT another UI feature.

Build a deterministic regression matrix covering:

- S1: Columns
- S2: Indexes
- S3: Relations / Foreign Keys
- S4: Triggers
- S5: DDL
- S6: ER Diagram

For each feature, evaluate both providers (PostgreSQL, SQLite) across:

- CREATE
- READ / INTROSPECT
- UPDATE / ALTER
- DROP
- REFRESH
- ERROR
- ROLLBACK
- READ-ONLY SAFETY
- SPECIAL IDENTIFIERS
- CACHE CONSISTENCY
- UI CONSISTENCY

## Problem

S1–S6 have been implemented and individually verified, but no single document
provides a holistic view of what is proven vs. what remains pending across the
entire schema surface. S7 closes this gap.

## Target behavior

A deterministic regression matrix (FINDINGS.md) that, for each S1–S6 feature ×
operation × provider cell, states one of:

- PASS — automated test or CI evidence proves this cell
- SOURCE — verified at source level but no runtime test
- PENDING — not yet proven
- NOT_APPLICABLE — operation does not apply to this feature/provider

Plus a release-style schema readiness report.

## Scope

- Audit existing test coverage for S1–S6
- Identify gaps and add targeted tests where feasible
- Build the regression matrix
- Produce the schema readiness report
- Any accepted P0/P1 reopens the affected feature

## Explicit out of scope

- New UI features
- Rewriting S1–S6 implementations
- Manual runtime testing (documented as PENDING)

## Invariants

1. Every cell in the matrix must have evidence or be explicitly PENDING
2. PostgreSQL and SQLite are verified independently
3. Tests must prove invariants, not merely exercise code paths
4. Any P0/P1 found during S7 reopens the affected feature

## Test strategy

- Rust: targeted integration tests for uncovered cells
- Frontend: verify existing unit test coverage for DDL/ER cells
- CI: PG service container + SQLite in-memory for automated evidence
- UI: manual — documented as PENDING

## Completion criteria

- P0 = 0, P1 = 0
- Regression matrix complete with evidence for every cell
- Schema readiness report produced
- All feasible automated gaps filled with tests
