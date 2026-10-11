# Agent evidence — native UI regressions after egui 2→3

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | OpenAI Codex · native UI regression repair lane |
| Issue(s) | n/a — owner request; no issue number supplied |
| Task state | Done (plan lifecycle: RUNTIME_VERIFY) |
| Baseline SHA | `c6a033da39dc1c1009201fe4c102a05fe17e3dfb`; cancellation-specific baseline: `1888e812360c2acb59cdb9c7af88582ba0290725` |
| Branch / PR | `main`; commits `92746074431b60f2f96347c940293da10952b292`, `551a35f7c159abc27cf97ac4d0f8bef86a002891`, `c02ec5ac9ac42d2c4375f94269f6d773bc642a53`; no PR |
| Scope interpretation | Repair the reported egui 2→3 regressions, cancel connection tests in the worker, and expose a distinct Cancel Test action while the test is pending. |
| Out of scope | Live provider verification remains pending; exact-height captures are unavailable on this host; no unrelated surfaces or palette redesign. |

## 2. Progress checkpoint

- Current connection-test UI implementation: `c02ec5ac9ac42d2c4375f94269f6d773bc642a53`; prior request cancellation implementation: `551a35f7c159abc27cf97ac4d0f8bef86a002891`; original UI repair: `92746074431b60f2f96347c940293da10952b292`.
- Completed acceptance rows: [x] theme-aware sidebar; [x] modal Tab/Shift+Tab traversal; [x] required/local validation and port-zero rejection; [x] reduced unintended borders; [x] request cancellation through the runtime; [x] a separate localized Cancel Test control appears while pending and its handler leaves the dialog open; [x] worker drops the test future and suppresses completion; [x] provider test resources avoid persistent connector registration.
- Remaining acceptance rows: [ ] live cancellation against each provider; [ ] exact 1440×900 and 1920×1080 captures; [ ] live keyboard interaction; [ ] Save Query As and Agent Settings captures; [ ] independent review and owner verification.
- Findings / risks: original baseline has 0 P0 / 0 P1 / 5 P2, addressed at `92746074431b60f2f96347c940293da10952b292`; cancellation baseline at `1888e812360c2acb59cdb9c7af88582ba0290725` adds one P2, addressed at `551a35f7c159abc27cf97ac4d0f8bef86a002891`. Self-review found 0 introduced P0/P1/P2. Capture shutdown still intermittently aborts in AppKit after writing images.
- Tests already run:
  - `cargo +1.95.0 fmt --all -- --check` → pass, exit 0.
  - `cargo +1.95.0 check --workspace` → pass, exit 0.
  - `cargo +1.95.0 clippy --workspace --all-targets -- -D warnings` → pass, exit 0.
  - `cargo +1.95.0 test --locked -p db-pro-ui --lib` → 1,124 passed / 0 failed / 0 ignored at original UI implementation SHA `92746074431b60f2f96347c940293da10952b292`.
  - `cargo +1.95.0 test --workspace` → 1,793 passed / 0 failed / 41 ignored (22 suites), exit 0, at cancellation implementation SHA `551a35f7c159abc27cf97ac4d0f8bef86a002891`.
  - `cargo +1.95.0 build --release --locked -p db-pro-native` → pass, exit 0.
  - `cargo +1.95.0 build --locked -p db-pro-native --features capture` → pass, exit 0.
  - `cargo +1.95.0 test --locked -p db-pro-ui --lib components::dialog::modal_guard::tests::tab_advances_focus_between_fields_inside_modal -- --nocapture` → 1 passed / 0 failed / 0 ignored, exit 0.
  - `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` → 12 pass / 4 warning categories / 0 errors, exit 0.
  - `cargo +1.95.0 build --release --locked -p db-pro-native --features capture` → pass, exit 0; two capture-feature dead-code warnings.
  - Release capture smoke at 1280×800 → light-normal exit 0; light-error, dark-normal, dark-error returned `-6` after writing screenshots. See crash report and images below; not a four-state pass.
- Follow-up UI cancellation and failed-dispatch tests, runtime future-drop and stalled-socket tests, native request-ID translation test, and SQLite `test_connection_succeeds` integration test → each 1 passed. Exact commands are recorded in VERIFICATION.md.
- Dependency / blocker changes: no dependency changes. `orca skills get computer-use --json` failed with `command not found: orca`; live keyboard review was unavailable through that route.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `c02ec5ac9ac42d2c4375f94269f6d773bc642a53` |
| Commit list | `92746074431b60f2f96347c940293da10952b292` original repair; `551a35f7c159abc27cf97ac4d0f8bef86a002891` request cancellation; `c02ec5ac9ac42d2c4375f94269f6d773bc642a53` visible pending-test cancel action |
| File / surface inventory | `crates/native-app/src/capture.rs` — deterministic connection validation capture; `crates/ui/locales/{en,ja,ko,vi,zh}.yml` — localized connection field errors; `crates/ui/src/agent_settings_view.rs` — required API-key label; `crates/ui/src/components/dialog/frame.rs` — remove footer rule; `crates/ui/src/components/dialog/modal_guard.rs` — Tab/Shift+Tab focus containment and regression test; `crates/ui/src/components/dialog/ui.rs` — remove body separator; `crates/ui/src/components/input/layout.rs` — no resting input outline; `crates/ui/src/components/input/search.rs` — preserve intentional search outline; `crates/ui/src/components/input/text.rs` — retain enabled/disabled semantic fills; `crates/ui/src/components/legacy.rs` — theme-aware sidebar buttons; `crates/ui/src/connection/advanced_panels.rs` — required SSH markers/errors and shared port predicate; `crates/ui/src/connection/form_fields.rs` — required connection markers/errors; `crates/ui/src/connection/logic.rs` — reject port 0; `crates/ui/src/connection/tests.rs` — port boundary regressions; `crates/ui/src/connection/view.rs` — SQLite database required/error state; `crates/ui/src/query_dialogs_view.rs` and `crates/ui/src/query_state.rs` — Save Query As error state; `crates/ui/src/query_save_dialog_surface_view.rs` — required marker and inline error; `crates/ui/src/theme.rs` — remove global noninteractive strokes; `crates/ui/src/tokens/component/input.rs` — document input chrome states; `crates/ui/src/workspace_actions.rs` — deterministic invalid connection capture; `docs/plans/STATUS.md`, `docs/plans/active/connection-dialog-accessibility/{PLAN.md,CHECKLIST.md,FINDINGS.md,VERIFICATION.md}`, and `CHANGELOG.md` — lifecycle, evidence, and user-facing change records. |
| Cancellation follow-up file inventory | `crates/ui/src/connection/view.rs`, `runtime_protocol.rs`, connection tests; `crates/native-app/src/translate_cmd.rs` and tests; `crates/runtime/src/worker.rs`; PostgreSQL/SQLite/MySQL/SQL Server connector test-resource paths; CHANGELOG and active plan/evidence files. |
| Acceptance mapping | Theme regression → `legacy.rs:286–307`, `theme.rs:439–447`, light/dark Query captures. Focus regression → `modal_guard.rs:72–94` plus `modal_guard.rs:209–347` test. Required fields/local errors → `connection/form_fields.rs:36–53`, `connection/advanced_panels.rs:232–260`, `query_dialogs_view.rs:220–224`, `query_save_dialog_surface_view.rs:30–38`, `agent_settings_view.rs:119–129`. Port 1–65535 → `connection/logic.rs:8–20` and tests. Border regression → `theme.rs:439–447`, `components/input/layout.rs:23–54`, `components/input/search.rs:153–163`, `components/dialog/frame.rs:27–32`. All source references above are at implementation SHA `92746074431b60f2f96347c940293da10952b292`; baseline findings are at `c6a033da39dc1c1009201fe4c102a05fe17e3dfb`. |
| Commands and counts | Exact commands and exit statuses are listed in §2 and VERIFICATION.md. Full workspace tests: 1,793 passed / 0 failed / 41 ignored. |
| CI run IDs / status | Not run; no PR or CI run was triggered. |
| Known limitations | Host caps logical screenshot height at 838: requested 1440×900 and 1920×1080 captures render at 1440×838 and 1920×838. Live provider cancellation for PostgreSQL, SQLite, MySQL, and SQL Server remains pending; no provider behavior is inferred from the local protocol socket smoke. Real keyboard interaction, independent review, and separate Save Query As / Agent Settings captures remain pending. The latest release matrix wrote all 12 images; 6/12 processes returned SIGABRT after rendering in the AppKit run-loop shutdown path. |
| Migrations / config implications | No schema/config migration or dependency change. Added localized Cancel Test/cancellation status strings and `DB_PRO_CAPTURE_CONNECTION_TEST_PENDING`; no persisted state is affected. |
| Out-of-scope changes | No palette redesign or unrelated UI surfaces; live provider runtime evidence is pending rather than assumed. |

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
 - Current cancellation capture matrix: `/tmp/db-pro-cancel-{normal,error}-{light,dark}-{1280x800,1440x900,1920x1080}.png`; all 12 images were written, six processes exited 0 and six SIGABRT after rendering. Representative light/dark, normal/error images were reviewed. Requested 1440×900 and 1920×1080 were capped to 1440×838 and 1920×838.
 - No in-progress connection-test capture fixture exists in `crates/native-app/src/capture.rs` at source SHA `551a35f7c159abc27cf97ac4d0f8bef86a002891`; loading visuals were not captured. Runtime cancellation is covered by the worker/socket smoke test.

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `c02ec5ac9ac42d2c4375f94269f6d773bc642a53` (self-review only; no independent reviewer pass) |
| Verdict | n/a — independent review, live provider verification, and owner interaction remain pending; lifecycle stays RUNTIME_VERIFY |
| P0 / P1 / P2 counts | Original UI baseline: 0 / 0 / 5 P2, addressed. Request-cancellation P2 addressed at `551a35f7c159abc27cf97ac4d0f8bef86a002891`. The missing pending-test Cancel Test affordance was addressed at `c02ec5ac9ac42d2c4375f94269f6d773bc642a53`. Self-review found 0 open P0/P1/P2. Capture shutdown SIGABRTs remain a host-runtime limitation. |
| Findings | The UI and cancellation findings are addressed; source pointers and baseline SHAs are recorded in FINDINGS.md. No open P0/P1/P2 found in self-review. |
| CI disposition | Not run; no CI workflow was triggered. Local pinned tests/build for this follow-up are listed below; earlier workspace gates passed at `551a35f7c159abc27cf97ac4d0f8bef86a002891`. |
| Next task(s) unblocked | Obtain live cancellation evidence independently for all four providers, real keyboard traversal, exact-height capture on a taller display, independent review, and separate Save Query As / Agent Settings captures. |

## 5. Research / audit handoff

- Source date: 2026-10-10 (implementation review); runtime follow-up: 2026-10-11.
- Source URLs / references: baseline repository source SHAs `c6a033da39dc1c1009201fe4c102a05fe17e3dfb` and `1888e812360c2acb59cdb9c7af88582ba0290725`; original implementation SHA `92746074431b60f2f96347c940293da10952b292`; cancellation implementation SHA `551a35f7c159abc27cf97ac4d0f8bef86a002891`; native captures listed above.
- Factual findings: original UI fixes are anchored in §3 to `92746074431b60f2f96347c940293da10952b292`; cancellation dispatch and worker cancellation are anchored in FINDINGS.md to `551a35f7c159abc27cf97ac4d0f8bef86a002891`. Runtime claims are limited to executed tests and written captures.
- Inference: none reported as fact.
- Decision / recommendation: keep the feature in RUNTIME_VERIFY until live provider cancellation, live keyboard interaction, exact viewport evidence, and independent review are collected.
- Unresolved questions: provider-specific cancellation behavior and AppKit capture shutdown remain unverified.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã sửa bốn nhóm hồi quy UI và nối nút Cancel/đóng modal tới việc huỷ test đang chạy (commit `551a35f7c159abc27cf97ac4d0f8bef86a002891`). Workspace: 1.793 test pass, 41 ignored; worker smoke xác nhận socket bị đóng. Live provider checks, keyboard thật, viewport đủ chiều cao và review độc lập còn pending; trạng thái `RUNTIME_VERIFY`.

## Follow-up handoff — visible pending-test cancellation — 2026-10-11

- Exact implementation SHA / commit: `c02ec5ac9ac42d2c4375f94269f6d773bc642a53` —
  `fix(ui): expose pending connection test cancellation`.
- Source evidence at that SHA: `crates/ui/src/connection/view.rs:236-263` dispatches the active
  `CancelTestConnection` request and keeps the dialog open; `view.rs:397-435` renders Cancel Test
  instead of the loading-only test control for `PendingConnectionOperation::Test`;
  `crates/ui/src/connection/state.rs:157-160` clears test state; the UI regression test is
  `crates/ui/src/connection/tests.rs:261-292`.
- File inventory: `crates/ui/src/connection/{view.rs,state.rs,tests.rs}`; `crates/ui/src/workspace_actions.rs`;
  `crates/ui/locales/{en,ja,ko,vi,zh}.yml`; `crates/native-app/src/capture.rs`.
- Commands: `cargo +1.95.0 fmt --all -- --check` — pass; the focused UI test
  `cancel_test_request_keeps_connection_dialog_open` — 1 passed; `cargo +1.95.0 test --locked
  -p db-pro-ui --lib cancel -- --nocapture` — 14 passed / 0 failed / 1,113 filtered;
  `cargo +1.95.0 build --release --locked -p db-pro-native --features capture` — pass with two
  pre-existing capture-feature dead-code warnings.
- Runtime evidence: six pending and six normal captures were written and inspected in light/dark at
  1280×800, requested 1440×900, and requested 1920×1080. All pending captures show Cancel Test;
  normal captures show Test Connection. Paths are listed in VERIFICATION.md. The 1440/1920
  logical heights are capped at 838.
- Known limits: no provider test was live; capture fixture sets only synthetic UI pending state.
  Seven of twelve capture subprocesses returned SIGABRT after writing screenshots in AppKit's
  `NSTouchBarFinderObservation` shutdown path. The Cancel Test click was not manually exercised:
  `orca` is unavailable, and no alternate GUI automation was used. The handler dispatch and
  keep-modal-open behavior are covered by the UI test.
- Remaining gates: live cancellation for PostgreSQL, SQLite, MySQL, and SQL Server; live pointer /
  keyboard interaction; exact-height captures on a taller display; independent review.

## 6. Tổng kết (Vietnamese summary)

Đã thêm nút **Cancel Test** riêng khi request đang chạy; nút này gửi huỷ request và giữ modal mở,
khác với nút Cancel đóng modal. Test UI, capture sáng/tối ở ba kích thước và build native đã chạy;
sample capture là trạng thái giả lập, chưa thay thế kiểm tra provider thật. Plan vẫn ở
`RUNTIME_VERIFY` cho live provider, thao tác trực tiếp và review độc lập.
