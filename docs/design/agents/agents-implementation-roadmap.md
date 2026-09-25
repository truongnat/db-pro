# Agents — implementation roadmap

## 1. Source baseline and inputs

- **Exact source baseline SHA:** `b0500b9a7ecbe37b454f3d917881154c5f7a403c`
- **Input documents:** [agents-baseline.md](agents-baseline.md), [agents-feature-research.md](agents-feature-research.md), [AGENT_EVIDENCE.md](AGENT_EVIDENCE.md).
- Source facts and anchors below describe that SHA only. Recommendations and proposed acceptance criteria are not claims of implemented behavior. The source research reports no P0/P1 and four source-derived P2 gaps; those findings were not runtime reproduced.

## 2. Decision

Retain the native right-side Agent surface and constrained typed-service architecture. Do not treat its existing tool set as completion of Phase H. Resolve all four P2 consistency/trust/documentation findings before widening context or tool power, and keep Agent Workflow at `RUNTIME_VERIFY` until retrievable native-UI and live-provider evidence exists. Preserve explicit patch/query confirmation, per-document run identity, bounded context, canonical services, cancellation, and SecretStore lifecycle rather than replacing them with a general-purpose chat/automation path. (Baseline: `agents-baseline.md:14-25,38-48`; research: `agents-feature-research.md:7-20,36-49`.)

## 3. Current state

### Implemented at source baseline (source fact, not runtime proof)

- Right-side panel, Ask/Edit/Agent modes, document-scoped sessions, settings, tool activity and confirmation UI exist (`agents-baseline.md:14-18`).
- Typed context is bounded and includes query/document/schema details; result samples and tool outputs are capped. Provider calls use HTTPS defaults/configuration, bearer auth, `store: false`, and a 60-second timeout. API keys flow through runtime `SecretStore` (`agents-baseline.md:15-18`).
- Eleven typed tools execute through shared schema/query/monitoring services; workflow and executor both enforce permission checks. Patch and mutating/destructive/unknown query execution use explicit confirmation, with safety reclassification on approval. Read-only auto-run is opt-in and defaults off (`agents-baseline.md:22-25`).
- Runtime orchestration bounds steps, protects call IDs/replays, supports pause/continue and cancellation; UI routes events by document/session/run and drops stale/terminal events (`agents-baseline.md:25`).

### Not implemented / incomplete (source fact or explicitly absent evidence)

- Phase H's broader advanced AI target is not completed by current narrow `MonitoringRead` and heuristic `SuggestIndexes`; richer session/lock inspection and other planned capability remain distinct (`agents-baseline.md:22,40-42`; `agents-feature-research.md:41,45`).
- Provider/runtime evidence for PostgreSQL, SQLite, live provider calls and native UI is absent; shared service wiring is not provider qualification (`agents-baseline.md:27-36,44-48`).
- Workspace file contents are not shown in the typed provider context. Whether file context is a product feature is unresolved (`agents-baseline.md:16`; `agents-feature-research.md:40`).
- User-facing egress disclosure does not enumerate every serialized metadata/diagnostic field or the configurable endpoint destination (`agents-baseline.md:17-18`; `agents-feature-research.md:39`).

### Needs fix / disposition (existing P2 findings; do not downgrade)

1. **P2 — ExplainQuery authorization mismatch.** Ask/Edit allowlists include `ExplainQuery`, but workflow execution policy and executor reject non-Agent mode; provider-visible tool definitions may therefore disagree with executable permissions (`agents-feature-research.md:38`; `agents-baseline.md:23`). Decide whether safe EXPLAIN belongs in Ask/Edit and align published tools, workflow, executor and visible failure behavior.
2. **P2 — Egress disclosure and endpoint identity.** Serialized context includes document/version and connection identifiers, selected range and diagnostics beyond the settings note; HTTPS endpoint overrides can direct bearer-authenticated requests to a configured custom host (`agents-feature-research.md:39`; `agents-baseline.md:15,17-18`). Align consent/disclosure with actual payload and destination without claiming key-prefix proves host identity.
3. **P2 — Workspace trust/context mismatch.** Submit can be blocked based on attached workspace items although typed provider context does not include those files (`agents-feature-research.md:40`; `agents-baseline.md:16`). Choose bounded, visible file inclusion after an explicit trust/consent design, or remove the misleading gate/message.
4. **P2 — Capability and lifecycle evidence drift.** Phase H/capability docs undercount existing narrow tools and overstate missingness; full monitoring/session/lock coverage remains absent. Corrected status keeps the workflow `RUNTIME_VERIFY`, despite stale PASS wording in later verification rows (`agents-feature-research.md:41`; `agents-baseline.md:40-42`). Reconcile artifacts without upgrading lifecycle status absent evidence.

## 4. Ordered V3 backlog

Priority labels P2 are inherited from the research; proposed priorities are roadmap sequencing only, not an existing severity.

| Priority | Type | Evidence | Concrete change / outcome | Dependencies | Observable acceptance criteria |
|---|---|---|---|---|---|
| P2 | fix | ExplainQuery allowlist/policy mismatch: `agents-feature-research.md:38`; `agents-baseline.md:23` | Decide Ask/Edit safe-EXPLAIN contract; make provider tool definitions, workflow policy and executor agree; ensure rejected calls have intentional visible handling. | Product decision on safe EXPLAIN policy. | In Ask, Edit and Agent, the provider-visible tool list and actual authorization have identical permissions; a permitted ExplainQuery reaches the canonical service, and a disallowed call is rejected with a user-visible reason. No mutation path is weakened. |
| P2 | fix | Payload/disclosure and configurable endpoint: `agents-feature-research.md:39`; `agents-baseline.md:15,17-18` | Make settings/consent and release egress documentation accurately enumerate serialized context and custom HTTPS endpoint/key destination, or minimize payload to match a deliberately narrower disclosure. | Explain chosen endpoint support/config policy; no broader context expansion first. | Review of rendered disclosure against serialized request shows every field and endpoint class; a custom endpoint is never represented as the default host; consent remains explicit before sending. |
| P2 | fix | Trust gate without file fields: `agents-feature-research.md:40`; `agents-baseline.md:16` | Decide file-context scope. Either implement bounded, individually visible, consented file context after trust, or remove the dead gate/message and stop implying attached file content is sent. | Product decision; if inclusion chosen, define limits and privacy/egress disclosure before implementation. | With a file attached, untrusted/trusted outcomes match the documented policy; UI identifies exact included files; provider payload contains only consented bounded content, or no file-specific trust block remains when files are not sent. |
| P2 | fix | Drift and corrected status: `agents-feature-research.md:41`; `agents-baseline.md:40-42`; `AGENT_EVIDENCE.md:21-25,48-51` | Reconcile Phase H, capability matrix and workflow verification/evidence to distinguish existing narrow tools from missing richer scope; retain `RUNTIME_VERIFY`. | Prior three P2 contract decisions inform accurate capability text. | Docs list eleven tools and describe `MonitoringRead`/`SuggestIndexes` narrowly; no claim of full session/lock inspection or runtime pass appears without artifacts; status remains `RUNTIME_VERIFY` until required evidence is attached. |
| Proposed P1 | verification | Source paths exist, but research expressly lacks provider/UI evidence: `agents-baseline.md:27-36,44-48` | Run and preserve native UI workflow plus live provider verification, separately for configured provider/destination; capture consent, mode/tool, patch/query confirmation, cancellation, stale event and key lifecycle outcomes. | P2 mode and egress decisions; test credentials and safe isolated environment. | Retrievable artifacts show Ask/Edit/Agent behavior, tool allow/deny, confirmations and cancellation; provider request destination/payload and key lifecycle are observable without exposing secrets. No lifecycle promotion based only on source or automated unit coverage. |
| Proposed P1 | verification | Provider matrix explicitly unrun: `agents-baseline.md:27-36,48` | Exercise supported shared-service operations against PostgreSQL and SQLite independently, recording capability-specific unsupported paths. | Safe test databases; verification plan; no inference across providers. | Separate PG and SQLite evidence records show inspection, query/EXPLAIN policy, monitoring availability/limits, errors and unsupported reasons; each remains unqualified until its own run. |
| Proposed P2 | missing | Existing tools are narrow and broader Phase H scope is planned: `agents-baseline.md:22,40-42`; `agents-feature-research.md:20,45` | Only after canonical typed service and safety contracts are specified, select any richer monitoring/routine/compare/performance capability for separate product approval; do not silently broaden tools. | Contract, caps, provider boundaries, confirmation policy and successful runtime verification for the proposed service. | Any newly approved tool is typed, bounded, mode-authorized at provider/workflow/executor layers, uses canonical service APIs, reports provider limitations, and has separate PG/SQLite evidence before release claims. |

## 5. Rollout and dependency order

1. Close the four P2 consistency decisions in order: ExplainQuery authorization; payload/destination disclosure; workspace file context policy; capability/status/evidence reconciliation.
2. Keep file-context expansion and new tools blocked until scope, consent, bounds, typed service and policy are explicit.
3. Collect native UI and provider evidence; test PostgreSQL and SQLite separately. Preserve the current `RUNTIME_VERIFY` lifecycle until artifacts are retrievable and reviewed.
4. Consider additional Phase H capabilities only after the above gates, with independent contracts and verification. Existing confirmations and bounded execution remain invariants throughout.

## 6. Provider/support matrix

The baseline names PostgreSQL and SQLite as applicable through shared services, but records no runtime result. Neither row is provider-qualified by this roadmap.

| Surface | PostgreSQL | SQLite |
|---|---|---|
| Schema inspection, query and EXPLAIN tools | Source path via shared services; runtime not run (`agents-baseline.md:29-36`). | Source path via shared services; runtime not run (`agents-baseline.md:29-36`). |
| Query execution and safety/confirmation | Shared Query API/classifier path; provider scenario not run (`agents-baseline.md:31-36`). | Shared Query API/classifier path; provider scenario not run (`agents-baseline.md:31-36`). |
| Monitoring snapshot | Calls shared `MonitoringApi`; provider behavior not independently exercised (`agents-baseline.md:33`). | Calls shared `MonitoringApi`; provider behavior not independently exercised (`agents-baseline.md:33`). |
| Provider request / key persistence | Provider-neutral runtime source; no request or key scenario run (`agents-baseline.md:34,48`). | Same source path; no request or key scenario run (`agents-baseline.md:34,48`). |

## 7. Verification gates still needed (not run)

- **Not run:** native UI end-to-end mode/tool workflows, confirmation/resume/cancel, event lifecycle, or key settings flows.
- **Not run:** live provider request, endpoint override disclosure/destination, serialized-context consent, or secret-store persistence scenario.
- **Not run:** PostgreSQL or SQLite operation. Shared-source paths are not runtime/provider evidence.
- **Not run:** Phase H/capability/status artifact reconciliation as an implementation change.
- **No tests/builds were run for this roadmap.** The input research also reports no tests/builds/provider/database/UI runtime exercise (`agents-baseline.md:44-48`; `AGENT_EVIDENCE.md:25,38,50`).

## 8. Out of scope / unresolved decisions

- No source/code, tests, plans/status, release disclosure, capability matrix or Phase H documents changed by this roadmap.
- Unresolved: whether ExplainQuery is permitted in Ask/Edit; whether workspace files are sent at all; whether custom HTTPS endpoint overrides remain supported and how endpoint identity is surfaced; when retrievable native UI/provider/PG/SQLite evidence can be collected (`agents-feature-research.md:38-49`; `AGENT_EVIDENCE.md:69-70`).
- No arbitrary filesystem/shell tools, autonomous multi-step mutation, DDL bypass, or broader tool surface is authorized (`agents-feature-research.md:17-20,45`).
