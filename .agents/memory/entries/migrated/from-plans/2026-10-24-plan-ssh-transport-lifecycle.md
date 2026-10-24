---
schema_version: 1
id: "MEM-PLAN-ssh-transport-lifecycle-bc32b3006dce"
title: "Plan archive: Ssh Transport Lifecycle"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, ssh-transport-lifecycle]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "bc32b3006dce7eb4d3f3657f2d8ecaa1d70caf11ee1e06cf2ef786b22e8ca1e8"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/ssh-transport-lifecycle/PLAN.md"
  - "docs/plans/active/ssh-transport-lifecycle/FINDINGS.md"
  - "docs/plans/active/ssh-transport-lifecycle/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/ssh-transport-lifecycle/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# SSH transport lifecycle — Plan

State: REVIEW (parent public process/provider smoke and integrated tests PASS; independent source review ACCEPT; #400 remains partial)
Branch: `main` (owner-authorized direct edits)
Issues: #435, #451, #400, #475
Baseline: `76b120724a7542de8b587a002811ddacad28156a`

## Requirements / context

The selected SSH route must not disappear between draft translation and connect/test/save. Password authentication must put `SSHPASS` on the process actually spawned, never in argv. A tunnel belongs to the provider connection through its awaited disconnect, including failure. Preflight must not contact the direct database or report an SSH handshake it did not perform.

At the baseline SHA above, `ssh/tunnel.rs:69-92` attaches the password to an unused command, `connector.rs:321-329` drops the tunnel before provider close, and native `translate_cmd.rs:25-35` discards network-driver tunnel intent except PostgreSQL. In contrast `connector.rs:332-342` already exercises SSH before TestConnection and keeps its handle through the test. The complete #400 diagnostic-stage acceptance is not already satisfied.

## Scope / architecture

- Infrastructure SSH command construction: one shared tunnel command builder, password environment on `sshpass`, omit empty identity arguments for start/test.
- Composite disconnect: remove the whole active record under the lock; release lock; await provider close; then drop tunnel on success/error.
- Native draft translation: preserve SSH for all network providers; reuse domain validation to reject unsupported SQL Server SSH on Test/Create/Update.
- Core diagnostics: defer tunneled preflight to the real connection test; omit unobserved SSH success and direct DB probes.
- Adjacent behavioral tests and this one plan directory.

## Non-goals

No SSH library, credential persistence changes, host-key relaxation, new command/event protocol, UI layout/panel redesign, or mandatory network access on Save. Parent owns final tests/build/format/review/commit/push/issue closure. No claim of real SSH DB authentication from local fixtures.

## Provider matrix

| Provider | Intent | Required evidence |
|---|---|---|
| PostgreSQL | supported generic SSH transport | password process + disconnect sockets + failed-hop routing; live SSH/DB pending |
| MySQL | preserve requested SSH and use shared transport | Test/Create/Update translation/domain validation + failed-hop routing; live SSH/MySQL pending |
| SQL Server | unsupported by existing domain validator | translated Test/Create/Update reaches explicit `ssh_tunnel` validation error, not direct connect |
| SQLite | existing local-file path unchanged | existing SQLite translation test remains authoritative; no SSH support added |

## Definition of done / acceptance

- Spawned password process receives environment secret, no secret argv, no empty `-i`.
- Socket-owning tunnel process remains alive across provider disconnect await and dies after success/error; provider error preserved.
- MySQL tunnel intent preserved for Test/Create/Update; SQL Server intent is not erased before existing validation.
- Connect/Test never contact a reachable direct DB port after the SSH hop fails.
- Diagnostics do not probe direct DB endpoints or invent SSH success for tunneled configs.
- Parent executes final gates and records results before closing independently proven #435/#451/#475.
- #400 remains OPEN: full-chain path already exists and gains failure-route regression, but separate handshake/bind/TCP/auth stage evidence and live SSH/DB authentication are not provided by this change.


## Source: `FINDINGS.md`

# SSH transport lifecycle — Findings

All baseline source claims below refer to exact SHA `76b120724a7542de8b587a002811ddacad28156a`; new edits are uncommitted candidate work, not an independently verified SHA.

| Issue | Baseline finding | Candidate disposition / evidence boundary |
|---|---|---|
| #435 P1 | `crates/infrastructure/src/ssh/tunnel.rs:69-92`: unused `ssh` command receives `SSHPASS`, spawned `sshpass` does not; both branches pass an empty identity if configured. | Shared start-command builder attaches environment to spawned command. Local executable fixtures fail unless secret environment arrives and reject secret/empty identity argv. No real SSH authentication claimed. |
| #451 P1 | `crates/infrastructure/src/connector.rs:321-329`: removing record and moving connector/inner drops remaining tunnel before provider await. | Keep whole record through await, drop tunnel after provider result. Real socket-owning child regressions cover both success and returned provider error. |
| #475 P1 | `crates/native-app/src/translate_cmd.rs:25-35`: tunnel retained only for PostgreSQL; MySQL/SQL Server are silently translated to direct configs. | Retain all network-driver intent; supported MySQL reaches shared SSH transport, SQL Server reaches existing explicit domain rejection. No panel redesign necessary for safety. |
| #400 partial | `crates/infrastructure/src/connector.rs:255-274,332-342`: TestConnection already opens/holds SSH and tests rewritten loopback config. `crates/runtime/src/worker.rs:1483-1521` calls connection API test. | Do not rewrite this already-correct path. Add local refused-hop test for PostgreSQL/MySQL Test/Connect; direct DB listener must receive no connection. |
| #400 unresolved P2/P1 acceptance gap | `crates/core/src/domain/connection_diagnostics.rs:99-110,179-189` probes direct DB endpoint and claims `SSH ok:true` without a handshake. UI draft validation already rejects empty SSH host (`crates/app/src/connection/logic.rs:22-31`). | Defer tunneled preflight and omit unobserved SSH/TLS/network success. Existing aggregate Auth result still carries full connection test error; no per-handshake/bind result is available from `Result<(), DbError>`. Keep #400 OPEN rather than claim complete stage diagnostics. |

## Validation and persistence trace

At `76b120724a7542de8b587a002811ddacad28156a`:

- `crates/core/src/domain/connection.rs:310-325` already allows PostgreSQL/MySQL SSH and rejects SQL Server/SQLite explicitly. The PostgreSQL-only comment is stale; no new support policy is invented here.
- `crates/core/src/application/connection_service.rs:77-78,334-336` validates Create and Test before persistence/provider traffic; Update delegates to the existing update service. Native Test/Create/Update all use `draft_to_domain` (`translate_cmd.rs:548-575`). Retaining unsupported tunnel intent therefore produces an explicit existing validation error, rather than losing it.
- `connection_service.rs:262-264` hydrates the stored SSH password and passes saved config to the shared connector. Credential persistence is unchanged.
- Save intentionally remains validation/persistence rather than requiring a reachable bastion. #400's save observation is not authorization to change offline-save semantics.

## Overlap / execution

- GitHub issue bodies were read during this work.
- `gh pr list --state all --limit 100 --search 'SSH in:title' --json number,title,state,headRefName,mergeCommit,url` returned `[]`. This narrow title query is not proof that no differently named PR overlaps; source correctness was evaluated directly.
- No cargo/build/test/lint/format/smoke gates executed by the worker, per assignment.
- No host-key options were weakened; no secret argv/log statement or credential storage changes added.


## Source: `VERIFICATION.md`

# SSH transport lifecycle — Verification / agent evidence

State: REVIEW; parent integration evidence below supersedes worker-only NOT RUN notes.

## Parent execution — authoritative integration update

- Baseline public `SshTunnel::start` using the same isolated process fixture fails with child exit 41 because spawned sshpass lacks SSHPASS; candidate starts, opens a real process-owned forwarding port and closes it on drop. Both calls are standalone public-API executables; no source-text assertion.
- Baseline public `CompositeConnector` smoke: start a real PostgreSQL query `pg_sleep(2), 42`, observe that exact query in PgSleep through a separate direct provider connection, then disconnect. Baseline query fails EOF after premature child kill. Candidate query drains to answer 42 before transport process is killed; smoke PASS.
- The relay/sshpass executables are explicitly local transport fixtures, not real SSH authentication. Real PostgreSQL wire protocol/database/auth was used independently; no successful real bastion handshake/password-auth claim.
- `cargo +nightly fmt --all && cargo check --workspace && cargo test --workspace`: exit 0, 1,999 passed / 0 failed / 43 ignored (`artifact://93`), includes socket lifetime success/error, no-direct-route, native Test/Create/Update and core diagnostic cases.
- Independent source reviewer ReviewScheduleSshSqlite: ACCEPT for #435/#451/#475, no introduced findings; #400 partial accepted with issue remaining OPEN.
- Focused reruns: `cargo test -p db-pro-infrastructure ssh::tunnel::tests::` 5 passed; `connector::tunnel_tests::` 3 passed; `cargo test -p db-pro-native translate_cmd::tests::` 3 passed; `cargo test -p db-pro-core connection_diagnostics::tests::` 5 passed. Each exit 0, 0 failed, 0 ignored (`artifact://133`). SQLServer SSH explicitly unsupported; SQLite local path unchanged.
- No mandatory online test on Save added. #400 still requires structured handshake/bind/TCP/auth diagnostic acceptance; do not close it.

Worker-only notes below remain a historical handoff, not the integration result.


## 1. Claim

| Field | Value |
|---|---|
| Agent identity | SshTransportLifecycle |
| Issues | #435, #451, #400, #475 |
| Task state | Review |
| Baseline SHA | `76b120724a7542de8b587a002811ddacad28156a` |
| Branch / PR | main; no PR created by worker |
| Scope | SSH process credentials/lifecycle, native tunnel intent, truthful deferred diagnostics |
| Out of scope | SSH library/authentication policy, persistence, UI redesign, structured command/API changes |

## 2. Progress checkpoint

- Source changes and behavioral regressions written; no runtime success claimed.
- Findings are recorded against baseline `76b120724a7542de8b587a002811ddacad28156a` in FINDINGS.md.
- Tests/build/lint/format/smoke: **NOT RUN**, per parent contract; pass/fail counts and exit status: n/a.
- #400 full-chain test path already existed at baseline. Separate SSH handshake/tunnel-bind/auth diagnostics remain unresolved; leave #400 OPEN.

## 3. Implementation handoff / review request

Exact committed candidate SHA: pending parent commit. Worker made no commits, pushes or issue closures.

### File inventory

- `crates/infrastructure/src/ssh/tunnel.rs`: one tunnel command builder, correct spawned-process environment, optional identity, test-only local-child constructor.
- `crates/infrastructure/src/ssh/tests/tunnel_tests.rs`: real subprocess environment/argv fixture and key/forward-target regression.
- `crates/infrastructure/src/connector.rs`: keep active connection across awaited close and add adjacent test module.
- `crates/infrastructure/src/tests/connector_tunnel_tests.rs`: socket-owning child lifetime tests (success/error), no-direct-route PostgreSQL/MySQL Test/Connect regression.
- `crates/native-app/src/translate_cmd.rs`: preserve network SSH choice, adjacent tests.
- `crates/native-app/src/tests/translate_cmd_tests.rs`: MySQL Test/Create/Update intent, SQL Server domain rejection, disabled-network-tunnel behavior.
- `crates/core/src/domain/connection_diagnostics.rs`: defer tunneled preflight; empty/deferred report does not claim success.
- `crates/core/src/domain/tests/connection_diagnostics_tests.rs`: no-direct-socket preflight and no unobserved SSH success.
- Four canonical files in `docs/plans/active/ssh-transport-lifecycle/`.

### Parent test selectors (not executed)

```sh
cargo test -p db-pro-infrastructure ssh::tunnel::tests::
cargo test -p db-pro-infrastructure connector::tunnel_tests::
cargo test -p db-pro-native translate_cmd::tests::
cargo test -p db-pro-core connection_diagnostics::tests::
```

No source-text tests: fixtures execute constructed commands, observe actual environment/argv via exit status, exchange data through child-owned TCP sockets across provider awaits, and inspect translated domain values. The local socket fixture is not DB authentication evidence.

### Independent before/after runtime smoke recipe

Use the same external harness/fixture against a temporary baseline source export and the candidate source; do not change production authentication/host-key options. Source export can be produced with `git archive 76b120724a7542de8b587a002811ddacad28156a` into a temporary directory (no branch/worktree manipulation required).

1. **Password process (#435):** put executable `sshpass` and `ssh` fixtures in a temporary directory, prepend it to **only the harness process** PATH. The `sshpass` fixture must fail unless `SSHPASS` equals the test-only secret, require `-e`, shift it, and exec its remaining argv. The `ssh` fixture must reject any argv containing that secret or `-i` with an empty identity, parse `-L`, and bind the requested loopback port until killed. Call the public `SshTunnel::start` with a password-only config from a small external Rust harness linked to `db-pro-infrastructure`; connect to the returned loopback port, drop the handle, and poll until connection is refused. Baseline fails the environment assertion before ready; candidate must open and then clean up the listener. Fixture password is synthetic, never real credentials. The checked-in `password_tunnel_process_receives_environment_without_password_or_empty_identity_arguments` executes the environment/argv half without any real SSH dependencies.
2. **Disconnect lifecycle (#451):** run both `connector::tunnel_tests::disconnect_...` selectors. These use a Python socket-owning child and a provider close that crosses an await, then exchanges actual socket bytes; assert the port closes after return and the exact provider error is preserved on failure. For a baseline comparison, overlay only the adjacent test module declaration plus the test-only `SshTunnelHandle::from_test_process` seam (no production disconnect change); baseline loses its child before the provider socket exchange, candidate succeeds. No provider/bastion credentials are needed.
3. **Route intent (#475 / #400 existing chain):** run native translation selectors, then `connect_and_test_never_bypass_a_failed_tunnel_to_a_reachable_database_port`. A reachable local DB listener and a refused local SSH endpoint are independent: PostgreSQL/MySQL Test and Connect must return an SSH failure without the DB listener accepting anything. Against baseline, MySQL Test/Create/Update translation tests expose lost intent; shared composite failure routing is already expected to pass. SQL Server draft must remain marked with SSH so existing domain validation returns `ssh_tunnel` error; disabled SSH continues to validate normally.
4. **Diagnostics (#400 partial):** run core diagnostics selectors. Bind a local DB listener and set SSH to an invalid bastion; preflight must not connect to the DB, report SSH success or summarize an empty deferred report as success. Append a failed full-chain result and ensure its original SSH error remains visible. Baseline contacts the direct endpoint and/or emits fake SSH success; candidate defers it. This does **not** prove separate handshake/bind/auth stage reporting.

### Prerequisites / evidence boundaries

| Probe | Prerequisites | What it proves |
|---|---|---|
| Password environment fixture | Unix `/bin/sh`, Rust test binaries | Spawned command environment/argv only |
| Disconnect local socket fixture | `python3`, loopback bind permissions | Awaited transport lifetime, error cleanup, process-owned socket disappears |
| Failed-hop route fixture | OpenSSH `ssh` executable, loopback bind permissions | No direct DB bypass after SSH failure, not successful SSH/DB auth |
| Native translation / core diagnostics | Rust test binaries | Actual mapping/validation and no-direct-preflight socket behavior |
| Real PostgreSQL SSH DB auth | trusted reachable SSH bastion, existing known-host trust, key/password + `sshpass` for password mode, reachable PG server and DB credentials | Full SSH/forwarding/PG protocol/auth chain; NOT VERIFIED |
| Real MySQL SSH DB auth | same SSH prerequisites plus MySQL server and credentials | Full SSH/forwarding/MySQL protocol/auth chain; NOT VERIFIED |
| SQL Server / SQLite | no bastion required for unsupported/domain or unchanged local-file behavior | Explicit existing rejection / unchanged translation; no support inferred |

CI run IDs/status: not run. Migrations/config implications: none; existing process environment `SSHPASS` is now correctly passed to spawned `sshpass`. No changes to persisted secrets or host-key handling.

## 4. Review outcome

Independent review: n/a, pending parent. Introduced versus inherited P0/P1/P2 counts: not independently assessed. #400 diagnostic granularity is an explicitly inherited/open acceptance gap, not a passing gate. Parent owns final gates, central status/changelog, candidate SHA and issue disposition.

## 5. Research / audit handoff

Source date: 2026-10-16. References: issue URLs https://github.com/truongnat/db-pro/issues/435, https://github.com/truongnat/db-pro/issues/451, https://github.com/truongnat/db-pro/issues/400, https://github.com/truongnat/db-pro/issues/475; baseline paths/anchors in FINDINGS.md. Source evidence is not executed evidence. Recommendation: close #435/#451/#475 only after parent runtime probes pass; keep #400 open for missing structured-stage acceptance. No mandatory live test on Save introduced.

## 6. Tổng kết

Đã sửa truyền mật khẩu vào đúng tiến trình SSH, giữ tunnel sống xuyên suốt disconnect kể cả nhánh lỗi, và bảo toàn lựa chọn SSH của MySQL/SQL Server để không tự ý kết nối trực tiếp. Diagnostics có tunnel được hoãn sang phép thử thực, không dựng kết quả SSH thành công giả. Đã thêm test tiến trình/socket/dịch lệnh; worker chưa chạy gate nào theo phân công. Agent chính cần chạy các selector/gate, ghi SHA và kết quả thực; #400 vẫn mở vì chưa có chẩn đoán riêng từng bước handshake/bind/auth và chưa xác thực SSH với DB thật.

