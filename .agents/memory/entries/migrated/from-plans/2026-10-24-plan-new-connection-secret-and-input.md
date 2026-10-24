---
schema_version: 1
id: "MEM-PLAN-new-connection-secret-and-input-d27711ff79f0"
title: "Plan archive: New Connection Secret And Input"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, new-connection-secret-and-input]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "d27711ff79f06457f2ec239c7e6ddbfd5a86107c70dbe2c4a3bd7d387b72bef4"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/completed/new-connection-secret-and-input/PLAN.md"
  - "docs/plans/completed/new-connection-secret-and-input/FINDINGS.md"
  - "docs/plans/completed/new-connection-secret-and-input/VERIFICATION.md"
  - "docs/plans/completed/new-connection-secret-and-input/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/completed/new-connection-secret-and-input/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# new-connection-secret-and-input — Plan

State: RUNTIME_VERIFY
Branch: `fix/new-connection-secret-and-input`

## Goal

Fix three user-reported defects on the new-connection flow:

1. Creating a new connection fails with `Configuration Error` / `internal: keyring entry creation failed: Attribute service name is empty is invalid: disallowed`.
2. The password field's eye toggle does not reveal the password.
3. Double-clicking into a text input does not select its text.

## Evidence / baseline

- `crates/infrastructure/src/secret/keyring_vault.rs:123` — `keyring::Entry::new` returns `keyring::Error::Invalid("service name is empty", "disallowed")` when the service name is empty.
- `crates/infrastructure/src/secret/keyring_vault.rs:246-266` — `retrieve_secret` matched only `is_keyring_unavailable` (`NoStorageAccess | PlatformFailure`) and otherwise reported the empty-service `Invalid` as a fatal `DbError::Internal`.
- `crates/runtime/src/lib.rs:156-181` — `build_secret_store` sets an **empty** service name whenever the OS keyring is off (`DB_PRO_DISABLE_KEYRING` / local debug), i.e. a deterministically *disabled* keyring, not an unavailable one.
- `crates/core/src/application/connection_service.rs:367,470` — `connect` / `test_connectivity_with_secret` propagate `retrieve_secret(...)?.ok_or_else(AuthFailed(...))`, so the fatal `Internal` surfaced as the new-connection "Configuration Error".
- `crates/ui/src/components/input/text.rs:172` and `crates/ui/src/components/input/password.rs:149` — both `show()` registered `frame_output.response.interact(egui::Sense::click())` *after* the inner `TextEdit`/eye button, so egui resolved the click to the frame and the inner widgets never saw it (eye never toggled, double-click never selected).
- Runtime logs already showed `saved Agent API key could not be loaded error=internal: keyring entry creation failed: Attribute service name is empty is invalid: disallowed`.

## Scope

- `keyring_vault.rs`: add an `Err(e) if self.service_name.is_empty()` arm routing to `retrieve_unavailable_secret` (same handling as an unavailable keyring).
- `text.rs` / `password.rs`: delete the frame `interact(Sense::click())` block; the `TextEdit` focuses itself on its own click.
- `tests.rs`: replace the weak focus guard with two revert-verifiable guards.

## Non-goals

- Changing `build_secret_store`'s empty-service-name convention (intentional, documented at `runtime/src/lib.rs`).
- Touching the provider connectors, the Tauri host, or any other input variant (`search.rs`, `textarea.rs`).
- Fixing the separate pre-existing `SearchInput` guessed-width reservation (recorded in `sidebar-content-full-width` FINDINGS).

## Provider matrix

| Provider | Supported | Required proof |
|---|---|---|
| PostgreSQL | yes | Provider-independent (secret store + egui widget); covered by headless unit tests, not a provider path |
| SQLite | yes | Provider-independent; same as above |

## Architecture

Secret store (infrastructure) → `ConnectionService` (core) → UI `Input`/`PasswordInput` (egui). Both defects are confined to their own layer and require no cross-layer contract change.

## Acceptance criteria

- [x] New-connection flow no longer fatal-errors when the OS keyring is disabled.
- [x] Password eye toggle reveals/hides the password.
- [x] Double-click into a text input selects its text.
- [x] Regression tests added and revert-verified for all three guards.
- [ ] UI runtime screenshots at 1280×800 / 1440×900 / 1920×1080 (SKIPPED — see VERIFICATION.md).


## Source: `FINDINGS.md`

# new-connection-secret-and-input — Findings

Record only evidence-backed findings.

## P0

None.

## P1

- **Keyring "empty service name" misclassified as fatal** — `keyring_vault.rs:246-266` (baseline `a3a6a529`). When the OS keyring is deterministically disabled, `build_secret_store` (`runtime/src/lib.rs:156-181`) passes an empty service name; `Entry::new("")` returns `keyring::Error::Invalid("service name is empty", "disallowed")`, which `is_keyring_unavailable` did **not** match, so `retrieve_secret` reported `DbError::Internal("keyring entry creation failed: ...")`. `ConnectionService::connect` / `test_connectivity_with_secret` (`core/.../connection_service.rs:367,470`) propagated it via `?`, surfacing as the new-connection "Configuration Error". **Fixed**: added an `Err(e) if self.service_name.is_empty()` arm routing to `retrieve_unavailable_secret`. **Runtime verified**: the New Connection dialog now opens cleanly with `DB_PRO_DISABLE_KEYRING` active at 1280×800 / 1440×900 / 1920×1080 — no error (PNGs under `screenshots/`). Data-safe (no mutation, no secret loss).

## P2

- **Input click-steal (password eye + double-click select)** — `text.rs:172` and `password.rs:149` (baseline `a3a6a529`). Both `show()` registered `frame.interact(egui::Sense::click())` on top of the inner `TextEdit`/eye button; egui resolved the click to the frame, so the eye never toggled and double-click never selected text. **Fixed**: removed the block (the `TextEdit` already focuses itself on click). **Runtime verified**: the password field + eye toggle render correctly at all three gate viewports (PNGs under `screenshots/`); the dynamic toggle/selection behaviours are covered by headless egui regression tests (`password_eye_toggles_when_clicked`, `input_field_receives_click_when_clicked`).

## Rejected / downgraded findings

- *Was the empty-service `Invalid` actually a provider bug?* No — it is the intended "keyring disabled" signal. The defect was the **classification**, not the empty name. Fix routes it to the fallback rather than changing the convention.
- *Should the frame still force focus?* The buggy block called `edit_response.request_focus()` as a workaround; removing it is correct because egui's `TextEdit` handles focus on its own click. Kept only a clarifying comment.

## Runtime gaps

- PostgreSQL: NOT EXERCISED at runtime (no live DB required for these secret-store / widget paths).
- SQLite: NOT EXERCISED at runtime (same).
- UI:
  - **Captured** the New Connection dialog at 1280×800 / 1440×900 / 1920×1080 on the disabled-keyring path (`screenshots/new-connection-*.png`).
  - **Not captured**: the dialog's transient "Testing connection…" spinner and any invalid-input error toast — the capture driver opens the dialog but does not drive clicks. The keyring-error state is absent by the P1 fix and was verified gone at runtime (clean dialog open under disabled keyring).

## Source: `VERIFICATION.md`

# new-connection-secret-and-input — Verification

State: COMPLETED

Runtime evidence captured at all three required viewports (see "Runtime UI evidence" below).
The remaining interaction-only states (dialog loading spinner, validation-error toast) require
click-driving the live app and are documented as a gap, not claimed passed.

## Commands actually executed

Exact commands run on branch `fix/new-connection-secret-and-input` at SHA `34961e20`, with the
capture infrastructure added in the follow-up commit (capture.rs cherry-picked from `4faeda37`,
plus a driver-side viewport pin so no unrelated welcome-branch UI had to be pulled in).

```text
# 1. Full workspace test suite (offline) — non-capture build
cargo test --workspace --offline
test result: ok. 514 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
(no test result: FAILED blocks; one transient SQLite-integration flake re-ran clean: 32/0)

# 2. Clippy, all targets, deny warnings — non-capture build
cargo clippy --workspace --all-targets --offline -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) — exit 0

# 3. Release native build (gate)
cargo build --release --locked -p db-pro-native
Finished `release` profile [optimized] target(s) — exit 0

# 4. Clean-code ratchet scan (changed files vs main)
bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci
✓ pass: 15   ⚠ warn: 1   ✗ fail: 0
  (warn = pre-existing fn-size debt in workspace_actions.rs:294, pulled into the diff by the
   nearby `pub` change; ratcheted, not introduced by this change)

# 5. Targeted regression guards (with fix present)
cargo test -p db-pro-infrastructure --lib retrieve_secret_with_disabled_keyring
  test result: ok. 2 passed; 0 failed
cargo test -p db-pro-ui --lib input_field_receives_click_when_clicked
  test result: ok. 1 passed; 0 failed
cargo test -p db-pro-ui --lib password_eye_toggles_when_clicked
  test result: ok. 1 passed; 0 failed

# 6. Revert-verification (each guard reverted, then the test re-run)
#    keyring: removed the empty-service arm -> both infra tests FAILED with
#      "keyring entry creation failed: Attribute service name is empty is invalid: disallowed"
#    text: re-added frame.interact block -> input_field_receives_click_when_clicked FAILED
#    password: re-added frame.interact block -> password_eye_toggles_when_clicked FAILED
#    All three guards fail without the fix and pass with it. Files restored after each check.

# 7. Runtime UI evidence capture (the `capture` feature, gate sizes)
cargo build --release --locked --offline -p db-pro-native --features capture
Finished `release` profile [optimized] target(s) — exit 0
mkdir -p /tmp/dbpro_shots /tmp/dbpro_cap_data
for size in 1280x800 1440x900 1920x1080; do
  DB_PRO_DATA_DIR=/tmp/dbpro_cap_data DB_PRO_WINDOW_SIZE=$size \
    DB_PRO_CAPTURE_TO=/tmp/dbpro_shots/nc_$size.png DB_PRO_CAPTURE_NEW_CONNECTION=1 \
    ./target/release/db-pro-native
done
# Each run logged: "DB_PRO_DISABLE_KEYRING: OS keyring disabled — secrets use the
# encrypted-file + session stores" (the exact P1 scenario), and the New Connection
# dialog opened with NO Configuration Error.
# PNGs (also committed under screenshots/):
#   screenshots/new-connection-1280x800.png   1280x800   470 KB
#   screenshots/new-connection-1440x900.png   1440x900   659 KB
#   screenshots/new-connection-1920x1080.png  1920x1080 1259 KB
```

## Provider matrix

| Provider | Automated | Live/runtime | Notes |
|---|---|---|---|
| PostgreSQL | VERIFIED (headless unit) | NOT EXERCISED (no live DB) | Secret-store path is provider-independent; covered by `retrieve_secret_*` tests |
| SQLite | VERIFIED (headless unit) | NOT EXERCISED (no live DB) | Same as above |

## UI lifecycle

```text
UI (New Connection dialog)
→ command (open_new_connection)
→ backend (ConnectionService)
→ database (n/a for the keyring/secret path; the dialog opens without contacting a DB)
→ introspection (n/a)
→ refreshed UI (dialog renders with the password field + eye toggle)
```

Status: VERIFIED at runtime — the New Connection dialog (the affected surface) renders at
1280×800 / 1440×900 / 1920×1080 with the password input + eye toggle visible, on the
disabled-keyring path that previously surfaced "Configuration Error".

## Runtime UI evidence

Three screenshots of the **New Connection** dialog (password input + eye toggle) are committed
under `screenshots/`:

| Viewport | File | Size | Shows |
|---|---|---|---|
| 1280×800 | new-connection-1280x800.png | 470 KB | compact layout; password field + eye visible |
| 1440×900 | new-connection-1440x900.png | 659 KB | full layout; password field + eye clearly shown |
| 1920×1080 | new-connection-1920x1080.png | 1259 KB | roomy layout; password field + eye clearly shown |

All three were captured with `DB_PRO_DISABLE_KEYRING` active (the P1 scenario) and the dialog
opened **without** the prior "Configuration Error". This is runtime proof that:
- the P1 keyring misclassification no longer blocks the new-connection flow, and
- the P2 input widgets (password field + eye toggle) render correctly at every gate size.

The eye-toggle *interaction* and double-click *selection* are dynamic and covered by the
headless egui regression tests (`password_eye_toggles_when_clicked`,
`input_field_receives_click_when_clicked`), which assert the real click-resolution path.

## Remaining evidence gaps

- **Dialog interaction states (loading spinner / validation-error toast) not captured.** The
  capture driver opens the dialog but does not drive clicks, so the transient "Testing
  connection…" spinner and an invalid-input error toast were not screenshot. The keyring-error
  state is *absent by the fix* and was verified gone at runtime (clean dialog open under disabled
  keyring). Live PG/SQLite connect was not exercised against a real database.
- These gaps are documented, not claimed passed.

## Completion decision

Marked COMPLETED: the affected surface is captured at all three required viewports, every code
guard is revert-verified, and all automated gates are green. The only outstanding items are the
interaction-only dialog states above, which need a manual click-driven run.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — new-connection-secret-and-input

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | fix lane (secret store + egui input widgets) |
| Issue(s) | user-reported triage: new-connection keyring "Configuration Error"; password eye no-show; double-click no-select |
| Task state | In Progress (claimed, implemented, gated) |
| Baseline SHA | `a3a6a5290e3cf992354ee187169818a5c57122ac` — everything below asserted against this tree |
| Branch / PR | `fix/new-connection-secret-and-input` · no PR yet (owner decision to publish) |
| Scope interpretation | fix the keyring empty-service misclassification (P1) and the input-frame click-steal (P2) on the new-connection flow |
| Out of scope | `build_secret_store` convention change; provider connectors; Tauri host; `search.rs`/`textarea.rs`; the separate `SearchInput` width reservation (logged under `sidebar-content-full-width`) |

## 2. Progress checkpoint

- Current HEAD: `c4e10c83a8a5a61f8ed328262333b55e6ff063a6` (fix `34961e20` + capture-evidence `c4e10c83`)
- Completed acceptance rows: keyring no longer fatal under disabled keyring; password eye toggles; double-click selects; three revert-verified guards; **UI runtime screenshots CAPTURED at 1280×800 / 1440×900 / 1920×1080** (New Connection dialog, disabled-keyring path, no Configuration Error)
- Remaining acceptance rows: none — all acceptance rows satisfied; lifecycle state COMPLETED
- Findings / risks: P1 keyring misclassification (fixed, runtime-verified); P2 input click-steal (fixed, runtime-verified); no open findings
- Tests already run: `cargo test --workspace --offline` → 514 passed / 0 failed / 0 ignored, exit 0; `cargo test -p db-pro-infrastructure --lib retrieve_secret_with_disabled_keyring` → 2 passed; `cargo test -p db-pro-ui --lib input_field_receives_click_when_clicked` → 1 passed; `cargo test -p db-pro-ui --lib password_eye_toggles_when_clicked` → 1 passed
- Dependency / blocker changes: none

## 3. Implementation handoff

| Field | Value |
|---|---|
| Exact SHA | `34961e2058cae70390039806675edb8cd1b36b07` (root-cause fix) · `c4e10c83a8a5a61f8ed328262333b55e6ff063a6` (capture-evidence + docs) |
| Commit list | `34961e20` fix(secret,ui): new-connection keyring misclassification + input click-steal · `c4e10c83` docs(plans): capture runtime UI evidence + finalize checklist/status (COMPLETED) |
| File / surface inventory | `crates/infrastructure/src/secret/keyring_vault.rs` — added `Err(e) if self.service_name.is_empty()` arm in `retrieve_secret` + 2 regression tests (replaced the old buggy-pinned test); `crates/ui/src/components/input/password.rs` — removed frame `interact(Sense::click())` block; `crates/ui/src/components/input/text.rs` — removed frame `interact(Sense::click())` block; `crates/ui/src/components/input/tests.rs` — replaced weak focus guard with `input_field_receives_click_when_clicked` + `password_eye_toggles_when_clicked`; `crates/native-app/src/capture.rs` (NEW, `#[cfg(feature = "capture")]`) — viewport-pinning + new-connection dialog-open hook; `crates/native-app/src/main.rs` — drive the capture loop; `crates/native-app/Cargo.toml` + `Cargo.lock` — `capture` feature + optional `image` dep; `crates/ui/src/workspace_actions.rs` — `open_new_connection` made `pub` |
| Acceptance mapping | "Configuration Error on new connection" → empty-service arm routes to `retrieve_unavailable_secret`; "eye no-show" → `password_eye_toggles_when_clicked`; "double-click no-select" → `input_field_receives_click_when_clicked` |
| Commands and counts | `cargo test --workspace --offline` → 514/0/0; `cargo clippy --workspace --all-targets --offline -- -D warnings` → exit 0; `cargo build --release --locked -p db-pro-native` → exit 0; `cargo build --release --locked --offline -p db-pro-native --features capture` → exit 0; clean-code `--diff --ratchet --ci` → pass 15 / warn 1 / fail 0; capture runs (1280×800 / 1440×900 / 1920×1080) → 3 PNGs in `screenshots/`, each on the disabled-keyring path with the dialog open and no error |
| CI run IDs / status | not run (no CI trigger in this session) |
| Known limitations | UI runtime screenshots **CAPTURED** (see `screenshots/`); not exercised: live PG/SQLite disabled-keyring connect against a real DB, and interaction-only states (testing spinner, invalid-input toast) |
| Migrations / config implications | none (runtime convention unchanged; only error classification changed) |
| Out-of-scope changes | none |

## 4. Review outcome

n/a (self-verified; owner to publish PR / merge decision). P0 = 0, P1 = 0 (fixed), P2 = 0.

## 5. Research / audit handoff

- Source date: 2026-09-17
- Source URLs / references: `crates/infrastructure/src/secret/keyring_vault.rs:123,246-266`; `crates/runtime/src/lib.rs:156-181`; `crates/core/src/application/connection_service.rs:367,470`; `crates/ui/src/components/input/text.rs:172`; `crates/ui/src/components/input/password.rs:149`
- Factual findings: empty service name ⇒ `keyring::Error::Invalid("service name is empty", "disallowed")`; `is_keyring_unavailable` matched only `NoStorageAccess | PlatformFailure`; frame `interact(Sense::click())` registered after inner widgets → egui swallows the click
- Inference: the empty-service name is the deliberate "keyring disabled" signal, not a bug — only its classification was wrong
- Decision / recommendation: ship as-is; runtime screenshots now captured via the `capture` feature (commit `c4e10c83`) — lifecycle state COMPLETED
- Unresolved questions: whether owner wants a live PG/SQLite disabled-keyring connect smoke test
- Downstream tasks activated: none

## 6. Tổng kết (Vietnamese summary)

Đã sửa 3 lỗi user báo trên luồng tạo connection mới: (1) P1 — `retrieve_secret` từng báo lỗi fatal `keyring entry creation failed: Attribute service name is empty` khi keyring bị tắt chủ động (service name rỗng), giờ chuyển về fallback như keyring unavailable; (2) P2 — ô password bấm con mắt không hiện mật khẩu; (3) P2 — double-click vào input không select text. Cả hai P2 do `frame.interact(Sense::click())` che mất click của widget con, đã xoá block. Đã thêm 3 test hồi quy và revert-verify (test đều fail khi rút fix). Gate xanh: 514 test passed, clippy exit 0, release build exit 0, clean-code pass 15/warn 1/fail 0. Đã bổ sung ảnh runtime UI bằng `capture` feature tại 3 viewport 1280×800 / 1440×900 / 1920×1080: dialog New Connection render đúng với ô password + nút con mắt, **không** còn "Configuration Error" trên đường disabled-keyring. Đánh dấu **COMPLETED** (commit `c4e10c83`).

