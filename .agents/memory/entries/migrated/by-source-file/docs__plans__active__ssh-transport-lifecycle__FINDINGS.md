---
schema_version: 1
id: "MEM-SRC-c54699d50f056c97"
title: "Source mirror: docs/plans/active/ssh-transport-lifecycle/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ssh-transport-lifecycle/FINDINGS.md"
source_sha256: "c54699d50f056c977f0f89994d5aa14a8600c0afc01f2a11397723286777c988"
mirror_body_sha256: "c54699d50f056c977f0f89994d5aa14a8600c0afc01f2a11397723286777c988"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
