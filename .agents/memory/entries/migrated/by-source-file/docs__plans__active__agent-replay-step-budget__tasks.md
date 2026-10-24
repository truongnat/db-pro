---
schema_version: 1
id: "MEM-SRC-5d759cdbe836d0ec"
title: "Source mirror: docs/plans/active/agent-replay-step-budget/tasks.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/agent-replay-step-budget/tasks.md"
source_sha256: "5d759cdbe836d0ec351ca7e48260e18676bd54137b2dae14ad0c285ad633d4d3"
mirror_body_sha256: "5d759cdbe836d0ec351ca7e48260e18676bd54137b2dae14ad0c285ad633d4d3"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Tasks — #448

- [x] T1 / R1–R4: Scope approved; baseline runtime reproduction recorded (D4).
- [x] T2 / R1–R3: Add bounded replay/error/rejection and limit-boundary regression coverage to the existing adjacent test file; observe a failing regression before production edits (D1–D4). Baseline: 15 passed, 3 failed.
- [x] T3 / R1–R3: Move existing budget enforcement before completed-call cache replay; retain pending-confirmation handling and all execution caches (D1–D3). Depends on T2.
- [x] T4 / R4: Run affected suite, independent public-orchestrator smoke, and Rust quality gates (D4–D5). All executed; workspace Clippy fails on unchanged app sources, other gates pass.
- [x] T5 / R1–R4: Record actual acceptance/gates/limitations and handoff state (D6). Scope implemented; lifecycle REVIEW, not COMPLETED or merge-ready.
