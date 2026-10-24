---
schema_version: 1
id: "MEM-PLAN-ui-spacing-consistency-625a56d680d6"
title: "Plan archive: Ui Spacing Consistency"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, ui-spacing-consistency]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "625a56d680d623951810e4ea3f31a9415702c4e570f5002bb01db613634a2095"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/ui-spacing-consistency/PLAN.md"
  - "docs/plans/active/ui-spacing-consistency/FINDINGS.md"
  - "docs/plans/active/ui-spacing-consistency/VERIFICATION.md"
  - "docs/plans/active/ui-spacing-consistency/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/ui-spacing-consistency/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# UI spacing consistency

State: IMPLEMENTING. Baseline: `6c41cc4fc93eb3e12eb9cada2069cd3514e9b3f2`.

Fix the owner-approved spacing audit SP01–SP07 and workspace tab padding. Shared layout/component geometry owns its gaps; no renderer, font or database changes. Keep egui, selection colors, button animation and removal of the workspace tab accent line.

Acceptance: 300pt fields retain width with clear/toggle/chevron controls; standard form fields use the existing 38pt token; grid gutters do not affect cell content; required markers and helpers use explicit semantic gaps; Gallery groups adapt to local width; tabs gain inset and separation without overlapping titles/close targets.

Provider impact: PostgreSQL/SQLite n/a, presentation only. Required quality gates: fmt, check, clippy, workspace tests, native release build. Capture affected normal/error states at three widths; report any display height clamp honestly.


## Source: `FINDINGS.md`

# Findings

At baseline `6c41cc4fc93eb3e12eb9cada2069cd3514e9b3f2`, the spacing audit records seven inherited P2 issues (see completed/ui-spacing-audit/REPORT.md). No P0/P1.

Decisions: keep a compact 32pt workspace tab, 12pt horizontal inset, 4pt track gap, 8pt shell gutter and 4pt vertical strip padding. Shared segmented tabs already have 12pt label padding; increase their target separation to 4pt. Underline tabs already have 12pt label padding and 4pt gap. Standard Input/Password/Select use the declared 38pt token; SearchInput and toolbar buttons retain their compact role.

Reusable lesson: egui scopes inherit item_spacing; row/grid gutters must be restored before rendering nested content. Width budgets must include actual visible accessory targets plus native sibling spacing. Explicit add_space adds to native spacing, so use a scoped spacing owner rather than compensating per view.

Native evidence exposed an additional source of inherited zero spacing: shell_frame_view.rs deliberately makes the workspace flush. Gallery must restore spacing from the context theme, rather than copying the already-zero workspace spacing. Query shell geometry keeps its explicit compact layout.


## Source: `VERIFICATION.md`

# Verification

Implementation source SHA: `04c1c9b3df3052ea24c23ddc361b203d4425222a`.

## Automated

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — 1605 passed, 0 failed, 41 ignored; db-pro-ui: 948 passed |
| `cargo build --release --locked -p db-pro-native --features capture` | PASS |
| `cargo build --release --locked -p db-pro-native` | PASS |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | PASS — 14 pass, 2 warnings, 0 failures; warnings are function-argument count and existing function-length ratchet |
| `git diff --check` | PASS |

The two new geometry tests cover tab title/close separation and field size/accessory reservations over parent gaps 0, 8, and 16. Another verifies required-marker, field, and helper gaps; a grid test checks that cell content and its parent keep their own spacing while the grid owns row/column gaps. The retained temporary geometry probe measured Input, clearable/disabled-clearable Input, Password, and Select at width 300: all are 38 points high and 300 points wide across the sampled contexts. Required marker gap is 4 points.

## Native evidence

The native capture build recorded Query tabs, Gallery form, error form, feedback/progress/shortcuts, and navigation tabs at requested 1280×800, 1440×900, and 1920×1080 window sizes. PNGs and their manifest are in `evidence/native/`; `overview.jpg` provides a contact sheet and `query-tabs-crop.png` shows the new tab insets at 2× physical pixels.

The Mac display constrained the 1440×900 and 1920×1080 requests to physical 2880×1676 (1440×838 logical) and 3840×1676 (1920×838 logical). Thus those captures do not prove the full requested heights. At 1440 captures, the PNG was written successfully before the isolated app required SIGTERM to close. Capture logs also show pre-existing missing replacement-glyph warnings.

## Performance and lifecycle

No frame-prepare timing, glyph-cache hit-rate, raster count, or atlas-growth measurement was collected. The change adjusts egui layout dimensions and spacing; it does not change fonts or rendering paths. Performance impact is therefore unmeasured, not claimed as a benchmark pass.

P0/P1: 0. Seven P2 spacing findings in the preceding audit were addressed in the scoped components and Gallery layout. The formal full-height native viewport gate remains pending because this machine cannot provide the requested logical heights, so lifecycle remains `RUNTIME_VERIFY`.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — UI spacing consistency

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex · native UI layout lane |
| Issue(s) | n/a |
| Task state | Review |
| Baseline SHA | `6c41cc4fc93eb3e12eb9cada2069cd3514e9b3f2` |
| Branch / PR | `main` / no PR |
| Scope interpretation | Address the seven audited P2 spacing defects and pad workspace/shared tabs. |
| Out of scope | rs-ui integration, renderer changes, DB Pro surface migration, typography/font/cache changes. |

## 2. Progress checkpoint

- Current implementation SHA: `04c1c9b3df3052ea24c23ddc361b203d4425222a`
- Completed acceptance: grid and Gallery spacing ownership, field widths/heights, labels/helpers, responsive feedback and tabs.
- Remaining acceptance: collect UI evidence at requested full logical heights; unavailable on the current display.
- Findings: P0=0, P1=0, P2=0 open. Inherited native-capture height constraint and glyph-replacement warnings are recorded in `VERIFICATION.md`.
- Tests: `cargo test --workspace` — 1605 passed / 0 failed / 41 ignored, exit 0; `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, native capture and standard release builds, and clean-code scan are recorded in `VERIFICATION.md`.
- Dependency / blocker changes: none; native display limits full-height captures.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `04c1c9b3df3052ea24c23ddc361b203d4425222a` |
| Commit list | `04c1c9b3` fix(ui): give tabs padding and isolate component spacing |
| File / surface inventory | shared responsive grid; Gallery shell/feedback; Label/FormField/Input/Password/Select geometry; segmented/workspace tabs; layout regression tests |
| Acceptance mapping | Audit SP01–SP07 and requested tab padding → shared component changes and evidence in `evidence/native/` |
| Commands and counts | All quality commands and counts in `VERIFICATION.md`; workspace tests 1605/0/41 |
| CI run IDs / status | Not run |
| Known limitations | Full 1440×900 and 1920×1080 logical-height captures are unavailable on this display; layout/performance counters were not profiled. |
| Migrations / config implications | None |
| Out-of-scope changes | No rs-ui files, DB Pro surface migration, renderer architecture, or font changes. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | n/a — implementer handoff |
| Verdict | PENDING independent review |
| P0 / P1 / P2 counts | Introduced: 0 / 0 / 0 open; inherited: 0 / 0 / 7 addressed |
| Findings | Full-height native runtime evidence remains pending at this SHA. |
| CI disposition | Not run |
| Next task(s) unblocked | Native visual check at the full requested logical heights |

## 5. Research / audit handoff

- Source date: 2026-10-02
- Source references: `docs/plans/completed/ui-spacing-audit/REPORT.md`; audited Rust paths listed in the commit; native captures under `evidence/native/`.
- Factual findings: shell sets central item spacing to zero; grid assigned its gutter to nested cells; Gallery copied the shell's zero spacing. The code now restores context spacing for Gallery detail, preserves cell content spacing, and reserves visible accessory gaps. Source SHA: `04c1c9b3df3052ea24c23ddc361b203d4425222a`.
- Inference: none.
- Decision: keep native UI on the existing egui component and theme layers.
- Unresolved questions: full-height native runtime review on a capable display.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã sửa các lỗi spacing dùng chung, tăng padding tab, bổ sung test và chụp 15 ảnh native. Các quality gate Rust đều đạt. Máy hiện tại giới hạn chiều cao cửa sổ ở 838 logical px, nên cần kiểm tra runtime lại ở màn hình hỗ trợ đủ chiều cao trước khi chuyển trạng thái hoàn tất.

