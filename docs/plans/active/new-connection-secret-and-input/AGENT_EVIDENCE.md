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
