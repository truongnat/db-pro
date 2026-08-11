# RC1 Full Product QA — Worker Handoff

## Start here

```bash
git fetch origin
git switch qa/rc1-static-audit
git pull --ff-only
```

Read in this order:

1. `AGENTS.md`
2. `REVIEW.md`
3. `docs/plans/active/rc1-full-product-qa/PLAN.md`
4. `docs/plans/active/rc1-full-product-qa/FINDINGS.md`
5. `docs/plans/active/rc1-full-product-qa/CHECKLIST.md`
6. `docs/plans/active/rc1-full-product-qa/VERIFICATION.md`

Do not modify frozen RC1 directly.

## Primary mission

Close P1 findings before any P2/new-feature work.

Do not assume a finding is correct just because this audit says so. For each P1:

- reproduce from source/test;
- CONFIRM, REJECT or DOWNGRADE with exact evidence;
- if confirmed, implement the smallest architecture-correct fix;
- add a regression test for the stated failure scenario;
- preserve PostgreSQL/SQLite separation;
- run full applicable gates;
- create a focused PR;
- request Kilo/Cubic;
- never merge yourself.

## First implementation wave

Create:

```bash
git switch -c fix/rc1-p1-precision-staged-state
```

Scope only:

- `QA-P1-01` BIGINT/i64 precision
- `QA-P1-02` preview/staged cross-resource contamination
- `QA-P1-03` staged edits bypass close/dirty lifecycle
- `QA-P1-04` SQLite table-data metadata/edit-type correctness

These issues share mutation identity/data safety concerns and must be solved coherently.

### Non-negotiable invariants

1. A database identifier/value that cannot be exactly represented by JavaScript Number must never be converted to Number.
2. A staged mutation is permanently bound to `{connectionId, schema, table, rowIdentity}` until it is applied or explicitly discarded.
3. A tab ID alone is not sufficient mutation identity when preview tabs can change resource.
4. Preview navigation must not silently move unsaved/staged work.
5. Every tab-close path uses one unsaved-work contract.
6. SQLite edit capability must come from real/introspected column metadata, not the query mapper's generic `TEXT` placeholder.
7. Backend safety remains authoritative even after frontend UX guards are added.

## Next waves after W1 merges

Sequentially branch from updated main/audit plan:

```text
W2 fix/rc1-p1-connection-lifecycle
   QA-P1-05..09

W3 fix/rc1-p1-workspace-recovery
   QA-P1-10..11

W4 fix/rc1-p1-er-large-schema
   QA-P1-12..13

W5 fix/rc1-p1-provider-types
   QA-P1-14
```

Do not keep later branches diverged from old RC1 after earlier P1 fixes merge. Rebase/retarget them onto current main.

## Required report after each wave

```text
Wave:
Branch:
Head SHA:
Confirmed findings:
Rejected/downgraded findings:
Files changed:
Tests added:
Frontend gates:
Rust gates:
Provider evidence:
P0:
P1:
P2 introduced:
Runtime evidence still pending:
PR:
```

## Stop conditions

Stop and report only when:

- a coherent wave is implemented and PR is open;
- a real external/runtime dependency prevents further proof;
- a product decision is genuinely required;
- proceeding would create unsafe ambiguity.

Do **not** stop merely after writing a plan, after one test, or to ask permission for the next obvious step inside the current wave.
