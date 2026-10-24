---
schema_version: 1
id: "MEM-PLAN-disclosure-components-polish-385e8d6c375e"
title: "Plan archive: Disclosure Components Polish"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, disclosure-components-polish]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "385e8d6c375ec17bb6e5cfab72dba28c15db136f37f26fbcdfff463588f06a08"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/disclosure-components-polish/PLAN.md"
  - "docs/plans/active/disclosure-components-polish/FINDINGS.md"
  - "docs/plans/active/disclosure-components-polish/VERIFICATION.md"
  - "docs/plans/active/disclosure-components-polish/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/disclosure-components-polish/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Collapsible & Accordion UI Polish

## State

`RUNTIME_VERIFY`

## Goal

Make Collapsible and Accordion read as one DB Pro disclosure family: quiet at
rest, surfaced on hover/open, aligned body content, and one predictable
expand/collapse animation.

## Scope

- Native egui Collapsible and Accordion rendering.
- Shared disclosure surface, color, spacing, focus, chevron, and body animation.
- Component-gallery capture for the affected state.

## Non-goals

- No public builder/API removal.
- No database/provider behavior.
- No React/Tauri frontend changes.
- No unrelated component-gallery redesign.

## Acceptance

1. Closed disclosure headers have no persistent fill; hover and open states use
   semantic theme surfaces.
2. Collapsible and Accordion share header geometry, body inset, focus treatment,
   and chevron transition.
3. Body expansion clips height and fades content without reserving the full
   expanded height during the transition.
4. Existing single/multi selection, disabled behavior, keyboard activation, and
   public APIs remain intact.
5. Native capture and Rust gates are recorded before completion.

## Provider impact

Presentation-only. PostgreSQL and SQLite behavior are unchanged.


## Source: `FINDINGS.md`

# Findings

## F-1 — Disclosure headers had divergent visual systems

**Severity:** P2 (UI consistency)

Collapsible and Accordion each painted their own hover fill, colors, geometry,
badge spacing, and body margins. Accordion also added a persistent divider while
Collapsible did not. The same interaction therefore looked like two unrelated
components.

**Decision:** Move header surface, state colors, focus ring, body margin, and
chevron treatment into `components/disclosure.rs` and consume it from both
components.

## F-2 — Body animation faded but did not animate layout height

**Severity:** P2 (interaction quality)

The old implementation rendered the full body layout as soon as openness passed
the threshold, then only changed opacity. Closing could leave reserved blank
space while the body faded.

**Decision:** Use egui `CollapsingState::show_body_unindented` for clipped height
and persist the measured body height; apply the shared opacity and inset inside
that clipped body.

## Self-review

- P0: 0
- P1: 0
- P2: 2 addressed in this slice.
- Provider impact: none.


## Source: `VERIFICATION.md`

# Verification

## Source baseline

- Base SHA: `3b1cc99b4c389a574c05af66bc1d97d7e2592263`.
- Working branch: local `main` per explicit owner workflow override.
- Implementation commit: `bb5830edf8a3775a3eb0d2e7b2087ffd952b582f`.
- Follow-up spacing correction: `8470ea83148da517b7ac12be6da5ca8c97fe96ec` — the trailing Accordion badge now reserves `SPACE_MD` between its pill and the chevron.
- Follow-up contrast correction: `4bf6bad4fdff253f3593ba125fcb636887aa750f` — the custom badge now uses the shared secondary badge palette with an elevated fill, border, and secondary text.
- Follow-up weight correction: `24ffabc756110cd3897304d21dbc2efd801d211b` — the custom badge keeps the elevated fill and secondary text but drops the heavy border, leaving separation to the spacing token.

## Runtime evidence

| Requested viewport | Captured framebuffer | Theme | Artifact |
|---|---:|---|---|
| 1280×800 | 2560×1600 (Retina 2×) | Dark | `screenshots/disclosure-gallery-dark-1280x800.png` |

The refreshed capture shows the closed SSH item without a background, while the
open Pool and SSL items use the active semantic surface with aligned body
content. The `Enforced` badge now has visible breathing room before the trailing
chevron and remains legible on the active header surface without a heavy border.
It is a static state
capture; hover and transition timing still require interactive review.

## Automated gates

- `cargo fmt --all -- --check`: PASS.
- `cargo check --workspace`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace`: PASS — 1583 passed / 0 failed / 41 ignored; doc
  tests also passed.
- `cargo build --release --locked -p db-pro-native`: PASS.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  PASS — 14 checks, 2 inherited warnings, and 0 failures. Warnings are the
  pre-existing long gallery/capture functions and the existing 831-line
  `components/mod.rs` file.

## Provider matrix

| Provider | Behavior changed | Automated evidence | Runtime evidence |
|---|---|---|---|
| PostgreSQL | No | N/A | N/A |
| SQLite | No | N/A | N/A |

## Tổng kết bằng tiếng Việt

Đã gom Collapsible và Accordion về cùng một disclosure style: nền trong suốt ở
trạng thái thường, surface semantic khi hover/active, cùng spacing/focus/
chevron, badge trailing có khoảng cách rõ với chevron và tương phản độc lập với
active surface, và dùng clipped body
animation để mở đóng không còn chừa khoảng trống. Capture native đã kiểm tra
trạng thái đóng/mở; toàn bộ full workspace gates đã đạt. Independent review vẫn
cần hoàn tất.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — Collapsible & Accordion UI polish

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex · native-egui component-system lane |
| Issue(s) | n/a |
| Task state | Review |
| Baseline SHA | `3b1cc99b4c389a574c05af66bc1d97d7e2592263` |
| Branch / PR | `main` / no PR |
| Scope interpretation | Unify Collapsible and Accordion visual states, spacing, focus, chevron transition, and body expansion animation. |
| Out of scope | Database/provider logic, public builder removal, React/Tauri frontend, and unrelated gallery redesign. |

## 2. Progress checkpoint

- Current HEAD: `24ffabc756110cd3897304d21dbc2efd801d211b`
- Completed acceptance rows: [x] transparent rest state, [x] semantic hover/open surfaces, [x] shared geometry/focus/body inset, [x] clipped height + fade animation, [x] trailing badge/chevron spacing and restrained contrast, [x] existing API/state/keyboard behavior preserved, [x] native capture, [x] full local gates.
- Remaining acceptance rows: [ ] independent review; [ ] interactive hover/transition recording beyond the static capture.
- Findings / risks: P2 F-1/F-2 in `FINDINGS.md:3-27` are addressed by `components/disclosure.rs:1-93`, `components/collapsible/ui.rs:72-135`, and `components/accordion/ui.rs:30-159`; no open P0/P1.
- Tests already run: `cargo test --workspace` → 1583 passed / 0 failed / 41 ignored, exit 0.
- Dependency / blocker changes: none.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `24ffabc756110cd3897304d21dbc2efd801d211b` |
| Commit list | `bb5830ed refactor(ui): unify disclosure components`; `8470ea83 fix(ui): add space between accordion badge and chevron`; `4bf6bad4 fix(ui): keep accordion badges visible on active headers`; `24ffabc7 polish(ui): lighten accordion status badges` |
| File / surface inventory | `crates/ui/src/components/disclosure.rs` — shared disclosure style/body animation; `accordion/ui.rs` + `collapsible/ui.rs` — consume shared state and painting, with `SPACE_MD` badge/chevron separation and a restrained elevated badge fill; component config/handler/README files — remove duplicate contracts and document the new behavior; `components/mod.rs` — register shared module; `component_gallery_inputs.rs` + `workspace_actions.rs` — deterministic disclosure capture path; `settings_navigation_view.rs` — use existing Ghost/Secondary button variants for nav items; `docs/plans/active/disclosure-components-polish/` — plan, findings, checklist, verification, evidence, and capture; `docs/plans/STATUS.md` — lifecycle row.` |
| Acceptance mapping | Rest/hover/open surface → `disclosure::paint_header_surface`; shared colors/focus/chevron → `disclosure.rs`; badge spacing/contrast/weight → `accordion/ui.rs` at `24ffabc756110cd3897304d21dbc2efd801d211b`; clipped body height → `disclosure::show_body` using egui `CollapsingState`; state/keyboard behavior → existing handlers and component tests; visual evidence → `screenshots/disclosure-gallery-dark-1280x800.png`. |
| Commands and counts | `cargo fmt --all -- --check` → pass, exit 0; `cargo check --workspace` → pass, exit 0; `cargo clippy --workspace --all-targets -- -D warnings` → pass, exit 0; `cargo test --workspace` → 1583 passed / 0 failed / 41 ignored, exit 0; `cargo build --release --locked -p db-pro-native` → pass, exit 0; clean-code scan → 14 pass / 2 inherited warnings / 0 fail, exit 0.` |
| CI run IDs / status | not run; local gates executed on the implementation SHA |
| Known limitations | Static capture does not prove hover timing; independent review remains pending. |
| Migrations / config implications | None. `DB_PRO_CAPTURE_GALLERY_SECTION=disclosure` is capture-only. |
| Out-of-scope changes | No database/provider behavior or public component API removal. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `24ffabc756110cd3897304d21dbc2efd801d211b` |
| Verdict | `ACCEPT WITH P2` for self-review; independent review not run |
| P0 / P1 / P2 counts | Introduced by this SHA: 0 / 0 / 2 addressed; inherited: 0 / 0 / 2 clean-code warnings |
| Findings | No open P0/P1. P2: interactive transition capture and independent review remain. |
| CI disposition | Local gates above passed; no CI run triggered on local `main`. |
| Next task(s) unblocked | Independent UI review and interactive hover/close/open verification. |

## 5. Research / audit handoff

- Source date: 2026-09-30.
- Source URLs / references: `.impeccable.md`; `.skills/clean-code/SKILL.md`; egui `CollapsingState` implementation in the local cargo registry; existing component-gallery usage.
- Factual findings: both components now consume the same header surface/color/focus/chevron helpers and the same clipped body helper; the Accordion trailing badge reserves `SPACE_MD` before the chevron and uses a restrained elevated fill without a border at `24ffabc756110cd3897304d21dbc2efd801d211b`.
- Inference: using egui's measured `CollapsingState` is safer than the previous opacity-only body because it preserves layout during open/close transitions.
- Decision / recommendation: retain `RUNTIME_VERIFY` until independent review is complete.
- Unresolved questions: none beyond the evidence limitation above.
- Downstream tasks activated: none.

## 6. Tổng kết bằng tiếng Việt

Đã sửa Collapsible và Accordion về cùng một visual language: trạng thái thường
trong suốt, hover/active dùng semantic surface, cùng header/focus/body spacing,
chevron crossfade và body animation có clip chiều cao thật. Capture native cho
thấy item đóng không có nền và item mở có active surface; toàn bộ gate local đã
đạt. Còn independent review và kiểm tra tương tác hover/animation trực tiếp.

