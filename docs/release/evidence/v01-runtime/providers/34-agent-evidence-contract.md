# Agent evidence/progress/review log contract (#114)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Tree: `main @ 0a25f31` (clean at the start of the change); fix commit recorded in `LEDGER.md`
- Issue: **#114** ([META][Agents] Standardize evidence, progress, and review log contract) — parent
  control issue **#113**

## 1. Where the contract lives

- **Template:** `docs/plans/_template/AGENT_EVIDENCE.md` — sits beside the existing plan templates
  (`PLAN.md`, `CHECKLIST.md`, `FINDINGS.md`, `VERIFICATION.md`), which is where the repo already
  stores its canonical document shapes. One file per task, placed where the task's plan lives.
- **Agent guidance:** `AGENTS.md` gained an "Agent evidence contract" section (after "PR workflow")
  that states the non-negotiable rules and points at the template. This extends the existing
  contributor/agent guide instead of creating a redundant top-level doc, as the issue asked.
- **#113 link:** a comment on #113 records the final template location.

## 2. What the template enforces (mapped to the issue's required sections)

| Issue requirement | Template section |
|---|---|
| Claim — agent identity, issue, baseline SHA, branch/PR, scope interpretation, out-of-scope | §1 (table) |
| Progress checkpoint — current HEAD, completed/remaining acceptance rows, findings/risks, tests run, dependency/blocker changes | §2 |
| Implementation handoff — exact SHA, commit list, file/surface inventory, acceptance mapping, exact commands and counts, CI run IDs/status, known limitations, migrations/config implications, explicit `out-of-scope changes` | §3 (table) |
| Review outcome — reviewed SHA, ACCEPT/BLOCK/ACCEPT WITH P2, P0/P1/P2 counts, findings with file:line, CI disposition, next tasks unblocked | §4 (table) |
| Research/audit handoff — source date, source URLs, factual findings vs inference, decision/recommendation, unresolved questions, downstream tasks | §5 |
| #113's Vietnamese handoff summary | §6 (`Tổng kết bằng tiếng Việt`) |

Acceptance specifics:
- exact SHA mandatory wherever source behaviour is asserted (rule 2 + every table row);
- skipped/failed gates cannot be represented as pass (rule 3, "contract violation");
- review separates inherited baseline failures from regressions (§4 column);
- normalized status words `Todo / Ready / In Progress / Review / Blocked / Done / Deferred` and PR
  states `Draft / Ready / Merged` (rules 5-6);
- two filled examples: one coding task (the #56 fix, with real commands and counts) and one
  research/audit task (the #121 inventory) — §Examples.

## 3. Verification

Docs-only change. `cargo check --workspace` re-run green after the commit (no code touched). Markdown
renders as a table-driven document; every referenced repo path exists
(`docs/plans/_template/`, `AGENTS.md`).

## 4. Deliberately not included

- Product code changes — none; the deliverable is the process contract.
- The #113 dispatcher rewrite itself (concurrency lanes, queue states) — out of this issue's scope.
