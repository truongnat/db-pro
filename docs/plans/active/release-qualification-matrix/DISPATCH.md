# Subagent dispatch briefs — future execution only

Không spawn QA execution từ plan này. Các lane là **role prompts**, không khẳng định đã đăng ký native Orca agents. Dùng subagent facility thật của host khi được owner cho phép; nếu không có thì tuần tự và ghi rõ. Source design baseline: `dd2bf5e4090e6d509d34c0c8c58ca95015d91640`.

## Assignment map

| Lane | Worker / skills | Owned cases | Contract |
|---|---|---|---|
| L0 | QA Lead / fixture owner; `qa-orchestrator, requirements-test-designer, release-readiness-auditor` | ENV-01, ENV-02, ENV-03 | Freeze source/scope; prepare allowed artifacts/fixtures; serialize desktop leases; own expected instances and final verdict. |
| L1 | Artifact / release / security gates; `release-readiness-auditor, test-automation-auditor, security-privacy-auditor` | REL-01, REL-02, REL-03, REL-04, SEC-03 | Build/package/provenance/notices/trust/advisory; no source changes. Any GUI launch obtains desktop lease. |
| L2 | Lifecycle / persistence / upgrade; `functional-tester, data-integrity-tester` | LIF-01, LIF-02, LIF-03, LIF-04, LIF-05, LIF-06 | Actual release launch/close/Quit/restart/dirty-state/upgrade; prioritize native shutdown regression and actual UI storage. |
| L3 | Connections / credentials / transport; `functional-tester, security-privacy-auditor` | CON-01, CON-02, CON-03, CON-04, CON-05, CON-06, CON-07, CON-08, SEC-01, SEC-02, SEC-04, CON-09 | Actual New/Edit Connection, fields, Test/Cancel, TLS/SSH and default secret-store; no real user secrets. |
| L4 | Explorer / files / workspace / settings; `functional-tester, ux-accessibility-auditor` | EXP-01, EXP-02, EXP-03, EXP-04, EXP-05, NAV-01, NAV-02, FIL-01, FIL-02, SET-01, SET-02, FIL-03, SET-03 | Navigator/palette/tabs/files/settings; path confinement and no stale resource identity. |
| L5 | Query / lifecycle / transactions; `functional-tester, logic-concurrency-tester` | QRY-01, QRY-02, QRY-03, QRY-04, QRY-05, QRY-06, QRY-07, QRY-08, QRY-09, QRY-10, EXE-01, EXE-02, EXE-03, EXE-04, QRY-11 | Actual editor buffer/inline parameters/Run/Explain/Stop; independent SQL oracles and request/document isolation. |
| L6 | Grid / mutation / conflicts; `data-integrity-tester, logic-concurrency-tester` | SAFE-01, SAFE-02, GRID-01, GRID-02, GRID-03, GRID-04, EDIT-01, EDIT-02, EDIT-03, EDIT-04, EDIT-05, EDIT-06, EDIT-07, EDIT-08 | Actual table Data editing, toolbar Save, pending/conflict/retry; second writer and full-key/data-safety oracles. |
| L7 | Schema / routine / ER; `functional-tester, data-integrity-tester, performance-reliability-tester` | SCM-01, SCM-02, SCM-03, SCM-04, SCM-05, ER-01, ER-02, ER-03, TOOL-08 | Snapshot/Diff/migration target guards, typed routine, actual ER interaction/design and measured large graphs. |
| L8 | Export / transfer / backup; `data-integrity-tester, functional-tester` | XPT-01, XPT-02, XPT-03, XPT-04, XPT-05, XPT-06, BAK-01, BAK-02, BAK-03, BAK-04, XPT-07 | Real Export Results, Copy tables and Backup/Restore; separate preview/harness/actual DB effects. |
| L9 | Tools / admin / schedule / capability; `functional-tester, security-privacy-auditor` | TOOL-01, TOOL-02, TOOL-03, TOOL-04, TOOL-05, TOOL-06, TOOL-07, TOOL-09, CAP-01 | Hub gating, PostgreSQL sessions/roles/RLS, in-app60s scheduler, visual builder; real effect not smoke-open. |
| L10 | Agent preview / live provider safety; `functional-tester, security-privacy-auditor` | AI-01, AI-02, AI-03, AI-04 | No-key preview/disclosure first; real AI only separately authorized test key/endpoint/data/spend. |
| L11 | Visual / keyboard / accessibility / responsiveness; `ui-visual-tester, ux-accessibility-auditor, performance-reliability-tester` | A11Y-01, A11Y-02, A11Y-03, PERF-01, VIS-01, VIS-02, VIS-03, VIS-04, VIS-05, VIS-06, VIS-07, VIS-08, VIS-09, VIS-10, VIS-11, VIS-12, VIS-13 | Named surface/theme/state/geometry evidence and real input modality; exact sizing and actual measurements. |

## Mandatory skill contract

- Mọi GUI worker đọc installed `orca-cli`, `computer-use`, `dbpro-orca-ui-drive`, `verification-before-completion` cùng lane skills trước action; resolve version-matched executable guide. Skill fallback về debug/capture không override requirement actual release/no-capture của plan.
- Visual review đọc thêm `frontend-ui-engineering`, `ui-design-review` và project UI guidance khi đánh giá layout/theme; **không sửa UI**. Native data/performance helpers không tự biến app thành web browser tests.
- Skill không có/command unsupported: báo prerequisite BLOCKED hoặc dùng documented supported interface; không invent tool/skill registration. Không cần cài/bật paid service hoặc sửa global config để hoàn thành packet.

## Sequence on one desktop

1. **L0 intake**: source/scope/capability/permission/sandbox policy and planned fixture registrations, no GUI product cases yet.
2. **L1 artifact preparation**: pinned gates/build/package once; advisory/notices/provenance read-only work may parallel independent fixture preparation. Cargo producer single owner.
3. **L0 final preflight ENV-01..03**: actual artifact hash, actual app/desktop lease and independent fixture proof. Freeze expected instances + immutable digest.
4. **L1 trust runtime** under GUI lease; if trust gate fails, record FAIL. Any direct-launch engineering diagnostic continuation explicitly does not qualify downloaded/trusted install.
5. GUI workers **L2 → L3 → L4 → L5 → L6 → L7 → L8 → L9 → L10 → L11**, releasing/resetting between lanes/cases. L2 known close failure must not erase other read-only evidence; recovery isolation must be restored before next worker.
6. Native cases needing default Keychain, denied keychain, upgrade account or admin cluster wait for their fixture; other independent Ready cases can proceed sequentially instead of blocking whole run.
7. Each available host runs same assignment discipline independently; no host => corresponding instances remain BLOCKED. Source/CLI review can be concurrent only when it neither replaces artifact nor mutates any active GUI fixture.
8. Lead reconciles results and assigns evidence review; reviewer can read artifacts concurrently, but any further GUI work needs a new lease and clearly justified new attempt. Only lead publishes overall verdict.

## Packet to copy into a future worker task

```text
# Goal
Execute ONLY the assigned case instance IDs from release-qualification-matrix.
Do not implement fixes, change source, publish, or expand scope.

# Read first
PLAN.md; REQUIREMENTS.md; assigned CASES.json definitions;
FIXTURES.md; RUNBOOK.md; RESULT.schema.json; AGGREGATION.md;
the assigned lane row in DISPATCH.md; canonical AGENT_EVIDENCE template.

# Immutable inputs from coordinator
run_id; candidate full SHA; product-dirty inventory; artifact path/hash/version/
compiler/features/role; exact platform; desktop lease id+epoch;
owned app PID/window; fixture manifest+hash and approved mutable resources;
capability/scope decisions; expected-instances manifest+digest;
assigned instance IDs only; output directory; operation deadlines;
secure test-secret references (never secret literals in prompt).

# Authorization
Read-only except explicit case-owned DB/file operations allowed in fixture manifest.
No production/user connections, global OS settings/known_hosts/keychain cleanup,
source edits, git resets/checkout/commits/push, installs, or external AI without
separate recorded permission. Only fixture-owned target data may be mutated.

# Execution
Acquire/acknowledge one GUI lease; revalidate PID/window/hash/fixture.
Use real native app/Orca for UI action; client CLI only fixture/oracle/cleanup.
Read actual pixels/AX after each action; record independent expected vs actual.
All assertions and boundaries must be performed before parent PASS.
If blocked, preserve evidence and give exact reason; do not replace real path
with CLI/mock/synthetic harness or borrow another provider/platform result.

# Output
runs/<run_id>/workers/<lane>/<instance_id>/attempt-<n>/result.json
plus copied evidence/hashes, sanitized action/oracle/process logs.
Also workers/<lane>/AGENT_EVIDENCE.md with sections1–6 and Vietnamese summary.
Return completed IDs/status counts; unfinished assigned IDs; defects with exact
repro/expected/actual/severity/evidence; fixture cleanup and lease-release state.

# Stop/escalate
Wrong target/outside sentinel change/suspected P0/P1 => stop mutation, retain
snapshot, notify lead; no automatic corrective SQL. Tool/focus/AX errors are
not product failures unless independent app evidence demonstrates mismatch.
```

Lead must fill all immutable inputs from observed manifests **before dispatch**; a packet missing IDs/secrets/isolation is not Ready. The packet is instruction template, not a fake result or an instruction to run now.

## Concrete UI anchors and traps

| Lane | Actual baseline affordances / oracle trap |
|---|---|
| L3 | New/Edit: Test Connection ↔ Cancel Test; Save/Update Connection. TLS Disable/Require/Verify CA/Verify Full; cert fields only verify modes. Raw fields preserve credential identity; do not build URI with secrets. |
| L5 | Inline `Parameters · N` + `Value for <name>` and secret checkbox, not a guessed parameter modal. Toolbar caps100/500/1000/100000. SQLite/MySQL ? support, PG JSONB ? not parameter; SQLServer product rewrite. |
| L6 | Table toolbar **Save**; **Pending changes**; dirty navigation **Apply Changes / Discard Changes / Cancel**. Conflict **Original / Local staged / Database current**, Keep Mine (Overwrite DB), Use Database (Discard Mine), Reload & Retry. |
| L7 | Schema Compare Snapshot / Diff now / Generate migration plan / Apply migration SQL. Data sync SQL preview-only; inspect plan direction. ER Design Mode preview is not execution. |
| L8 | Real Copy tables uses Source/Target **connection ID text fields**, Tables CSV, Conflict abort/skip/replace; both active handles needed. Run synthetic harness/Run then cancel/CSV preview are supplemental only. |
| L8 | Real query Export Results exports frozen result/visible snapshot, not unlimited DB refetch. SQL COPY only PG. Backup settings controls Create backup / Restore database / Confirm restore; SQLite restore target disconnected. |
| L9 | Saved Tasks: New SQL task/New backup task, Save task, Run, Schedule60s, Disable schedule. App must stay open; no invented daemon or arbitrary interval GUI. Visual builder Apply SQL → editor does not Run. |
| L9 | Server monitor only own marked session PID/backend_start; never current/other user backend. Roles/RLS require standalone privileged PG fixture. |
| L10 | No-key Preview is real required behavior; live response/patch needs explicitly approved test provider. Mock counts and generated fallback prose not PASS. |

These are source navigation aids @ baseline, **not runtime PASS and not immutable index numbers**. Re-resolve actual labels/locale/controls each run. Relevant source references are in REQUIREMENTS.md and FIXTURES.md.
