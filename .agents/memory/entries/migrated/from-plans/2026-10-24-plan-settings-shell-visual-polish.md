---
schema_version: 1
id: "MEM-PLAN-settings-shell-visual-polish-07a3eb191075"
title: "Plan archive: Settings Shell Visual Polish"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, settings-shell-visual-polish]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "07a3eb191075cc148c2088ed0313a45591046e6153830b185f68cd8d4713d19d"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/settings-shell-visual-polish/PLAN.md"
  - "docs/plans/active/settings-shell-visual-polish/FINDINGS.md"
  - "docs/plans/active/settings-shell-visual-polish/VERIFICATION.md"
  - "docs/plans/active/settings-shell-visual-polish/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/settings-shell-visual-polish/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Settings Shell Visual Polish

## State

`RUNTIME_VERIFY`

## Goal

Make the native Settings surface read as a dedicated settings workspace like
the supplied light/dark references: a quiet background, a distinct navigation
rail, grouped settings navigation, and an obvious active pill.

## Scope

- Native Settings shell composition in `crates/ui`.
- Light/dark theme mapping through existing `DbProTheme` semantic tokens.
- Settings navigation grouping and active/hover painting.
- Deterministic native capture coverage for the affected surface.

## Non-goals

- No new settings, persistence fields, database commands, or provider logic.
- No redesign of the normal database workspace shell outside Settings mode.
- No React/Tauri frontend changes.

## Acceptance

1. Settings opens as a full central surface while the normal workspace chrome is
   suppressed for that mode.
2. Existing section selection and settings actions still dispatch unchanged.
3. Light and dark surfaces use `DbProTheme` tokens without new raw colors.
4. Native captures exist for 1280×800 in both themes; standard Rust gates and
   the locked native release build are executed.

## Provider impact

Presentation-only. PostgreSQL and SQLite behavior is unchanged and therefore
not applicable.


## Source: `FINDINGS.md`

# Findings

## F-1 — Settings was rendered as a sidebar activity, not a settings workspace

**Severity:** P2 (visual hierarchy)

**Evidence:** Before the change, `Activity::Settings` only routed through
`draw_sidebar_activity_body`, while the central panel continued rendering the
active workspace tab. The baseline capture showed Settings controls in the
sidebar beside a Welcome workspace.

**Decision:** Route Settings through the central panel and suppress unrelated
workspace chrome for this mode. Existing settings content and action dispatch
remain the source of truth.

## F-2 — Navigation had no grouped active treatment

**Severity:** P2 (navigation clarity)

**Evidence:** The old navigation used plain `selectable_label` rows with a
fixed 128px width. It did not express the grouped rail + active pill shown in
the supplied references.

**Decision:** Group sections into Workspace, Connections, and System; paint
semantic active/hover fills and Lucide icons using `DbProTheme` tokens.

## Self-review

- P0: 0
- P1: 0
- P2: 2 (F-1, F-2), addressed in this slice.
- No database/provider/runtime behavior changed.

## Provider matrix

| Provider | Impact | Evidence |
|---|---|---|
| PostgreSQL | None; presentation-only | N/A |
| SQLite | None; presentation-only | N/A |


## Source: `VERIFICATION.md`

# Verification

## Source baseline

- Base SHA: `bd0535fb`.
- Working branch: local `main` per explicit user workflow override.
- Implementation commit: `fb57eebf1426328078a5aefa13c0dcf5ce7b0c7c`.
- Follow-up visual correction: `bb5830edf8a3775a3eb0d2e7b2087ffd952b582f` — Settings navigation now uses the shared `ButtonVariant::Ghost` / `ButtonVariant::Secondary` treatment, so inactive items stay transparent and the surface appears only on hover or active state.

## Runtime evidence

| Requested viewport | Captured framebuffer | Theme | Artifact |
|---|---:|---|---|
| 1280×800 | 2560×1600 (Retina 2×) | Dark | `screenshots/settings-dark-1280x800.png` |
| 1280×800 | 2560×1600 (Retina 2×) | Light | `screenshots/settings-light-1280x800.png` |
| 1440×900 | 2880×1676 (Retina 2×, host-capped height) | Dark | `screenshots/settings-dark-1440x900-capped.png` |
| 1920×1080 | 3840×1676 (Retina 2×, host-capped height) | Dark | `screenshots/settings-dark-1920x1080-capped.png` |

The captures were visually inspected after the follow-up variant correction.
They show the dedicated Settings surface, grouped navigation, a semantic active
General pill, transparent inactive navigation, layered background, and existing
General/Diagnostics content. The host capture path produces a 2× Retina
framebuffer and caps logical height at approximately 838px, so the 1440×900
and 1920×1080 artifacts are valid width checks but not exact-height evidence.

## Automated gates

- `cargo fmt --all -- --check`: PASS.
- `cargo check --workspace`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace`: PASS — 928 UI tests passed; all workspace suites
  passed, with only the repository's expected provider/fixture tests ignored.
- `cargo build --release --locked -p db-pro-native`: PASS.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  PASS — 15 checks passed, 1 inherited size warning, 0 failures. The warning
  is limited to pre-existing long functions in `settings_model.rs`,
  `workspace_actions.rs`, and `workspace_view.rs`.

## Provider matrix

| Provider | Supported behavior changed | Automated evidence | Live/runtime evidence | Capability gate |
|---|---|---|---|---|
| PostgreSQL | No | N/A | N/A | Existing capability behavior unchanged |
| SQLite | No | N/A | N/A | Existing capability behavior unchanged |

## Tổng kết bằng tiếng Việt

Settings đã được tách thành một workspace native riêng với nền phân lớp, rail
navigation theo nhóm và active pill; pipeline action/state cũ được giữ nguyên.
Dark/light capture 1280×800 và các capture width 1440/1920 đã kiểm tra trực
quan. Toàn bộ gate tự động đã chạy đạt; độ cao 1440/1920 bị giới hạn bởi host,
và independent review vẫn chưa thực hiện.

## Audit evidence update — 2026-10-13

`docs/plans/_audit-2026-10-13/06-shell-crosscutting.md` verdict: **SUPPORTS** — the
dedicated Settings mode and chrome suppression verified at HEAD
(`app_lifecycle.rs:187-221` `settings_mode` suppresses topbar/tabs/sidebar/dock/statusbar).
Fresh captures: `docs/plans/_audit-2026-10-13/captures/settings_{1280x800,1440x900,1920x1080}.png`
(larger heights host-capped at 838, as before). Only independent review remains.
State stays RUNTIME_VERIFY.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — native Settings shell visual polish

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex · native-egui UI implementation lane |
| Issue(s) | n/a |
| Task state | Review |
| Baseline SHA | `bd0535fb4e4258f969e090d554d3f5dcb86e5996` |
| Branch / PR | `main` / no PR |
| Scope interpretation | Recompose the native Settings activity into a dedicated full-surface workspace with grouped navigation, semantic active treatment, and light/dark visual evidence. |
| Out of scope | New settings behavior, persistence, database/provider logic, React/Tauri frontend, and normal workspace redesign. |

## 2. Progress checkpoint

- Current implementation baseline: `fb57eebf1426328078a5aefa13c0dcf5ce7b0c7c`; follow-up visual correction: `bb5830edf8a3775a3eb0d2e7b2087ffd952b582f`
- Completed acceptance rows: [x] dedicated Settings surface, [x] normal Settings chrome suppressed, [x] grouped navigation with active/hover states, [x] existing action/state pipeline preserved, [x] light/dark 1280×800 captures, [x] automated Rust gates and locked release build.
- Remaining acceptance rows: [ ] independent review; [ ] exact-height 1440×900 and 1920×1080 captures on a host without the current height cap.
- Findings / risks: P2 visual hierarchy findings F-1/F-2 in `FINDINGS.md:3-27` are addressed by `app_lifecycle.rs:172-208`, `workspace_view.rs:85-90`, and `settings_surface_view.rs:40-156`; no open P0/P1.
- Tests already run: `cargo test --workspace` → 1585 passed / 0 failed / 41 ignored, exit 0.
- Dependency / blocker changes: none; the capture host caps logical height at approximately 838px for the 1440×900 and 1920×1080 requests.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `bb5830edf8a3775a3eb0d2e7b2087ffd952b582f` |
| Commit list | `fb57eebf feat(ui): redesign native settings shell`; `bb5830ed refactor(ui): unify disclosure components` — includes the Settings navigation variant correction |
| File / surface inventory | `AGENTS.md` — records the owner rule to edit local `main` and never create/use worktrees; `crates/ui/src/app_lifecycle.rs` — Settings shell chrome routing; `crates/ui/src/settings_model.rs` — removes obsolete navigation helper; `crates/ui/src/settings_navigation_view.rs` — grouped semantic navigation rail; `crates/ui/src/settings_surface_view.rs` — full Settings surface composition; `crates/ui/src/shell_frame_view.rs` — Settings background/margin mode; `crates/ui/src/workspace_actions.rs` — deterministic light capture switch; `crates/ui/src/workspace_view.rs` — Settings central rendering route; `docs/plans/STATUS.md` — lifecycle status; `docs/plans/active/settings-shell-visual-polish/` — plan, checklist, findings, verification, evidence, and four runtime captures.` |
| Acceptance mapping | Dedicated surface/chrome suppression → `app_lifecycle.rs`, `shell_frame_view.rs`, `workspace_view.rs`; grouped active navigation → `settings_navigation_view.rs`; semantic light/dark rendering → `settings_surface_view.rs` and existing `DbProTheme`; runtime evidence → `screenshots/*.png`; gate evidence → `VERIFICATION.md`. |
| Commands and counts | `cargo fmt --all -- --check` → pass, exit 0; `cargo check --workspace` → pass, exit 0; `cargo clippy --workspace --all-targets -- -D warnings` → pass, exit 0; `cargo test --workspace` → 1585 passed / 0 failed / 41 ignored, exit 0; `cargo build --release --locked -p db-pro-native` → pass, exit 0; `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` → 15 pass / 1 warning / 0 fail, exit 0.` |
| CI run IDs / status | not run; local gates executed on the implementation SHA |
| Known limitations | Exact logical heights above the host cap are unverified; loading/error/empty Settings states were not traversed because this slice is presentation-only; independent review remains pending. |
| Migrations / config implications | None. `DB_PRO_CAPTURE_SETTINGS_LIGHT` is a capture-only environment switch; no persisted schema or settings key changed. |
| Out-of-scope changes | No database/provider behavior, React/Tauri frontend, or normal workspace shell redesign. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `bb5830edf8a3775a3eb0d2e7b2087ffd952b582f` |
| Verdict | `ACCEPT WITH P2` for self-review; independent review not run |
| P0 / P1 / P2 counts | Introduced by this SHA: 0 / 0 / 2 addressed visual findings plus 2 verification limitations; inherited: 0 / 0 / 1 clean-code size warning |
| Findings | No open P0/P1. P2: host height cap prevents exact 1440×900 and 1920×1080 evidence; independent review is still required before lifecycle completion. |
| CI disposition | Local gates above passed; no CI run was triggered on local `main`. |
| Next task(s) unblocked | Independent UI review and, if available, recapture at the exact requested logical heights. |

## 5. Research / audit handoff

- Source date: 2026-09-30.
- Source URLs / references: supplied light/dark screenshots; `docs/10-egui-native-migration-plan.md`; `.cursor/skills/frontend-design/SKILL.md`; `.skills/clean-code/SKILL.md`.
- Factual findings: the implementation uses existing `DbProTheme` semantic surfaces, keeps settings action/state behavior unchanged, and applies the shared button variant correction at `bb5830edf8a3775a3eb0d2e7b2087ffd952b582f`.
- Inference: the dedicated central composition is the closest native-egui equivalent to the supplied reference without restoring the archived web frontend.
- Decision / recommendation: retain `RUNTIME_VERIFY` until independent review and exact-height evidence are available.
- Unresolved questions: none beyond the evidence limitations above.
- Downstream tasks activated: none.

## 6. Tổng kết bằng tiếng Việt

Đã làm lại native Settings theo visual reference: nền phân lớp, rail điều hướng
theo nhóm, active pill rõ ràng, hỗ trợ dark/light và mở thành workspace riêng;
đồng thời ghi rõ trong `AGENTS.md` rằng các task sau chỉ code trực tiếp trên
`main`, không tạo hoặc dùng worktree. Toàn bộ gate local đạt; còn independent
review và capture đúng chiều cao 1440×900 / 1920×1080 cần thực hiện sau.

