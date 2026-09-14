# Open-issue ledger — db-pro

**Phase:** 1 of *"work through all open issues"* — triage and execution plan only.
**Baseline audited:** `main @ a9c1174`, worktree clean, no product code changed.
**Snapshots:** [`issues-open-2026-09-14.json`](issues-open-2026-09-14.json) (136 issues, the frozen input)
plus [`issues-open-2026-09-14-late.json`](issues-open-2026-09-14-late.json) (11 issues, #217–#227, created
on GitHub a few minutes *after* the first snapshot was written). Live open count re-verified with
`gh issue list --state open --limit 300` = **147**, which is exactly the row count in `INVENTORY.md`.

## What this directory is

A triage of every open GitHub issue against the **actual tree**, not against the issue text. The
repository has 147 open issues; a large part of what they ask for landed on `main` afterwards
(the native rewrite, Gate 4, Gate 5, the RC1 closure work, the v0.1 release pipeline), and a large
part is explicitly post-v0.1 product scope. Nobody had mapped the two onto each other. This
directory is that map, plus the plan for working the leftovers.

| File | What it holds |
|---|---|
| [`INVENTORY.md`](INVENTORY.md) | One row per open issue (**147 rows** — 136 from snapshot 1 plus the 11 late arrivals in their own table), each with exactly one disposition, a proposed priority, a dependency/blocker note and a one-line evidence pointer. |
| [`PLAN.md`](PLAN.md) | The ordered execution plan: batches, per-issue complexity, rationale, dependencies, stop conditions. |
| [`LEDGER.md`](LEDGER.md) | The running traceability ledger — issue, status, commit(s), timestamp, verification evidence. Starts at the triage state and is updated after every later push. |
| [`METHOD.md`](METHOD.md) | How each disposition was decided, with the exact commands/greps used, so a later reader can re-derive or challenge the classification. |
| [`issues-open-2026-09-14.json`](issues-open-2026-09-14.json) | Raw snapshot of the 136 open issues (number, title, labels, body, timestamps, author, milestone) that this triage was computed from. Frozen; never edited by hand. |
| [`issues-open-2026-09-14-late.json`](issues-open-2026-09-14-late.json) | Raw snapshot of the 11 further open issues (#217–#227) created after the first snapshot was taken, fetched read-only before this triage was published. |

Non-goals of this directory: it is **not** a release document, it does not replace
`docs/release/*` (those stay the release-truth source), and it does not change any issue on
GitHub. Phase 1 writes only under `docs/plans/issue-ledger/`.

## Disposition vocabulary

Every issue gets **exactly one** disposition.

| Disposition | Meaning | What it commits the next phase to |
|---|---|---|
| `DONE_ON_MAIN` | The work the issue asks for is present on `main` and the row points at concrete evidence (commit, file:line, test name, release document, evidence file). Old age is not evidence. | Nothing but a confirmation pass, then closure. |
| `PARTIAL_ON_MAIN` | Part exists. The row names exactly what is missing. | A scoped continuation; re-read the row before starting. |
| `ACTIONABLE_NOW` | A real, bounded change implementable and verifiable in this repository now — no new host, credential or owner decision. | Work it. The row names the concrete change and the files. |
| `NEEDS_OWNER_DECISION` | Requires a product/governance choice by the repository owner (license, public naming/positioning, release scope, repository protection, tag policy). | Ask the owner; record the answer in `docs/release/risk-register.md` + `docs/release/0.1.0-human-decisions.md` first. |
| `NEEDS_EXTERNAL_RESOURCE` | Needs something this environment does not have: a Windows/Linux GUI host, a GUI session for interactive smoke, live web diligence, or GitHub Projects v2 mutation. | Provision the resource or explicitly reduce scope. |
| `OUT_OF_SCOPE_V01` | Explicitly post-v0.1 product scope (v0.2–v0.5 backlog tiers, the post-v0.1 Goal, Phase A–H, productivity/settings/tasks/diagnostics). | Do not start; do not count as a v0.1 gap. |
| `SUPERSEDED` | An older duplicate/planning artifact replaced by a newer issue, document or workstream — the replacement is named in the row. The usual reason here is the 2026-09-11 retirement of the React/Tauri frontend. | Close as superseded, pointing at the replacement. |
| `UNCLEAR` | Cannot be judged from the issue body plus the repository; the row must say what information is missing. | Gather the missing information first. |

Priority is the **proposed** priority of the *remaining* work:

- `P0` — stops all further v0.1 work until resolved.
- `P1` — blocks the v0.1 release until fixed or explicitly accepted by the owner.
- `P2` — v0.1 quality/evidence item; matters for a trustworthy release, not for the binary.
- `P3` — post-v0.1 / backlog; no effect on v0.1.

## How dispositions were decided

Short version; the reproducible detail is in [`METHOD.md`](METHOD.md).

1. **The tree is the authority.** Every `DONE_ON_MAIN`/`PARTIAL_ON_MAIN` row cites a file:line, a
   test name, a release document or an evidence artifact that was actually read at `a9c1174`.
   A similar-sounding document is never enough: several documents in this repo are explicitly
   marked historical (Tauri/frontend era) and one of them — the claim that the encrypted
   credential fallback is "dev/CI only, disabled in production" — is directly contradicted by
   `crates/runtime/src/lib.rs:73-75`.
2. **Container issues inherit their children.** For the goal/epic/workstream issues (#14, #21–#31,
   #27, #28, #29, #30, #31, #96) the disposition is derived from the state of their child tasks and
   the release documents, never from the container's own `Status:` line, which is stale in every
   case (they all still say "Todo"/"In Progress").
3. **Release documents and evidence trees are the primary base** for the RC1 (#74–#147), Gate 5
   (#21–#67), v0.1.0 release (#105–#111) and workstream (#25–#31) families. The specific files are
   named in the rows: `docs/release/0.1.0-readiness.md`, `docs/release/0.1.0-handoff.md`,
   `docs/release/0.1.0-final-report.md`, `docs/release/known-limitations.md`,
   `docs/release/risk-register.md`, `docs/release/provider-capability-matrix.md`,
   `docs/release/platform-prerequisites.md`, `docs/plans/STATUS.md`,
   `docs/notes/V0_1_CLOSURE_PLAN.md`, `docs/release/evidence/v01-06/*` (14 files) and
   `docs/release/evidence/v01-runtime/*` (26 files).
4. **Conservative bias.** When the evidence was not conclusive the row is `PARTIAL_ON_MAIN`, not
   `DONE_ON_MAIN`. Ten issues are `DONE_ON_MAIN` and each one has a pointer that was opened and
   read. Where an issue's *premise* is stale but the underlying defect is real, the row says so
   explicitly (for example #147: the described sequential partial-write path no longer matches the
   code, but the documented contract is still missing).
5. **Retired layers produce `SUPERSEDED`, not silent deletion.** The React/Vite/Tauri-webview
   frontend was archived on 2026-09-11 (`_archive/frontend/`, `CHANGELOG.md:46`,
   `docs/plans/STATUS.md:44`). The seven issues whose target layer no longer exists are
   `SUPERSEDED` with the replacement named. They are not "done": the *intent* behind a few of them
   (a native provider-value policy matrix, an introspection shape contract) is live work and is
   carried by other rows (#61, #62, #72).
6. **One disposition per issue, no hedging.** If an issue needed two labels it was split by asking
   "what is the *next* action?" and classified by that.

## How to keep this ledger updated

This directory is a running artifact, not a one-shot report. After every later push:

1. Move the issue's row in [`LEDGER.md`](LEDGER.md) to its new status and add the commit SHA,
   the ISO timestamp and the verification evidence (command + result, or file path).
2. If a row's disposition changes, change it in **both** `INVENTORY.md` and `LEDGER.md`, and add
   one line to the "Revisions" section at the bottom of `LEDGER.md` saying which issue moved and
   why. Never silently rewrite a disposition.
3. If a `DONE_ON_MAIN` or `PARTIAL_ON_MAIN` claim turns out to be wrong, record the correction
   rather than deleting the old row — the release documents in this repo use the same rule
   (`docs/plans/STATUS.md` retains the downgraded V01 claims as history), and a ledger that quietly
   edits itself is worthless.
4. Update `INVENTORY.md`'s generated tables by editing the classification block and re-running the
   generator (see `METHOD.md` §6) — or edit by hand *and* recount: the invariant is
   **open issues = inventory rows**.
5. Keep both frozen JSON snapshots in place. If the triage is redone against a newer snapshot, add a
   new file rather than overwriting these, and say which snapshot the rows refer to. `INVENTORY.md`
   currently keeps snapshot 1 and snapshot 2 in separate tables so the two can be recounted
   independently; if a third snapshot is needed, follow the same pattern.

## Related documents (do not duplicate them)

- `docs/release/known-limitations.md` — the release-facing truth registry (LIM-001…LIM-018).
- `docs/release/risk-register.md` — risks, dispositions and decision records (R-*, DECISION D*).
- `docs/release/0.1.0-human-decisions.md` — the owner decisions (HD-001…HD-008) that this triage
  routes to.
- `docs/plans/STATUS.md` — feature/plan state, including the evidence corrections.
- `docs/notes/V0_1_CLOSURE_PLAN.md` — the v0.1 closure plan and its status corrections.
- `docs/plans/FEATURE_LIFECYCLE.md` — the state machine a plan must pass before `COMPLETED`.
