---
schema_version: 1
id: "MEM-SRC-a6c71345a55f6166"
title: "Source mirror: docs/plans/active/responsive-layout-primitives/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/responsive-layout-primitives/FINDINGS.md"
source_sha256: "a6c71345a55f6166b248cc26c5f1e33ec122e1c42bd21ded7c7aec8efd4da622"
mirror_body_sha256: "a6c71345a55f6166b248cc26c5f1e33ec122e1c42bd21ded7c7aec8efd4da622"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Responsive Layout Primitives — Findings

## Baseline

- **Source SHA:** `44d157e2e4c6ad9d83be9ec69414f508f91d38d8`
- **Working branch:** `feature/responsive-layout-primitives`
- **Source evidence date:** 2026-09-26

## Findings

### P2 — Responsive layout rules are duplicated per view

- Evidence: `crates/ui/src/component_gallery_inputs.rs:4,109-120` defines a local `680.0` threshold and separate rendering branches for one and two columns.
- Failure scenario: when a sidebar reduces the local available width, caller-specific thresholds and widget intrinsic minimums can disagree; the right side may clip rather than reflow consistently.
- Scope decision: establish a reusable local-Ui width contract and migrate only the Gallery form as proof.

### P2 — Gallery content container has a bespoke width policy

- Evidence: `crates/ui/src/component_gallery_view.rs:230-248` applies local horizontal inset and a `1120.0` max width.
- Failure scenario: a fixed max-width policy detached from allocated local width can allow the content child to exceed its actual viewport if parent min/max constraints disagree.
- Scope decision: use a shared fluid bounded Container, but avoid changing unrelated gallery category navigation.

### No database/provider risk identified

This feature affects layout only. PostgreSQL and SQLite behavior are both N/A; no SQL, persistence, runtime worker or provider capability changes are planned.

## Open questions for API owner

1. Can the Row/Col closure API support heterogeneous one-shot children without boxed per-frame allocations?
2. Can its allocated child `Response::rect` be tested against the local parent clip rect at narrow widths?

## Decisions

- V1 includes the requested Container, Row, Col and equal-width ResponsiveGrid concepts, but **not** Bootstrap's full 12-column/named-breakpoint utility system.
- Default horizontal gutter and row/grid gap use `SPACE_LG`; callers can override deliberately.
- Gallery form order is row-major and remains constant across breakpoints: Display name → Database → Host → Password → Port → SSL.
- API owner must show no-overflow behavior for actual rendered child response bounds before broad migration.
