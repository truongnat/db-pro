# Agent evidence — macOS app menu and schema cache refresh

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | `/root` — coding |
| Issue(s) | n/a |
| Task state | In Progress |
| Baseline SHA | `3dd988e2ff06ac71942c7ec331acf66389d3c5cb` |
| Branch / PR | `main` / no PR |
| Scope interpretation | Add the native macOS app menus and connect New Connection, New Query, Refresh, Refresh Cache, standard edit/window actions, and Help to working actions. |
| Out of scope | Cache policy changes, provider-specific introspection changes, non-macOS menu work. |

## 2. Progress checkpoint

- Current HEAD: `3dd988e2ff06ac71942c7ec331acf66389d3c5cb`. Implementation changes are uncommitted and are not represented by this SHA.
- Completed acceptance rows: menu tree and typed cache invalidation are implemented; active-connection and no-active-connection UI tests pass; standard and capture-feature native app checks pass.
- Remaining acceptance rows: release build, live macOS menu interaction, SQLite and PostgreSQL runtime observations.
- Findings / risks: P2 native menu/runtime behavior remains unverified on a live window. Baseline source findings and rationale are in `FINDINGS.md`.
- Tests already run: `cargo test -p db-pro-ui native_menu_refresh -- --nocapture` — 2 passed / 0 failed / 0 ignored, exit 0.
- Checks already run: `cargo check -p db-pro-native` and `cargo check -p db-pro-native --features capture` — both exit 0; `cargo fmt --all -- --check` and `git diff --check` — both exit 0.
- Skipped gate: `cargo build --release --locked -p db-pro-native` was not run because only 125 MiB remains on the full filesystem after debug compilation; building risks exhausting the host disk.
- Dependency / blocker changes: added macOS-only `muda` 0.20.0; current disk headroom prevents the release build and launching a newly linked binary for runtime evidence. `cargo metadata --locked --no-deps --format-version 1` passes, so the lockfile resolves without a lock update.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | No implementation commit created. Checked-in baseline is `3dd988e2ff06ac71942c7ec331acf66389d3c5cb`; working-tree changes are not represented by it. |
| Commit list | none |
| File / surface inventory | `crates/native-app/src/app_menu.rs` native menu construction/event dispatch; `main.rs`/`capture.rs` app integration; `crates/ui/src/app.rs`, `runtime_protocol.rs`, `schema_actions.rs`, `app_tests.rs` menu actions and typed UI command; `crates/native-app/src/translate_cmd.rs` command mapping; `crates/runtime/src/worker.rs` invalidate-before-introspect path; `translate_tests.rs` translation regression test; `Cargo.toml`/`Cargo.lock` macOS menu dependency; `docs/plans/STATUS.md` and this plan directory. |
| Acceptance mapping | Menus and menu events: `app_menu.rs`; New Connection/New Query/Refresh/Refresh Cache behavior: `app.rs`; cache invalidation before forced introspection: `worker.rs`; active and no-active connection cases: `app_tests.rs`; translation flag: `translate_tests.rs`. |
| Commands and counts | `cargo test -p db-pro-ui native_menu_refresh -- --nocapture`: 2/2 passed; `cargo check -p db-pro-native`: pass; `cargo check -p db-pro-native --features capture`: pass; `cargo metadata --locked --no-deps --format-version 1`: pass; `cargo fmt --all -- --check`: pass; `git diff --check`: pass; release build: skipped due 125 MiB available. |
| CI run IDs / status | not run |
| Known limitations | Native menu actions have not been observed in a running macOS window; SQLite/PostgreSQL provider behavior is not runtime-verified; release build is pending disk headroom. The added native command-translation unit test was not executed. |
| Migrations / config implications | macOS-only dependency; no persisted schema/config changes. |
| Out-of-scope changes | Existing unrelated working-tree changes were preserved. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | not reviewed; no implementation commit |
| Verdict | pending |
| P0 / P1 / P2 counts | not independently reviewed |
| Findings | Runtime/menu review remains pending. |
| CI disposition | not run |
| Next task(s) unblocked | Run release build and live menu/provider verification when disk headroom permits. |

## 5. Research / audit handoff

- Source date: 2026-10-06.
- Source URLs / references: repository source at baseline SHA above; `https://docs.rs/muda/0.20.0/` and upstream issue `https://github.com/tauri-apps/muda/issues/365`.
- Factual findings: Muda 0.20.0 provides the macOS NSApp menu and menu event callback; upstream issue #365 reports a Services-submenu crash on Muda 0.19.3. The app does not add a Services item; macOS runtime inspection remains required.
- Inference: menu installation compiles and should be retained by `AppMenu`, but compile evidence does not establish actual macOS rendering, responder behavior, or event delivery.
- Decision / recommendation: keep this plan active until the release build and live macOS menu interaction have been verified.
- Unresolved questions: does the native menu render as intended in the packaged app and do refresh actions complete against SQLite and PostgreSQL?
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã thêm bộ menu macOS và nối Refresh Cache qua invalidate cache theo connection rồi introspect lại. Unit test UI, metadata/lockfile và hai cấu hình `cargo check` đã qua; còn thiếu release build và kiểm tra trực tiếp trên cửa sổ macOS do ổ đĩa chỉ còn 125 MiB.
