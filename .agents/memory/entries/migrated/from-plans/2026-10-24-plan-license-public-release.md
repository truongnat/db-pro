---
schema_version: 1
id: "MEM-PLAN-license-public-release-3c3398355c87"
title: "Plan archive: License Public Release"
type: workflow
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, workflow, license-public-release]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "3c3398355c877e59cf4782c1995ca2965a468c7e5cbdac116df4b3bf41a79c24"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/license-public-release/PLAN.md"
  - "docs/plans/active/license-public-release/FINDINGS.md"
  - "docs/plans/active/license-public-release/VERIFICATION.md"
  - "docs/plans/active/license-public-release/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/license-public-release/`. Loại suy luận: `workflow`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# DB Pro license and public-release policy

## Goal

Record the owner-approved MIT policy and remove the undefined-license state
from the current release documentation and Rust package metadata.

## Scope

- add the MIT license text;
- declare MIT metadata for every workspace package;
- record the DB Pro trademark boundary;
- add the third-party notice inventory index;
- reconcile current release/readiness documents.

## Non-goals

- renaming source identifiers, binaries, bundle IDs, or persisted paths;
- changing release runtime support, signing, or GUI-smoke decisions;
- legal clearance of the DB Pro trademark;
- generating a final platform-specific binary notice bundle.

## Acceptance

- `LICENSE` exists and is the selected MIT policy;
- all workspace packages inherit `license = "MIT"`;
- current release documents no longer say the license is undecided;
- remaining runtime/platform blockers remain accurately open;
- tests/gates actually executed are recorded in `VERIFICATION.md`.


## Source: `FINDINGS.md`

# Findings

## F1 — P1 resolved: project license was undefined

At baseline `a81757aadcb97062c74259b83b304ede051ca5ed`, the repository had no
root `LICENSE` file and no package license metadata. This blocked public
distribution because users had no explicit source or binary redistribution
policy.

Decision: MIT License, effective for the DB Pro source repository and v0.1
artifacts, with third-party notices preserved separately and DB Pro trademarks
excluded from the MIT grant.

## F2 — P2 remains: binary notice bundle is not generated

`cargo metadata --format-version 1 --locked` exposes license expressions for
the resolved dependency graph, while the bundled Inter font already carries
`crates/ui/assets/fonts/OFL.txt`. A complete exact-artifact notice bundle still
needs to be generated before public binary distribution.


## Source: `VERIFICATION.md`

# Verification

## Baseline

- Baseline SHA: `a81757aadcb97062c74259b83b304ede051ca5ed`
- Implementation SHA: `11ff3534ab059748e449a0ab8791f4b60e95ee72`
- Branch: `fix/license-public-release`

## Commands

- `cargo metadata --format-version 1 --locked` — PASS, exit 0; resolved package
  license expressions were inspected, including local packages and bundled
  font-related metadata.
- `cargo fmt --all -- --check` — PASS, exit 0.
- `cargo check --workspace` — PASS, exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS, exit 0.
- `cargo test --workspace` — PASS, exit 0; 1133 passed / 0 failed / 38 ignored across the workspace.
- `cargo build --release --locked -p db-pro-native` — PASS, exit 0.
- `cargo metadata --format-version 1 --locked` — PASS, exit 0; all six local packages report `MIT`.
- `git diff --check` — PASS, exit 0.

## Runtime evidence

Not applicable. This change does not alter database behavior or the native UI.

## Remaining limitation

No final platform-specific third-party notice bundle has been generated. This
must be completed before publishing binary artifacts publicly. The existing
candidate archives predate this policy and were not rebuilt in this change.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — license and public-release policy

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex · release-governance lane |
| Issue(s) | #119 |
| Task state | Review |
| Baseline SHA | `a81757aadcb97062c74259b83b304ede051ca5ed` |
| Branch / PR | `fix/license-public-release` · PR not created |
| Scope interpretation | Implement the owner-approved MIT policy and reconcile current release documentation. |
| Out of scope | Product rename, runtime qualification, signing, legal trademark clearance, final binary notice packaging. |

## 2. Progress checkpoint

- Implementation SHA: `11ff3534ab059748e449a0ab8791f4b60e95ee72`; evidence-only follow-up commits are `573494d8a2a578432e659a492ffa110df66c938f` and `ab7187a1e26adc9ee75e94377ccaa0cb6fc420dd`.
- Completed acceptance rows: MIT policy selected; source files added; metadata/doc reconciliation complete; required gates passed.
- Remaining acceptance rows: final binary notice bundle before public binary distribution.
- Findings / risks: P2 notice bundle remains before public binary distribution.
- Tests already run: `cargo metadata --format-version 1 --locked` → exit 0.
- Dependency / blocker changes: `R-LICENSE` is resolved by this change; runtime/platform blockers remain.

## 3. Implementation handoff / review request

| Exact SHA | `11ff3534ab059748e449a0ab8791f4b60e95ee72` |
| Commit list | `11ff353` docs(release): adopt MIT license policy |
| File / surface inventory | `LICENSE`; `THIRD_PARTY_NOTICES.md`; six package manifests plus workspace `Cargo.toml`; README/changelog and current release governance/readiness documents; active plan/evidence files |
| Acceptance mapping | #119 license decision → MIT `LICENSE`; package metadata → workspace/package `license`; third-party scope → `THIRD_PARTY_NOTICES.md`; release truth → current readiness, handoff, checklist, risk register and notes |
| Commands and counts | `cargo fmt --all -- --check` PASS; `cargo check --workspace` PASS; `cargo clippy --workspace --all-targets -- -D warnings` PASS; `cargo test --workspace` PASS — 1133 passed / 0 failed / 38 ignored; `cargo build --release --locked -p db-pro-native` PASS; `cargo metadata --format-version 1 --locked` PASS — six local packages MIT; `git diff --check` PASS |
| CI run IDs / status | not run; no push/PR created |
| Known limitations | Final platform-specific dependency notice bundle not generated; existing candidate archives predate this policy and were not rebuilt |
| Migrations / config implications | No runtime or persisted-state migration; future release archives must include the license and applicable third-party notices |
| Out-of-scope changes | Product rename, runtime qualification, signing, legal trademark clearance |

## 4. Review outcome

Pending independent review; self-verification is not independent approval.

## 5. Research / audit handoff

- Source date: 2026-09-16
- Source references: `Cargo.toml`, `crates/*/Cargo.toml`, `README.md`, `docs/release/0.1.0-readiness.md`, `docs/release/risk-register.md`, `crates/ui/assets/fonts/OFL.txt`.
- Factual finding: baseline had no project `LICENSE` or manifest license metadata.
- Inference: MIT is the lowest-friction permissive policy for the intended public desktop tool; legal counsel remains authoritative for entity-specific terms.
- Decision: MIT selected by the project owner; DB Pro trademarks remain separate.
- Unresolved questions: exact binary notice bundle generation before public distribution.
- Downstream tasks activated: none.

## 6. Tổng kết bằng tiếng Việt

Đã triển khai policy MIT cho DB Pro trên branch riêng. Scope chỉ gồm license,
manifest metadata, third-party notice index và release docs; không đổi runtime hay rename
source. Còn phải chạy gate và tạo notice bundle hoàn chỉnh trước khi public binary release.

