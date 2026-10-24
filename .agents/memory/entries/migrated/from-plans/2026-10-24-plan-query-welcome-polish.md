---
schema_version: 1
id: "MEM-PLAN-query-welcome-polish-788c496c7822"
title: "Plan archive: Query Welcome Polish"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, query-welcome-polish]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "788c496c7822a7225f40cc6fd9d67f2058ea34bee056f956f8ff9b0c663ea3b5"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/query-welcome-polish/PLAN.md"
  - "docs/plans/active/query-welcome-polish/FINDINGS.md"
  - "docs/plans/active/query-welcome-polish/VERIFICATION.md"
  - "docs/plans/active/query-welcome-polish/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/query-welcome-polish/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Query Select and Welcome polish

State: RUNTIME_VERIFY. Owner requested UI changes on 2026-10-06. Main checkout per owner override; baseline SHA 3dd988e2ff06ac71942c7ec331acf66389d3c5cb, previous uncommitted Data Grid/footer work preserved.

Scope: replace Query row-cap legacy ComboBox with shared Select; preserve limits and next-run policy. Reduce Welcome to a centered introduction with product name, description and supported providers. Remove its duplicate SQL composer, starter examples, connection cards, action list, draft row and only their private state/handlers. Shell actions remain the existing entry points. No SQL/service/provider change, no new dependencies.

Architecture: native egui; use DbProTheme and existing typography/spacing tokens. Shared Select owns popup styling, selected state, keyboard and dismissal. Public size and variant customize trigger context: Default/Sm and Outline/Ghost; Query uses Sm/Ghost and existing forms retain default configuration. Query adapter maps option index to Option<u64>; unknown custom caps remain unchanged until a user chooses another option. Welcome is static presentation without mutation or connection state.

Acceptance: light/dark popup uses semantic selected text/background with checkmark; four labels preserve values. Welcome remains simple and centered at three requested viewport widths. No inaccessible hidden actions, no obsolete Welcome draft state. Native captures needed; feature cannot complete with pending exact-height or interactive runtime evidence.


## Source: `FINDINGS.md`

# Findings

Baseline SHA: 3dd988e2ff06ac71942c7ec331acf66389d3c5cb. Final uncommitted source snapshot/hash will be stored under evidence/. Prior Data Grid/footer edits remain in the shared checkout and are not rewritten.

P2: Query row limit used egui::ComboBox plus selectable_value, bypassing shared Select. Screenshot shows selected item's dark text on strong blue. Shared Select already paints selected option with theme.accent_soft, theme.accent and a checkmark; reuse it rather than overriding global theme or duplicating popup logic. Adapter preserves 100/500/1000/None and existing unknown-cap behavior.

P2: Welcome duplicated composer, SQL fixtures, connection catalog, shell start actions and draft preview. Owner explicitly asked for just an attractive introduction. Static centered icon/name/copy/provider line fulfills that scope; removed the obsolete private Welcome action and prompt state. Query documents and connection catalog remain owned by their existing state modules.

Reusable lessons: caller migration matters after shared component styling changes; Welcome should not duplicate workspace controls when a simple introduction is requested. Recorded locally; global memory not modified because no memory-write authorization.

## Owner follow-up: toolbar variants

The owner rejected the form-sized row-limit trigger in the Query toolbar. Shared Select previously had fixed 38px input height, 14px trigger text, and outlined chrome. Added public SelectSize (Default / Sm) and SelectVariant (Outline / Ghost), with defaults retaining existing form callers. Sm reuses Button's 28px height, 11.5px type, icon and padding tokens; Ghost has transparent idle chrome, semantic hover fill and open/focus feedback. Query uses Sm + Ghost at 108px width. Popup navigation and option selection logic remain unchanged. Geometry tests compare compact Select to adjacent Button and preserve default form height.

Lesson: using the shared component alone does not establish visual consistency; its context-appropriate size/chrome must be supported centrally and verified in the real parent toolbar. Prior captures are historical; current variant screenshots and hashes will supersede them in the manifest.


## Source: `VERIFICATION.md`

# Verification

Baseline SHA: `3dd988e2ff06ac71942c7ec331acf66389d3c5cb`, main. Source is uncommitted; evidence/source.patch records cumulative checkout changes including prior Grid/footer work. Exact patch/binary/PNG hashes: evidence/manifest.json. No commit, merge, PR, push or CI run.

Passed final commands (CARGO_INCREMENTAL=0 for cargo build/check/test/clippy):
- git diff --check; cargo fmt --all -- --check.
- cargo check --workspace.
- cargo clippy --workspace --all-targets -- -D warnings.
- cargo test --release --locked -p db-pro-ui components::select: 10 passed, including selection substring matches.
- cargo test --release --locked -p db-pro-ui query_row_limit: 10 passed.
- cargo build --release --locked -p db-pro-native, plus --features capture.
- clean-code --diff --ratchet --ci: 14 passed / 2 warning groups / 0 failures. Existing function-size/file-size debt reviewed; Welcome's redundant custom action/composer helpers removed.

Twelve fresh native eframe captures: Query dropdown open and static Welcome, light/dark, at each requested 1280×800, 1440×900 and 1920×1080 size. PNGs are 2x resolution. Host caps larger logical heights to 838; exact 900/1080 heights remain pending. Visually inspected both light surfaces at 1280, dark Select at 1440 and dark Welcome at 1920. Welcome has no data-dependent/loading/error panels after removal.

Native captures establish layout and shared popup paint. Real mouse/keyboard row-limit selection and accessibility review remain pending; SQL/provider behavior is unchanged, with existing row-cap tests passing. No new dependency, provider adapter, transaction or migration. No benchmark or performance improvement claim.

Feature remains RUNTIME_VERIFY pending exact taller viewports, interactive UI and independent review. Final workspace suite: 1653 passed / 2 failed / 41 ignored, exit 101. UI: 995 passed / 2 failed. Failures are new_postgresql_connection_defaults_to_tls_require and explicit_disable_selection_is_preserved_on_submit, the same two baseline-reproduced connection-form failures. Removed three obsolete Welcome tests together with removed private behavior; no replacement mirror tests written. Known two connection-form failures reproduced on previous baseline export, see ../data-grid-record-view/evidence/baseline-tests.log.

## Select variant follow-up

Shared public SelectSize Default/Sm and SelectVariant Outline/Ghost implemented; Query uses compact Ghost. Default form behavior is preserved. Two new headless geometry regressions pass: compact variants match adjacent Sm Button height and center; default Select retains input form height. Select-filter suite: 12 passed / 0 failed.

Final fmt/check/workspace clippy and locked native release/capture builds passed. Full workspace release suite: 1655 passed / 2 failed / 41 ignored; same two baseline-reproduced connection-form failures, not newly introduced. Native current Query open/closed captures cover light/dark and three widths (12); prior Welcome captures retained with their original binary hashes. Larger exact heights and physical keyboard/mouse/accessibility acceptance remain pending. Source remains uncommitted; final cumulative patch and artifact hashes in manifest. No performance claim or new dependency.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — Query Select and Welcome

## 1. Claim

Agent Codex root; task Review. Baseline SHA 3dd988e2ff06ac71942c7ec331acf66389d3c5cb, branch main, PR n/a. User-authorized UI scope: shared Query row-cap Select and static Welcome introduction. Prior uncommitted Grid/footer changes preserved.

## 2. Progress checkpoint

Implementation, targeted tests, check/clippy/build and twelve native captures complete. Feature RUNTIME_VERIFY; exact taller viewports, interactive selection and independent review pending. Native egui only. Provider semantics unchanged; no database commands added.

## 3. Implementation handoff / review request

Query uses shared Select index/value adapter. Welcome removes duplicate composer/actions/connections and their private state. Capture-only hooks open Query popup and Welcome fixture. Exact cumulative source patch, binary and PNG hashes: evidence/manifest.json. Validation details: VERIFICATION.md.

## 4. Review outcome

Self-review only; independent approval n/a. Introduced P0/P1: none identified. CI not run. Self-review verdict ACCEPT WITH P2 for source handoff; no introduced P0/P1 identified, runtime acceptance pending. Final workspace suite is not green: 1653 passed / 2 baseline-reproduced failures / 41 ignored. Both failure names and baseline evidence are in VERIFICATION.md; no introduced test failure identified. Actual gate outcomes are reported in VERIFICATION.md.

## 5. Research / audit handoff

External research n/a; local current source is sufficient. Findings and reusable lessons recorded in FINDINGS.md; global memory unchanged.

Select variant follow-up: public Default/Sm sizes and Outline/Ghost variants now support context-specific presentation. Sm shares compact Button tokens; Query uses Sm + Ghost. Two real geometry regressions added. Selected suite 12/12; workspace 1655 passed / 2 inherited failures / 41 ignored. fmt/check/clippy/build passed. Twelve fresh Query open/closed native captures; provenance in manifest. Independent review and exact-height/interactive gates still pending.

## 6. Tổng kết (Vietnamese summary)

Dropdown Query chuyển sang Select dùng chung. Welcome chỉ còn phần giới thiệu, bỏ các khối thao tác trùng lặp. Các test liên quan đạt, fmt/check/clippy/build đạt và có 12 ảnh native light/dark. Exact height và thao tác chọn/keyboard thực tế còn pending; giữ RUNTIME_VERIFY.

Select chung đã có size/variant; Query dùng kiểu toolbar 28px, bỏ khung form lớn. Geometry tests chứng minh chiều cao/căn giữa đồng bộ với Button, ảnh native đã kiểm tra đóng/mở.

