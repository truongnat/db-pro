---
schema_version: 1
id: "MEM-PLAN-ui-spacing-audit-05a786f92142"
title: "Plan archive: Ui Spacing Audit"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, ui-spacing-audit]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "05a786f92142ba40981efe30de2cdaf004f7760a25872f6e7ae96ea7d04ed49f"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/completed/ui-spacing-audit/PLAN.md"
  - "docs/plans/completed/ui-spacing-audit/FINDINGS.md"
  - "docs/plans/completed/ui-spacing-audit/VERIFICATION.md"
  - "docs/plans/completed/ui-spacing-audit/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/completed/ui-spacing-audit/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# UI spacing audit

State: COMPLETED (audit deliverable only). Source: `52cdf982e748a0f57a1dae3b4493316268e22318`.

Owner scope: audit comprehensive native UI spacing/padding/sizing/alignment after removing workspace tab accent. Output: verified findings, actual geometry, native captures, role contract and prioritized fix plan. Audit does not implement its findings.

Coverage: 559 production source files scanned; 162 contain matched spacing lines; 24 native captures and 15 geometry cases. Boundaries and unexercised paths are explicit in REPORT.md. Future implementation of the seven P2 findings is separate work.


## Source: `FINDINGS.md`

# Findings / learning pass

Authoritative findings: REPORT.md, SP01–SP07. All are P2; no P0/P1 identified within the audited spacing scope.

Reusable lessons recorded here and in report:

1. egui `add_space` advances a cursor already advanced by `item_spacing`; measure actual bounds rather than reading token names as final gaps.
2. Grid gutter belongs to row/cell placement, not internal component density. Restore child spacing after parent layout overrides.
3. Trailing control reservation includes native sibling spacing. PasswordInput is the existing pattern to reuse for Input/Select.
4. Zero-gap galleries can hide width overflow present in normal parents. Validate shared components in more than one parent-spacing context.
5. Logical points and physical screenshots differ on Retina. Width-only captures with height clamp cannot claim full viewport QA.

No global memory writes. No colors/fonts/renderer changes. No provider mutations.


## Source: `VERIFICATION.md`

# Verification

Source SHA: `52cdf982e748a0f57a1dae3b4493316268e22318`. Baseline before tab accent removal: `6b39d7c47d5744ad52b40dab731ce0336a08d171`.

## Commands actually executed

Tab removal: `git diff --check`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo build --release --locked -p db-pro-native`: all exit 0. Workspace totals 1601 passed / 0 failed / 41 ignored. No test added for the four-line paint removal. The full test log was transient `/tmp/db-pro-tab-border-tests.log`.

Audit: `cargo build --release --locked -p db-pro-native --features capture` passed; native capture batch 24/24 wrote screenshots. Geometry probe: `cargo run --quiet -p db-pro-ui --example spacing_audit_probe` exited 0, output `evidence/measurements.json`. Initial probe constructor arity error corrected before successful run. Probe source saved as `evidence/measurements-probe.rs`; temporary workspace example removed. Source code unchanged by audit. Final `cargo fmt --all -- --check` and base release build passed.

To reproduce: copy probe into `crates/ui/examples/spacing_audit_probe.rs`, run the command, then remove the temporary example. It uses public shared components, native egui Context and actual output shape bounds.

## Artifacts

- `spacing-inventory.csv`: 976 matched source lines, 431 direct numeric lines, 162 files.
- `sizing-inventory.csv`: 307 sizing/allocation lines.
- `spacing-contracts.csv`: 251 declared constants.
- `inventory-summary.json`: counts and hotspot summary.
- `measurements.json`, `measurements-probe.rs`: 15 component/context measurements.
- `capture-manifest.json`: every capture's exit status, existence and physical size.
- `overview.jpg`, `overview-other.jpg`: montages reviewed; originals retained.

Native capture: isolated data directory per process, 30 settle frames. 20 initial captures at logical 1280×800 have physical 2560×1600. Buttons and Form Error at 1440×900/1920×1080 have physical 2880×1676/3840×1676 due host work area clamp. Both 1440 processes wrote PNG then were terminated after 12-second timeout (-15); others exited 0. This does not establish full-height 900/1080 acceptance.

Screenshots include normal/error/loading/empty fixture evidence across surfaces, not every state of every component. Gallery popup open states, all scroll positions, every locale and live providers were not exercised. No CI, independent review or frame-performance benchmark run. The report does not claim those gates passed.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — spacing audit

## 1. Claim

Agent: root / audit. Issue: owner request. Task state: Done (audit). Lifecycle: COMPLETED audit deliverable.
Exact source SHA: `52cdf982e748a0f57a1dae3b4493316268e22318`. Branch: main; PR n/a, owner authorizes commit/push.
Scope: native UI spacing/padding/sizing/alignment, including shared controls and product/Gallery surfaces.

## 2. Progress checkpoint

Completed: inventory, source traces, 15 geometry cases, 24 screenshots, seven confirmed P2 findings. Proof boundaries explicit in REPORT.md/VERIFICATION.md. No dependency/provider changes. Suggested fixes are future work, not pending audit execution.

## 3. Implementation handoff / review request

Source commit `52cdf982e748a0f57a1dae3b4493316268e22318`: `fix(ui): remove active workspace tab top accent` (four paint lines removed). Audit changes are documents, inventories, probe artifact and screenshots only.
Inventory and acceptance mapping: CHECKLIST.md / REPORT.md. Commands/counts: VERIFICATION.md (1601/0/41 workspace; probe/captures statuses retained). CI: not run. Persisted data/API/env changes: none. PostgreSQL/SQLite: fixture UI only, no live-provider claim.

## 4. Review outcome

Reviewed source: `52cdf982e748a0f57a1dae3b4493316268e22318`. Audit verdict: ACCEPT WITH P2 for spacing quality; audit delivery itself done. P0/P1/P2: 0/0/7 inherited findings at the audited source; none introduced by tab-line removal. This is self-audit, not independent approval. Frame/whole-UI quality not scored. Remaining implementation priorities: REPORT.md.

## 5. Research / audit handoff

Read local source and skills on 2026-10-02. Existing `.impeccable.md` supplies audience/design context; current owner preferences guide retained egui density. Read egui 0.29.1 cursor-spacing implementation; used it to trace measured native geometry. No external research. Scope, measurements, recommendations and uncertainty: REPORT.md. Reusable learning: FINDINGS.md.

## 6. Tổng kết bằng tiếng Việt

Viền xanh tab đã bỏ và push. Audit quét 559 file UI, đo 15 case geometry và thu 24 ảnh native, xác nhận 7 lỗi spacing/sizing P2. Ưu tiên sửa inheritance Grid/Gallery, width reservation Input/Select, height contract và Feedback responsive. Audit không tự thay padding production; ảnh lớn có giới hạn chiều cao được ghi rõ.

