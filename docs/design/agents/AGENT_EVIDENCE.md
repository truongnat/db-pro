# Agent evidence — Agents feature research

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Main · product research lane |
| Issue(s) | n/a — user requested Agents after Saved Tasks |
| Task state | Done — native source baseline, workflow comparison, feature assessment, and handoff recorded |
| Baseline SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` |
| Branch / PR | `docs/files-tab-design` / no PR |
| Scope interpretation | Source-grounded assessment of the native Agent panel, provider/key lifecycle, context egress, tool authorization, SQL/patch confirmation, provider matrix, DBeaver workflow references, and Phase H/capability-document drift. |
| Out of scope | Product implementation, code fixes, plan/status transition, tests/builds, live provider requests, database operations, native UI runtime verification, and independent external review. |

## 2. Progress checkpoint

- Current source baseline: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`.
- Completed acceptance rows: [x] traced native Agent UI-to-provider/runtime workflow; [x] inspected context, key storage, tools, permissions, confirmations and cancellation; [x] compared official database Agent workflow/privacy references; [x] assessed provider context and disclosure; [x] checked Ask/Edit/Agent permission consistency, context-trust behavior, and roadmap claims; [x] wrote all three artifacts.
- Remaining rows: none for this research task. Resolving source/documentation findings and collecting PostgreSQL, SQLite, live-provider and native UI evidence remain separate work.
- Findings / risks (source claims refer to the baseline SHA):
  - **P2 mode contract mismatch:** `ExplainQuery` is allowlisted for Ask/Edit but `execution_decision` rejects every non-Agent mode, and the executor repeats the Agent-only guard (`crates/core/src/domain/agent_workflow.rs:367-382,541-570`; `core/src/domain/agent.rs:398-410`; `crates/runtime/src/agent_executor.rs:451-473`). Source-derived, not runtime-reproduced; this is not a mutation bypass.
  - **P2 egress disclosure/configuration gap:** provider payload serializes context fields including document ID/version, connection ID, selected range and diagnostics, beyond the settings note's prompt/SQL/schema/result-sample summary (`crates/core/src/domain/agent_context.rs:112-125,212-225`; `crates/runtime/src/agent.rs:379-414`; `crates/ui/src/agent_settings_view.rs:7-8,65-79`). Environment endpoint overrides allow a custom HTTPS host while the key is sent as Bearer auth (`runtime/src/agent.rs:134-152,163-183,237-245`).
  - **P2 workspace trust/context mismatch:** the submit path requires a trusted workspace when workspace context items exist, but the typed run builder passes core query/schema context without those files (`crates/ui/src/agent_actions.rs:67-79,143-179`; `crates/core/src/domain/agent_context.rs:112-125`). Source does not show file contents transmitted through this workflow.
  - **P2 stale capability/evidence documents:** Phase H and capability-matrix rows omit or call missing the existing narrow `MonitoringRead` and `SuggestIndexes` tools; monitoring tool output is still far narrower than full session/lock inspection (`docs/goals/goal-phase-h-ai.md:22-65`; `docs/notes/PRODUCT_CAPABILITY_MATRIX.md:213-233`; `crates/core/src/domain/agent.rs:220-235`; `crates/runtime/src/agent_executor.rs:287-321`). Keep Agent Workflow `RUNTIME_VERIFY`: Status correction says provider/UI passes are not retrievable even though later workflow verification rows still say PASS (`docs/plans/STATUS.md:45`; `docs/plans/active/agent-workflow/VERIFICATION.md:3,32-39`).
- Tests already run: none. Existing source test names/plans were inspected but no test was executed. No build, provider, database or native UI scenario was exercised.
- Dependency / blocker changes: none.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` — source baseline; research artifacts are uncommitted |
| Commit list | none |
| File / surface inventory | `agents-baseline.md` — source/UI/runtime/provider and provider matrix; `agents-feature-research.md` — product recommendation, official DBeaver comparison and four P2 items; this file — evidence handoff. |
| Acceptance mapping | Native Agent flow/safety/provider/key → baseline; DBeaver workflow/privacy → feature assessment; mode mismatch, context disclosure, endpoint override, workspace trust gap and doc/status drift → assessment and §2 findings; no code or lifecycle claims changed. |
| Commands and counts | Source/doc reads/search and official DBeaver docs reads only. No build/test/runtime command executed. |
| CI run IDs / status | not run |
| Known limitations | All P2s are source-derived and not runtime-reproduced. Neither supported database provider nor a live AI provider was exercised. No independent review. Existing Agent Workflow remains `RUNTIME_VERIFY`. |
| Migrations / config implications | none — no code or persistent state changed. Existing `DB_PRO_GROQ_ENDPOINT` / `DB_PRO_CODEX_ENDPOINT` environment variables were observed, not modified. |
| Out-of-scope changes | No source, tests, release disclosure, Phase H goal, capability matrix, plan/status, provider settings, database, or UI behavior changed. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` (source baseline; research artifacts uncommitted) |
| Verdict | n/a — research handoff, not an independent code review or implementation approval |
| P0 / P1 / P2 counts | introduced by this documentation-only research: 0 / 0 / 0; source-baseline contract/documentation findings: 0 / 0 / 4 |
| Findings | Four P2 source-derived findings are listed in §2 and `agents-feature-research.md` “Prioritized open work”; none was reproduced at runtime. No source-change verdict is asserted. |
| CI disposition | not run — documentation-only research |
| Next task(s) unblocked | Resolve ExplainQuery mode/tool-definition consistency; align user-facing egress scope with serialized context and endpoint overrides; decide whether trusted workspace file context is a feature or remove the dead gate; refresh Phase H/capability/evidence records; collect independently sourced PG, SQLite, provider and native UI evidence before changing lifecycle state. |

## 5. Research / audit handoff

- Source date: 2026-09-24.
- Exact repository source baseline for every source behavior claim: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`.
- Repository references at that SHA:
  - Surface/prompt/context: `crates/ui/src/app_lifecycle.rs:180-193,227-229`; `agent_view.rs:27-55`; `agent_actions.rs:67-79,118-179`; `agent_context.rs:27-99`; `crates/core/src/domain/agent_context.rs:112-145,166-227`.
  - Provider/key/egress: `crates/ui/src/agent_settings_view.rs:7-8,65-79,98-110,157-167`; `crates/runtime/src/agent.rs:13-20,127-183,190-210,225-260,379-414`; `crates/runtime/src/lib.rs:421-431`; `crates/runtime/src/worker.rs:2593-2613`; `crates/infrastructure/src/secret/keyring_vault.rs:11-22,194-229`.
  - Tool permissions/safety: `crates/core/src/domain/agent.rs:7-18,220-248,375-412`; `crates/core/src/domain/agent_workflow.rs:337-385,388-409,458-485,541-590`; `crates/runtime/src/agent_executor.rs:39-69,196-241,287-321,451-473`; `crates/runtime/src/agent_orchestrator.rs:120-194,338-547`.
  - Runtime/UI/cancellation: `crates/runtime/src/worker.rs:1091-1179,1180-1247,1248-1289`; `crates/ui/src/agent_events.rs:6-57`; `crates/ui/src/agent_confirmation.rs:20-67`; `crates/ui/src/agent_patch.rs:11-41`.
  - Product/lifecycle docs: `docs/goals/goal-phase-h-ai.md:20-65`; `docs/notes/PRODUCT_CAPABILITY_MATRIX.md:213-233`; `docs/plans/STATUS.md:45`; `docs/plans/active/agent-workflow/VERIFICATION.md:3,32-69`; active plans `agent-workflow`, `agent-key-secret-store`, and `ui-agent-workspace-polish`.
- External official references read 2026-09-24:
  - https://dbeaver.com/docs/dbeaver/AI-Smart-Assistance/
  - https://dbeaver.com/docs/dbeaver/AI-Assistance-settings/
  - https://dbeaver.com/docs/dbeaver/AI-Assistance-and-Data-Privacy/
- Factual findings: current native code has eleven typed tools, shared service execution, per-document runs, bounded context/results, explicit patch/query confirmation and runtime key persistence. Provider request construction serializes typed context/tool messages; configurable HTTPS endpoint overrides are accepted. Ask/Edit permission list and execution policy disagree specifically for ExplainQuery. Workspace-trust submission gating exists without file fields in typed AgentContext. Plan/status and product-capability statements are inconsistent with current source and corrected evidence state.
- Inference: Ask/Edit ExplainQuery calls will be rejected; the provider receives extra metadata/diagnostics beyond the disclosure summary; custom endpoints can receive both Bearer key and prompts/context; current trust gate can block a request without sending workspace files. These follow source control flow but were not observed in a running app.
- Decision / recommendation: retain the typed, constrained Agent architecture; align the four P2 consistency/trust boundaries before broadening context or tool power; preserve `RUNTIME_VERIFY` until provider and native UI evidence is retrievable.
- Unresolved questions: should Ask/Edit be permitted to use ExplainQuery; should workspace files be supported at all, and what exact consent/context UI should apply; should custom provider endpoints be supported/documented or restricted; when can PG/SQLite and native UI end-to-end evidence be captured?
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã khảo sát Agents tại SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c` và ghi ba tài liệu. Mã nguồn có 11 typed tools, context/kết quả có giới hạn, API key qua `SecretStore`, patch và SQL mutation/destructive có bước xác nhận. Bốn vấn đề P2 source-level: `ExplainQuery` được allowlist trong Ask/Edit nhưng execution policy từ chối; disclosure chưa liệt kê toàn bộ context và endpoint tùy chỉnh; trust gate workspace không tương ứng với file context trong request; tài liệu Phase H/capability/status chưa khớp source hoặc runtime evidence đã hiệu chỉnh. Không phát hiện P0/P1 trong phạm vi này. Không chạy test/build, không gọi provider, không kết nối PostgreSQL/SQLite và không kiểm tra UI runtime; Agent Workflow vẫn `RUNTIME_VERIFY`.
