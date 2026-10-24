---
schema_version: 1
id: "MEM-SRC-78dd0f200a803cad"
title: "Source mirror: docs/plans/active/release-qualification-matrix/AGGREGATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/release-qualification-matrix/AGGREGATION.md"
source_sha256: "78dd0f200a803cade41bd16b1939a620bb634da435ffeda2059fd317fb5f373f"
mirror_body_sha256: "78dd0f200a803cade41bd16b1939a620bb634da435ffeda2059fd317fb5f373f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Aggregate results — QA Lead only

Plan/specification only. No runner, fake result or green summary is supplied. CASES.json is canonical test design; RESULT.schema.json defines one attempt. An aggregator implementation may be written in a future authorized execution task, but must obey this contract.

## 1. Freeze and expand

Freeze `runs/<run_id>/run-manifest.json` plus `expected-instances.json` **before** worker execution. Manifest includes source/artifact hashes/features, channel/OS/platforms/providers/capabilities, fixture manifests, worker assignments, data/secret/OS permissions, operation deadlines, approved exclusions and performance budgets. Hash both manifests. Product/candidate change => new run, never mutate old results.

Expansion algorithm (deterministic, case order then lexicographic axes):

```text
for case in CASES.json.cases:
  sizes = [[1280,800]] if functional else [[1280,800],[1440,900],[1920,1080]]
  themes = [dark] if functional else [dark,light]
  for platform in case.platforms:
    for provider in case.providers:
      for theme in themes:
        for size in sizes:
          for state in case.states:
            for subvariant in case.named_subvariants:
              id = case_id::platform::provider::theme::WIDTHxHEIGHT::state::subvariant
              initial_status = NOT_TESTED
```

This produces a **candidate cell list**, not an assertion that every Cartesian combination is meaningful. Preflight must attach explicit capability/state applicability per instance. E.g. static Hub has no pending request: approved N_A with source proof for that specific state, not fabricated loading card. Positive unsupported backup can be N_A but its negative CAP-01 still required. No host/fixture/authorization/measurement availability is BLOCKED, never N_A. Preserve all candidates + exclusions so scope changes remain auditable.

No multiplication of all functional tests by visual6 cells. Functional includes every listed oracle/boundary/transition in one scenario: all check-level results required. Per-subcheck dispatch is permitted only if the lead rejoins checks without inventing a completed parent attempt; data dependencies/order must remain intact. No parent PASS from partial execution.

`case_id` ownership is fixed by DISPATCH; assignment of subset instances to multiple workers allowed only across independent desktops/fixtures or serial leases. Parent maintains unique instance→worker map. Unsupported OS/arch formats outside configured target matrix are explicit out-of-scope, not counted PASS.

## 2. Result file validation

1. Parse UTF-8 JSON and validate RESULT.schema.json. Record malformed/missing file as evidence-contract failure; never convert to product PASS.
2. Membership: instance ID exists in frozen expected manifest; case/lane/provider/platform/theme/size/state/subvariant exactly match catalog. Attempt positive integer monotonic per instance; no duplicate collision or overwrite. Worker identity and GUI lease epoch valid for action timestamps.
3. Artifact: source_sha/executable/archive/compiler/features/role match declared actual artifact. Release gating cannot consume capture/debug role or unqualified product-dirty build. CLI/manifest-only case records no GUI PID; native case must record owned PID/window. Source refs use full SHA, not HEAD shorthand.
4. Fixture: registered manifest/hash, actual target identity and isolation proof valid for this case; independent oracle not merely same UI echo. Outside sentinels unchanged and cleanup evidence attached. Wrong target => stop run, not automatic pass with edited fixture name.
5. Evidence: all references unique/resolved beneath allowed run evidence tree, no traversal/external ephemeral URI as sole proof. Recompute file hashes; screenshot/video actually inspected; preserve action/process/DB oracle logs. Redaction cannot remove expected/actual meaning; synthetic secret raw material never included.
6. Checks: create `oracle-01..N` from definition independent_oracle entries, plus named checks for every explicit boundary/action/input (e.g.99/100/101). PASS requires each performed=true/status=PASS and referenced inspected evidence. Tool action success without readback/observed effect cannot pass a check. Schema validates structure; lead also performs these semantic checks.
7. Native mode: require at least one inspected post-state screenshot + action transcript + independent state/provider oracle appropriate to assertion. Real-keyboard/pointer cases reject AXPress-only proof; native DB journey rejects CLI-only/provider-suite proof. Missing app logging not fatal if independent observable end-to-end outcomes establish path, but no guessed command/event claims.
8. Geometry: six-cell PASS requires exact **content** dimensions after OS/titlebar/DPI adjustment, actual theme/surface/state. A valid screenshot at clamped height may support behavior but leaves requested geometry BLOCKED. No image resize/padding or metadata relabeling.
9. FAIL: retain expected/actual/reproduction and CONFIRMED defect or explicit tested requirement mismatch. BLOCKED: concrete prerequisite/action attempted and affected instances. N_A: lead approval reference plus capability/state evidence, not worker convenience. NOT_TESTED: no performed proof, remain in counts.
10. Cleanup: PASS only DONE/NOT_NEEDED with required sentinel checks. Failed cleanup produces operational blocker even if assertions passed; don't delete partial attempt. Do not fabricate exit0 for SIGTERM/emergency kill.

## 3. Attempts, inherited failures and deduplication

- Every attempt append-only; never overwrite FAIL with green screenshot. Latest **eligible** completed attempt may be current result only after lead explains retry reason and resolves earlier failure/flakiness. A different source SHA cannot be silently another attempt under same run.
- Old audit pass/fail files linked as history with exact tested artifact. Current suite starts NOT_TESTED, not prefilled PASS. Known unchanged historical release blocker remains open risk even when its current cell not executed; failure list and current execution counts are separate.
- No rerun solely to confirm user-reported/known failure. Evidence review first; new-candidate regression or expressly assigned fresh qualification is separate. Fix work never inside QA worker; new candidate + affected regression and smoke selection required after remediation.
- Deduplicate defects by mechanism + exact repro + resource/target, not only screenshot wording. Preserve impacted instance IDs/providers/platforms and all evidence, classify introduced/inherited/unknown rather than assume blame from HEAD.
- P0/P1 findings require second independent **evidence review**; do not demand unsafe repeated mutation or GUI reproduction to satisfy independence. Resolve/reject/downgrade with concrete evidence before ready. P2 release blockers, including clean-shutdown failure, cannot disappear because count severity is low.

## 4. Aggregated outputs

Lead writes `runs/<run_id>/SUMMARY.md`, `coverage.json`, `defects.json` and canonical `AGENT_EVIDENCE.md`. These files are future outputs, **not created with fabricated statuses in this plan**.

Required SUMMARY sections:

1. Candidate/version/hashes, supported scope/channel, actual compiler/platforms/providers, dirty product status and permission boundaries.
2. Per requirement × provider × platform: total planned, approved N_A, eligible, PASS, FAIL, BLOCKED, NOT_TESTED; link all missing instances. Separate critical/high/normal priorities and visual states/sizes.
3. Counts of actual assertions/DB cases vs harness passed, ignored and guard-skipped. Cross-provider suite success never fills native cells. Scheduled/queued action count never counted as completed mutation count.
4. Confirmed defects P0/P1/P2 plus historical open blockers; hypothetical risks/design concerns separately. Unique defects vs impacted failed instance counts are different numbers.
5. Visual evidence requested/actual dimensions/theme/state, read/inspection provenance, state N_A and clamped/unavailable gaps.
6. Cleanup/residual fixture/process/permission risks; environment blockers with owner/unblock action.
7. Channel verdict and exact unsatisfied gates, accepted exceptions with owner/expiry. No averaged release-readiness percentage.
8. Vietnamese summary with evidence paths and next safe actions.

Allowed coverage arithmetic:

```text
planned = all frozen candidate instances
approved_NA = explicitly approved N_A instances
effective_required = required instances excluding approved_NA
executed = eligible PASS + eligible FAIL
execution_coverage = executed / effective_required  (report denominator)
pass_rate_executed = PASS / executed                (not release readiness)
unproven_required = BLOCKED + NOT_TESTED + invalid/missing evidence instances
```

Missing output record maps NOT_TESTED (or BLOCKED if known scheduling prerequisite), never zero-impact disappearance. A case with invalid PASS evidence is returned to worker for correction or remains NOT_TESTED/BLOCKED, not accepted by aggregator. Data comparisons and state snapshots are reviewed independently of worker claims.

## 5. Release decision table

| Gate | Internal engineering evaluation | External RC/beta | Public/stable |
|---|---|---|---|
| Exact artifact identity, fixture safety, truthful evidence | Required | Required | Required |
| Confirmed P0/P1 unresolved | No approved ready release; investigation only with explicit safe scope | BLOCK | BLOCK |
| Normal launch/close/Quit and persistence core journey | Known failure may be diagnostic scope, clearly disclosed | Required PASS; no unchanged shutdown crash accepted as healthy RC | Required PASS |
| Core advertised provider/native data-safety cases | Only explicitly assigned disposable evaluation scope, no production claim | Required; unproven blockers not hidden | Required on advertised providers/platforms |
| Pinned build/tests/actual candidate CI | Record local limitations, not RC qualification | Required candidate evidence or explicit channel-scoped CI exception | Required exact candidate/platform release provenance |
| Notices/legal/trust | Known private preview restrictions recorded | Channel-specific notices and unsigned risk decision, no fake trust PASS | Applicable notices complete; platform trust/signing/notarization per frozen policy |
| Credential/upgrade/backup/export | Unknowns clearly excluded from internal usage claim | Advertised scope needs appropriate evidence/explicit constrained exception | Required for advertised features, unsupported gated and documented |
| Geometry/accessibility/performance | Gaps permitted in diagnostic report, not marked passed | Critical UI operability and declared budgets/state criteria required | Advertised platform/visual/accessibility/performance claims supported; scope exceptions explicit |

No worker issues global GO. QA Lead recommends GO/NO-GO/BLOCKED for each channel; repository owner remains release/sign-off authority. Formal P0/P1 cannot be waived to ready contrary to repository policy. Intentional unsupported feature requires deterministic gating, reason and negative test, not expansion of unsupported SQL.

## 6. Suggested handoff review

Lead and independent reviewer inspect full raw result set/deduped defects, not just SUMMARY. Spot-check identity/oracle/screenshot for each lane and **all** high-risk mutation/conflict/restore/credential/quit failures. Validate report sums and incomplete checklist against expected-instances digest. If disagreement, preserve original result/evidence and record reviewer disposition; self-review is not external approval.
