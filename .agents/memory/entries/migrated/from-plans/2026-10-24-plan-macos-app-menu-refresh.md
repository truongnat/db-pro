---
schema_version: 1
id: "MEM-PLAN-macos-app-menu-refresh-fb36ff8d48b3"
title: "Plan archive: Macos App Menu Refresh"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, macos-app-menu-refresh]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "fb36ff8d48b32b88591ac51fafcd2c774e8cd1a0d3466f8362df8c5cdb5098f0"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/macos-app-menu-refresh/PLAN.md"
  - "docs/plans/active/macos-app-menu-refresh/FINDINGS.md"
  - "docs/plans/active/macos-app-menu-refresh/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/macos-app-menu-refresh/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# macOS app menu and schema refresh

State: RUNTIME_VERIFY. Baseline: `main@3dd988e2ff06ac71942c7ec331acf66389d3c5cb`.

## Goal

Provide a standard macOS application menu for DB Pro and expose the existing schema refresh plus an explicit cache-clearing refresh from the native menu. `Refresh` forces a fresh schema introspection. `Refresh Cache` invalidates the active connection's schema cache first, then introspects and updates the UI through the existing request/event lifecycle.

## Scope

- Add standard DB Pro, File, Edit, View, Window, and Help menus on macOS, with working New Connection, New Query, documentation, and standard window/edit actions.
- Route View → Refresh and View → Refresh Cache to the active connection.
- Return cache invalidation failures through the existing `UiEvent::Failed` path.
- Preserve normal eframe app behavior and the optional capture driver.

## Non-goals

- Changing the app's connection-selection rules, schema cache policy, or provider introspection.
- Adding unsupported menu commands or a custom help/documentation surface.
- Changing non-macOS window menus.

## Architecture and invariants

macOS uses `muda` for the system menu. Menu events are forwarded to the UI thread and invoke public `DbProApp` actions. The cache-refresh request remains typed across `UiCommand` and `RuntimeCommand`; the runtime invalidates the connection-scoped cache before forced introspection and emits either `SchemaLoaded` or `Failed`. The root menu remains alive for the native window lifetime. Existing UI refresh behavior remains unchanged.

## Acceptance

- [ ] Native macOS menu bar shows the standard app, File, Edit, View, and Window menus.
- [ ] View → Refresh forces schema introspection for the active connection.
- [ ] View → Refresh Cache invalidates that connection's schema cache, then refreshes it; failures surface to the UI.
- [x] No active connection produces a clear user-facing message and no runtime command (UI regression test passes).
- [x] Capture and non-capture builds retain their existing launch paths (capture check and release build pass).
- [x] Targeted tests and the native release build pass.
- [ ] Record live macOS menu interaction and independent SQLite/PostgreSQL runtime evidence; keep the plan active until then.


## Source: `FINDINGS.md`

# Findings

Baseline inspected at `main@3dd988e2ff06ac71942c7ec331acf66389d3c5cb`.

- **P2 — Native app menu was absent.** Fixed in the working tree by installing the native app/File/Edit/View/Window/Help menus and dispatching their actions to the UI. Live menu behavior remains pending.
- **P2 — Explicit cache refresh was not exposed.** Fixed in the working tree by adding a typed cache-invalidation flag and invalidating before forced introspection. Automated UI dispatch tests pass; SQLite/PostgreSQL runtime behavior remains pending.

No P0/P1 finding is in scope. These are source findings, not live UI observations.


## Source: `VERIFICATION.md`

# Agent evidence — macOS app menu and schema cache refresh

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | MyClaw — implementation follow-up |
| Issue(s) | n/a |
| Task state | Runtime verification pending |
| Baseline SHA | `3dd988e2ff06ac71942c7ec331acf66389d3c5cb` (historical plan baseline) |
| Verified HEAD | `99460d2b42ca79f18e191880d4710b58e5881a1e` |
| Branch / PR | `main` / no PR |
| Scope | Finish the remaining code-lane checks for the native macOS menus and connection-scoped schema cache refresh. |
| Out of scope | Cache policy changes, provider introspection changes, non-macOS menu work. |

## 2. Progress checkpoint

- Source implementation is present in the verified tree; the menu owns a native `muda` menu, dispatches menu actions to the UI thread, and routes Refresh Cache through typed cache invalidation before schema introspection.
- UI tests: `cargo test -p db-pro-ui native_menu_refresh -- --nocapture` — 2 passed, 0 failed.
- Native translator tests: `cargo test -p db-pro-native translate_tests -- --nocapture` — 12 passed, 0 failed, 0 ignored.
- Build: `cargo build --release --locked -p db-pro-native` — exit 0. Build emitted an existing dead-code warning for `crates/ui/src/table_editor_context.rs::can_edit_table_rows`; it is outside this plan's scope.
- Capture configuration: `cargo check -p db-pro-native --features capture` — exit 0, with the same dead-code warning.
- Formatting and whitespace: `cargo fmt --all -- --check` and `git diff --check` — exit 0.
- Runtime evidence still missing: live interaction with the macOS menu and independent Refresh/Refresh Cache observations against SQLite and PostgreSQL. Source and build results do not substitute for those runtime checks.
- Working tree was clean at the verified source SHA before these plan-document updates. No implementation code change was necessary: the plan's implementation was already present; its stale evidence/checklist was the unfinished part.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact verified SHA | `99460d2b42ca79f18e191880d4710b58e5881a1e` |
| Relevant implementation history | macOS menu implementation and follow-up shortcut handling are committed on `main`; menu source includes `crates/native-app/src/app_menu.rs`. |
| Acceptance evidence | Active/no-active cache-refresh UI dispatch tests pass; command translation tests pass; release and capture builds pass. |
| CI run IDs / status | not run |
| Known limitations | No interactive native-window/menu evidence and no live SQLite/PostgreSQL provider observations collected. Independent review remains pending. |
| Migrations / config implications | macOS-only menu dependency; no persisted schema/config changes. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `99460d2b42ca79f18e191880d4710b58e5881a1e` — source inspected for this follow-up; no independent reviewer run |
| Verdict | implementation checks pass; runtime verification pending |
| P0 / P1 / P2 counts | not independently reviewed |
| Findings | No new in-scope source defect identified. One build warning (`can_edit_table_rows` unused) is outside this plan and was not changed. |
| CI disposition | not run |
| Next tasks | Verify live macOS menu actions and Refresh behavior against SQLite and PostgreSQL independently; obtain independent review. |

## 5. Research / audit handoff

- Existing design rationale and dependency research remain recorded above in repository history.
- Evidence level: source inspection + automated tests/builds only; no provider-runtime or UI-runtime claims are made.
- Lifecycle: remain in `RUNTIME_VERIFY`; do not move this plan to `completed/` until its runtime and review gates are satisfied.

## 6. Tổng kết (Vietnamese summary)

Phần code menu và Refresh Cache đã có sẵn trên `main`; test UI (2/2), translator (12/12), release build, capture check và format đều đạt. Đã sửa checklist/evidence cũ theo SHA hiện tại. Plan vẫn mở ở `RUNTIME_VERIFY` vì chưa quan sát menu trên cửa sổ macOS, chưa kiểm tra riêng SQLite/PostgreSQL và chưa có review độc lập.

