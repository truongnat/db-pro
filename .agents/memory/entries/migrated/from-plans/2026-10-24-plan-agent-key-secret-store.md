---
schema_version: 1
id: "MEM-PLAN-agent-key-secret-store-019980f574db"
title: "Plan archive: Agent Key Secret Store"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, agent-key-secret-store]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "019980f574db8b7cce894213bf325d720bc4af55e484c9505a9db9de5e693769"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/agent-key-secret-store/PLAN.md"
  - "docs/plans/active/agent-key-secret-store/FINDINGS.md"
  - "docs/plans/active/agent-key-secret-store/VERIFICATION.md"
  - "docs/plans/active/agent-key-secret-store/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/agent-key-secret-store/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Agent API Key Secret Store

## Goal

Keep the Agent API key lifecycle inside the runtime `SecretStore`: load it at
startup, persist it on configuration, and delete it on “Forget key”.

## Scope

- replace native-app direct keyring access with runtime secret-store calls;
- preserve the existing key name for compatibility;
- make configuration fail before provider activation when persistence fails;
- make forgetting remove the secret from all configured stores;
- add focused tests for provider selection and command/event routing.

## Non-goals

- platform packaged-runtime qualification;
- changing the public DB Pro identity or secret-store backend policy;
- adding a new Agent provider or changing network request behavior.

## Acceptance

- no native-app code constructs or mutates a keyring entry for the Agent key;
- startup reads the saved key through `SecretStore`;
- save persists through `SecretStore` before activation;
- forget deletes through `SecretStore` and deactivates the provider only on success;
- persistence failures are surfaced and do not leave the UI busy;
- focused automated tests pass.


## Source: `FINDINGS.md`

# Findings

## P1 — Agent key bypasses the shared secret lifecycle

At baseline `941c397065a59f8e976bd212e8ee00287ca6f5ac`,
`crates/native-app/src/main.rs` constructs a raw keyring entry for
`agent/groq_api_key`, saves it best-effort, and seeds it into the process
environment. The runtime owns a configured `KeyringVault` already used by
connection and backup services, but the Agent key does not use it. “Forget key”
therefore cannot remove encrypted-file/session fallback data and its failure is
not returned to the UI.

## P2 — Configure failure can leave the UI request pending

The generic `RuntimeEvent::Failed` path does not clear
`agent_configure_request`, so a failed Agent key operation can leave the settings
panel in its busy state. The fix will route the failure through the existing
request failure handler.

## Runtime evidence

Native UI runtime verification is intentionally skipped for this slice because
the user requested moving to another coding issue. Automated evidence remains
required.


## Source: `VERIFICATION.md`

# Verification

## Baseline

- Baseline SHA: `941c397065a59f8e976bd212e8ee00287ca6f5ac`
- Branch: `fix/agent-key-secret-store`

## Commands

- `cargo fmt --all -- --check` — PASS, exit 0.
- `cargo check --workspace` — PASS, exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS, exit 0.
- `cargo test -p db-pro-runtime provider_from_api_key` — 2 passed / 0 failed / 0 ignored, exit 0.
- `cargo test -p db-pro-infrastructure secret` — 11 passed / 0 failed / 0 ignored, exit 0.
- `cargo test -p db-pro-native` — 19 passed / 0 failed / 0 ignored, exit 0.
- `cargo test -p db-pro-ui agent_key_section_discloses_what_the_ai_path_sends` — 1 passed / 0 failed / 0 ignored, exit 0.
- `cargo test --workspace` — 1137 passed / 0 failed / 38 ignored, exit 0.
- `cargo build --release --locked -p db-pro-native` — PASS, exit 0.
- `bash .skills/clean-code/scripts/clean-code-scan.sh --diff` — FAILED baseline scan: existing long translator/UI/worker functions and files remain; this change did not create a new long function or file. Exit 1.

## Provider matrix

| Provider | Supported operation | Automated evidence | Live/runtime evidence | Capability gate |
|---|---|---|---|---|
| PostgreSQL | n/a; Agent secret lifecycle is provider-neutral | pending | not applicable | n/a |
| SQLite | n/a; Agent secret lifecycle is provider-neutral | pending | not applicable | n/a |

## UI runtime evidence

Skipped by explicit user direction after the previous smoke attempt; no UI pass
is claimed. The release build is automated evidence only.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — Agent API key secret-store lifecycle

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex · focused coding lane |
| Issue(s) | #243 remainder: Agent key lifecycle (K-3/K-5) |
| Task state | Review |
| Baseline SHA | `941c397065a59f8e976bd212e8ee00287ca6f5ac` |
| Branch / PR | `fix/agent-key-secret-store` · PR not opened |
| Scope interpretation | Move the Agent API key lifecycle into the runtime-owned `SecretStore` and add a real forget path. |
| Out of scope | packaged platform smoke, debugger, SQL Server provider, UI screenshot verification |

## 2. Progress checkpoint

- Current HEAD: `6c78ebe` (full SHA below)
- Completed acceptance rows: [x] runtime-owned load/save/delete; [x] real forget action; [x] focused provider and translation tests; [x] automated gates; [x] failure clears the busy request.
- Remaining acceptance rows: native UI runtime traversal is intentionally skipped.
- Findings / risks: no introduced P0/P1/P2 remains in the fixed path; packaged keyring behavior is not runtime-qualified here.
- Tests already run: `cargo test --workspace` → 1137 passed / 0 failed / 38 ignored, exit 0.
- Dependency / blocker changes: native UI runtime verification explicitly skipped by user direction.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `6c78ebe3c087e4b14e2ac3081212b600a4d74f0f` |
| Commit list | `6c78ebe fix(agent): route API key lifecycle through secret store` |
| File / surface inventory | runtime `DbProRuntime`/worker secret lifecycle; native command/event mapping; UI forget action and error reset; native-app keyring dependency removal; focused plan/evidence |
| Acceptance mapping | raw native keyring bypass → `crates/native-app/src/main.rs`; shared lifecycle → `crates/runtime/src/lib.rs` + `crates/runtime/src/worker.rs`; provider selection → `crates/runtime/src/agent.rs`; command/event routing → `crates/native-app/src/translate_*`; forget UI/error state → `crates/ui/src/agent_view.rs`, `events.rs`, `events_query.rs`; regression tests → `translate_tests.rs` + runtime tests |
| Commands and counts | fmt/check/clippy/release pass; workspace 1137/0/38; infrastructure secret 11/0/0; native 19/0/0; runtime focused 2/0/0; UI focused 1/0/0 |
| CI run IDs / status | not run |
| Known limitations | native UI runtime evidence skipped; clean-code scanner retains inherited long-file/function failures |
| Migrations / config implications | preserve the existing `agent/groq_api_key` key name |
| Out-of-scope changes | provider protocol and packaged platform support |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `6c78ebe3c087e4b14e2ac3081212b600a4d74f0f` |
| Verdict | `ACCEPT WITH P2` — automated gates pass; UI runtime evidence skipped |
| P0 / P1 / P2 counts | introduced-by-this-SHA: 0 / 0 / 0; inherited product findings: 0 / 0 / 0 |
| Findings | clean-code scanner reports inherited long translator/UI/worker functions and files; no new long function or file was introduced |
| CI disposition | not run |
| Next task(s) unblocked | provider/platform runtime qualification remains separate |

## 5. Research / audit handoff

- Source date: 2026-09-16
- Source URLs / references: repository source at baseline SHA; issue #243
- Factual findings: recorded in `FINDINGS.md`.
- Inference: the shared runtime vault is the smallest safe owner because it already serves connection and backup secrets.
- Decision / recommendation: implement the focused runtime-owned lifecycle.
- Unresolved questions: packaged keyring behavior remains outside this slice.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã chuyển toàn bộ vòng đời Agent API key về runtime `SecretStore`, thêm Forget key
xoá thật qua vault, và thêm regression tests. Commit `6c78ebe`; workspace 1137/0/38,
clippy và release build pass. UI runtime smoke được bỏ qua theo yêu cầu.

