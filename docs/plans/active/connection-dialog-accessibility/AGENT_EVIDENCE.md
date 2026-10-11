# Agent evidence — native UI regressions after egui 2→3

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | OpenAI Codex · native UI regression repair lane |
| Issue(s) | n/a — owner request; no issue number supplied |
| Task state | In Progress (plan lifecycle: RUNTIME_VERIFY) |
| Baseline SHA | `c6a033da39dc1c1009201fe4c102a05fe17e3dfb` |
| Branch / PR | `main`; implementation commit `92746074431b60f2f96347c940293da10952b292`; no PR |
| Scope interpretation | Repair all four reported egui 2→3 regressions: theme application, modal keyboard traversal, required-field affordances/local validation, and unintended borders. |
| Out of scope | PostgreSQL/SQLite provider behavior, palette redesign, exact-size captures unavailable on this host, and unrelated UI surfaces. |

## 2. Progress checkpoint

- Current implementation HEAD: `92746074431b60f2f96347c940293da10952b292`.
- Completed acceptance rows: [x] legacy sidebar uses the shared theme-aware buttons; [x] modal test covers Tab and Shift+Tab; [x] validated connection fields and Save Query As expose required/error feedback; [x] Agent API key is visibly required; [x] port zero is rejected; [x] global noninteractive/resting-input/dialog strokes are reduced while explicit outlines remain.
- Remaining acceptance rows: [ ] exact 1440×900 and 1920×1080 runtime captures; [ ] live keyboard interaction; [ ] Save Query As and Agent Settings surface captures; [ ] independent review and owner verification.
- Findings / risks: baseline findings at `c6a033da39dc1c1009201fe4c102a05fe17e3dfb`: 0 P0 / 0 P1 / 5 P2 (addressed in implementation SHA `92746074431b60f2f96347c940293da10952b292`). Self-review found 0 introduced code findings. Separate release-capture anomaly: 3/4 post-render processes returned `-6` (SIGABRT); the available report points into AppKit's NSTouchBar/KVO invalidation path, but causation is not established.
- Tests already run:
  - `cargo +1.95.0 fmt --all -- --check` → pass, exit 0.
  - `cargo +1.95.0 check --workspace` → pass, exit 0.
  - `cargo +1.95.0 clippy --workspace --all-targets -- -D warnings` → pass, exit 0.
  - `cargo +1.95.0 test --locked -p db-pro-ui --lib` → 1,124 passed / 0 failed / 0 ignored, exit 0.
  - `cargo +1.95.0 test --workspace` → 1,788 passed / 0 failed / 41 ignored (22 suites), exit 0.
  - `cargo +1.95.0 build --release --locked -p db-pro-native` → pass, exit 0.
  - `cargo +1.95.0 build --locked -p db-pro-native --features capture` → pass, exit 0.
  - `cargo +1.95.0 test --locked -p db-pro-ui --lib components::dialog::modal_guard::tests::tab_advances_focus_between_fields_inside_modal -- --nocapture` → 1 passed / 0 failed / 0 ignored, exit 0.
  - `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` → 12 pass / 4 warning categories / 0 errors, exit 0.
  - `cargo +1.95.0 build --release --locked -p db-pro-native --features capture` → pass, exit 0; two capture-feature dead-code warnings.
  - Release capture smoke at 1280×800 → light-normal exit 0; light-error, dark-normal, dark-error returned `-6` after writing screenshots. See crash report and images below; not a four-state pass.
- Dependency / blocker changes: no dependency changes. `orca skills get computer-use --json` failed with `command not found: orca`; live keyboard review was unavailable through that route.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `92746074431b60f2f96347c940293da10952b292` |
| Commit list | `92746074 fix(ui): repair post-egui-upgrade regressions` |
| File / surface inventory | `crates/native-app/src/capture.rs` — deterministic connection validation capture; `crates/ui/locales/{en,ja,ko,vi,zh}.yml` — localized connection field errors; `crates/ui/src/agent_settings_view.rs` — required API-key label; `crates/ui/src/components/dialog/frame.rs` — remove footer rule; `crates/ui/src/components/dialog/modal_guard.rs` — Tab/Shift+Tab focus containment and regression test; `crates/ui/src/components/dialog/ui.rs` — remove body separator; `crates/ui/src/components/input/layout.rs` — no resting input outline; `crates/ui/src/components/input/search.rs` — preserve intentional search outline; `crates/ui/src/components/input/text.rs` — retain enabled/disabled semantic fills; `crates/ui/src/components/legacy.rs` — theme-aware sidebar buttons; `crates/ui/src/connection/advanced_panels.rs` — required SSH markers/errors and shared port predicate; `crates/ui/src/connection/form_fields.rs` — required connection markers/errors; `crates/ui/src/connection/logic.rs` — reject port 0; `crates/ui/src/connection/tests.rs` — port boundary regressions; `crates/ui/src/connection/view.rs` — SQLite database required/error state; `crates/ui/src/query_dialogs_view.rs` and `crates/ui/src/query_state.rs` — Save Query As error state; `crates/ui/src/query_save_dialog_surface_view.rs` — required marker and inline error; `crates/ui/src/theme.rs` — remove global noninteractive strokes; `crates/ui/src/tokens/component/input.rs` — document input chrome states; `crates/ui/src/workspace_actions.rs` — deterministic invalid connection capture; `docs/plans/STATUS.md`, `docs/plans/active/connection-dialog-accessibility/{PLAN.md,CHECKLIST.md,FINDINGS.md,VERIFICATION.md}`, and `CHANGELOG.md` — lifecycle, evidence, and user-facing change records. |
| Acceptance mapping | Theme regression → `legacy.rs:286–307`, `theme.rs:439–447`, light/dark Query captures. Focus regression → `modal_guard.rs:72–94` plus `modal_guard.rs:209–347` test. Required fields/local errors → `connection/form_fields.rs:36–53`, `connection/advanced_panels.rs:232–260`, `query_dialogs_view.rs:220–224`, `query_save_dialog_surface_view.rs:30–38`, `agent_settings_view.rs:119–129`. Port 1–65535 → `connection/logic.rs:8–20` and tests. Border regression → `theme.rs:439–447`, `components/input/layout.rs:23–54`, `components/input/search.rs:153–163`, `components/dialog/frame.rs:27–32`. All source references above are at implementation SHA `92746074431b60f2f96347c940293da10952b292`; baseline findings are at `c6a033da39dc1c1009201fe4c102a05fe17e3dfb`. |
| Commands and counts | Exact commands and exit statuses are listed in §2. Full workspace tests: 1,788 passed / 0 failed / 41 ignored. |
| CI run IDs / status | Not run; no PR or CI run was triggered. |
| Known limitations | Host caps logical screenshot height at 838: requested 1440×900 and 1920×1080 captures rendered at 1440×838 and 1920×838. Live keyboard interaction, independent review, and separate Save Query As / Agent Settings captures remain pending. The follow-up release capture build passed, but 3/4 capture processes SIGABRT after rendering; AppKit crash evidence is recorded in VERIFICATION.md. |
| Migrations / config implications | No database/schema/config migrations and no dependency changes. Added locale strings only. |
| Out-of-scope changes | No PostgreSQL or SQLite provider behavior changed; no palette redesign. |

### Runtime capture evidence

The capture binary was run with `DB_PRO_CAPTURE_SETTLE_FRAMES=90`; each subprocess exited 0.

- Connection dialog light/dark, normal/error, requested 1280×800: `/tmp/ui-regression-connection-{normal,error}-{light,dark}-1280x800.png`.
- Connection dialog light/dark, normal/error, requested 1440×900: `/tmp/ui-regression-connection-{normal,error}-{light,dark}-1440x900.png` (effective 1440×838).
- Connection dialog light/dark, normal/error, requested 1920×1080: `/tmp/ui-regression-connection-{normal,error}-{light,dark}-1920x1080.png` (effective 1920×838).
- Query workspace: `/tmp/ui-regression-query-{light,dark}-1280x800.png`.

Connection captures showed visible required markers and inline name/database/port errors; Query light/dark captures showed theme-consistent workspace styling. Save Query As and Agent Settings were not separately captured.

Follow-up release run (2026-10-11), using `target/release/db-pro-native` built with `--features capture`:

- `/tmp/db-pro-release-run-{normal,error}-{light,dark}-1280x800.png` — all four images were written and visually inspected; only light-normal exited 0, the other processes returned `-6`.
- Crash report: `/Users/truongdq/Library/Logs/DiagnosticReports/db-pro-native-2026-10-11-073843.ips`; main-thread stack enters AppKit `NSTouchBarFinderObservation` KVO invalidation during the NSApplication run loop. Capture source at implementation SHA `92746074431b60f2f96347c940293da10952b292` writes the image and requests `ViewportCommand::Close` (`crates/native-app/src/capture.rs:521-525`). This is a localized platform-path observation, not a proven root cause.

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `92746074431b60f2f96347c940293da10952b292` (self-review only; no independent reviewer pass) |
| Verdict | n/a — independent review and owner runtime verification remain pending; lifecycle stays RUNTIME_VERIFY |
| P0 / P1 / P2 counts | Inherited baseline: 0 / 0 / 5 P2, addressed. Self-review found 0 introduced source-code findings. Separate unresolved runtime anomaly: 3/4 release-capture processes SIGABRT after rendering; it is not attributed to source without further evidence. |
| Findings | No introduced P0/P1/P2 found in self-review. Inherited P2 findings and source pointers are recorded in `FINDINGS.md` at baseline SHA `c6a033da39dc1c1009201fe4c102a05fe17e3dfb` and implementation SHA `92746074431b60f2f96347c940293da10952b292`. |
| CI disposition | Not run; no CI workflow was triggered. Local Rust gates passed at implementation SHA `92746074431b60f2f96347c940293da10952b292`. |
| Next task(s) unblocked | Investigate whether the AppKit close-path abort is capture-only or affects normal release shutdown; obtain independent review, owner keyboard traversal check, exact-height captures on a taller display, and separate Save Query As / Agent Settings captures. |

## 5. Research / audit handoff

- Source date: 2026-10-10 (implementation review); runtime follow-up: 2026-10-11.
- Source URLs / references: baseline repository source SHA `c6a033da39dc1c1009201fe4c102a05fe17e3dfb`; implementation repository source SHA `92746074431b60f2f96347c940293da10952b292`; native captures listed above.
- Factual findings: theme, modal, form-validation, and border changes are anchored in §3 to the exact implementation SHA; baseline behavior is attributed to the exact baseline SHA. Runtime claims are limited to captures and test/command results listed above.
- Inference: none reported as fact.
- Decision / recommendation: keep the feature in RUNTIME_VERIFY until live keyboard interaction, exact viewport evidence, and independent review are collected.
- Unresolved questions: none requiring a source change; owner visual/keyboard acceptance remains pending.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã sửa bốn nhóm hồi quy UI sau nâng cấp egui, commit `92746074431b60f2f96347c940293da10952b292`; 1.788 test pass, 41 ignored. Debug captures đã xem; release capture có ảnh nhưng 3/4 tiến trình SIGABRT trong đường AppKit NSTouchBar khi đóng, chưa xác định nguyên nhân gốc. Còn kiểm tra bàn phím thật, ảnh hai form, viewport đủ chiều cao và review độc lập; trạng thái vẫn `RUNTIME_VERIFY`.
